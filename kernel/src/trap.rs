use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("umode_trap.S"));

#[repr(C, align(16))]
pub(crate) struct TrapFrame {
    pub(crate) x: [usize; 32],
    pub(crate) sepc: usize,
    pub(crate) sstatus: usize,
    pub(crate) scause: usize,
    pub(crate) stval: usize,
}

impl TrapFrame {
    pub(crate) fn new(entry: usize, stack_top: usize, gp: usize) -> Self {
        let mut frame = Self {
            x: [0; 32],
            sepc: entry,
            sstatus: (1 << 8) | (1 << 5),
            scause: 0,
            stval: 0,
        };
        frame.x[2] = stack_top;
        frame.x[3] = gp;
        frame
    }

    fn new_user(entry: usize, user_stack_top: usize) -> Self {
        let mut frame = Self {
            x: [0; 32],
            sepc: entry,
            // SPP=0 selects U-mode; SPIE=1 enables supervisor interrupts on
            // a future privilege return (the U-mode probe itself has no SIE).
            sstatus: 1 << 5,
            scause: 0,
            stval: 0,
        };
        frame.x[2] = user_stack_top;
        frame
    }
}

const _: [(); 0] = [(); core::mem::offset_of!(TrapFrame, x)];
const _: [(); 256] = [(); core::mem::offset_of!(TrapFrame, sepc)];
const _: [(); 264] = [(); core::mem::offset_of!(TrapFrame, sstatus)];
const _: [(); 272] = [(); core::mem::offset_of!(TrapFrame, scause)];
const _: [(); 280] = [(); core::mem::offset_of!(TrapFrame, stval)];
const _: [(); 288] = [(); core::mem::size_of::<TrapFrame>()];
const _: [(); 16] = [(); core::mem::align_of::<TrapFrame>()];

static BREAKPOINT_COUNT: AtomicUsize = AtomicUsize::new(0);
static FRAME_VALID: AtomicBool = AtomicBool::new(false);
static UMODE_ISOLATION_FAULT: AtomicBool = AtomicBool::new(false);
static UMODE_STACK_SWITCH_VALIDATED: AtomicBool = AtomicBool::new(false);
static UNKNOWN_SYSCALL_REJECTED: AtomicBool = AtomicBool::new(false);

#[repr(align(4096))]
struct UModeStack([u8; 4096]);
static mut UMODE_STACK: UModeStack = UModeStack([0; 4096]);
#[repr(align(4096))]
struct SupervisorTrapStack([u8; 4096]);
static mut SUPERVISOR_TRAP_STACKS: [SupervisorTrapStack; crate::scheduler::MAX_TASKS] =
    [const { SupervisorTrapStack([0; 4096]) }; crate::scheduler::MAX_TASKS];

extern "C" {
    static __gravity_umode_trap_vector: u8;
    static __kernel_start: u8;
    fn __gravity_s_trap_roundtrip_test() -> usize;
    fn __gravity_u_mode_ecall_probe() -> !;
}

pub fn install() {
    let vector = core::ptr::addr_of!(__gravity_umode_trap_vector) as usize;
    unsafe { asm!("csrw stvec, {vector}", "csrw sscratch, zero", vector = in(reg) vector, options(nostack)); }
}

pub fn smoke_test() {
    let result = unsafe { __gravity_s_trap_roundtrip_test() };
    let count = BREAKPOINT_COUNT.load(Ordering::Acquire);
    let frame_valid = FRAME_VALID.load(Ordering::Acquire);
    if result == 0 && count == 1 && frame_valid {
        crate::console_write("S-mode trap GPR save/restore + sret: PASS\n");
    } else {
        crate::console_write("S-mode trap round-trip: FAIL\n");
        crate::halt();
    }
}

/// Build the initial U-mode frame on its private supervisor trap stack.
pub(crate) unsafe fn create_user_task_frame(task_id: usize) -> *mut TrapFrame {
    if task_id >= crate::scheduler::MAX_TASKS {
        return core::ptr::null_mut();
    }
    let user_base = core::ptr::addr_of_mut!(UMODE_STACK.0).cast::<u8>();
    let user_top = user_base.add(4096) as usize;
    let supervisor_base = core::ptr::addr_of_mut!(SUPERVISOR_TRAP_STACKS[task_id].0).cast::<u8>();
    let supervisor_top = supervisor_base.add(4096) as usize;
    let frame = (supervisor_top - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
    frame.write(TrapFrame::new_user(__gravity_u_mode_ecall_probe as *const () as usize, user_top));
    frame
}

#[no_mangle]
pub extern "C" fn gravity_s_trap_handler(frame_ptr: *mut TrapFrame) -> *mut TrapFrame {
    let frame = unsafe { &mut *frame_ptr };
    let interrupt_bit = 1usize << (usize::BITS - 1);

    if frame.scause == (interrupt_bit | 5) {
        return crate::runtime::on_timer(frame_ptr);
    }

    if frame.scause == 3 {
        let expected_stack = frame_ptr as usize + 288;
        let valid = frame.x[0] == 0
            && frame.x[2] == expected_stack
            && frame.x[2] & 0x0f == 0
            && frame.sstatus & (1 << 8) != 0;
        FRAME_VALID.store(valid, Ordering::Release);
        let first_half = unsafe { (frame.sepc as *const u16).read_volatile() };
        let instruction_len = if first_half & 0b11 == 0b11 { 4 } else { 2 };
        frame.sepc = match frame.sepc.checked_add(instruction_len) {
            Some(next_pc) => next_pc,
            None => crate::halt(),
        };
        BREAKPOINT_COUNT.fetch_add(1, Ordering::Release);
        return frame_ptr;
    }

    crate::console_write("Unexpected S-mode trap; scause=0x");
    crate::print_hex(frame.scause);
    crate::console_write(" sepc=0x");
    crate::print_hex(frame.sepc);
    crate::console_write(" stval=0x");
    crate::print_hex(frame.stval);
    crate::console_write("\n");
    crate::halt()
}

pub(crate) fn user_stack_bounds() -> (usize, usize) {
    let start = unsafe { core::ptr::addr_of!(UMODE_STACK.0) as usize };
    (start, start + 4096)
}

/// Return the base of each scheduler slot's page-aligned supervisor trap stack.
pub(crate) fn supervisor_trap_stack_starts() -> [usize; crate::scheduler::MAX_TASKS] {
    core::array::from_fn(|task_id| unsafe {
        core::ptr::addr_of!(SUPERVISOR_TRAP_STACKS[task_id].0) as usize
    })
}

/// Check that every slot is page-aligned, exactly one page, and non-overlapping.
pub(crate) fn supervisor_trap_stacks_layout_valid() -> bool {
    let starts = supervisor_trap_stack_starts();
    for task_id in 0..crate::scheduler::MAX_TASKS {
        if starts[task_id] & 0xfff != 0 {
            return false;
        }
        if task_id > 0 && starts[task_id] != starts[task_id - 1] + 4096 {
            return false;
        }
    }
    true
}

fn supervisor_trap_stack_bounds(task_id: usize) -> Option<(usize, usize)> {
    if task_id >= crate::scheduler::MAX_TASKS {
        return None;
    }
    let start = unsafe { core::ptr::addr_of!(SUPERVISOR_TRAP_STACKS[task_id].0) as usize };
    Some((start, start + 4096))
}

/// The assembly probe never changes its user sp. Require the saved frame to
/// occupy the top 288 bytes of the supervisor-owned stack and preserve that
/// exact user stack pointer across every U-origin trap.
fn validate_user_trap_frame(frame_ptr: *mut TrapFrame) -> bool {
    if frame_ptr.is_null() {
        return false;
    }

    let task_id = match crate::runtime::current_task_id() {
        Some(task_id) => task_id,
        None => return false,
    };
    let (trap_start, trap_end) = match supervisor_trap_stack_bounds(task_id) {
        Some(bounds) => bounds,
        None => return false,
    };
    let frame_start = frame_ptr as usize;
    let frame_end = match frame_start.checked_add(core::mem::size_of::<TrapFrame>()) {
        Some(end) => end,
        None => return false,
    };
    let expected_frame_start = match trap_end.checked_sub(core::mem::size_of::<TrapFrame>()) {
        Some(start) => start,
        None => return false,
    };
    if frame_start != expected_frame_start
        || frame_start < trap_start
        || frame_end > trap_end
        || frame_start & 0x0f != 0
    {
        return false;
    }

    let (_, user_stack_end) = user_stack_bounds();
    unsafe { (*frame_ptr).x[2] == user_stack_end }
}

#[no_mangle]
pub extern "C" fn gravity_u_trap_handler(frame_ptr: *mut TrapFrame) -> *mut TrapFrame {
    if !validate_user_trap_frame(frame_ptr) {
        crate::console_write("U-mode supervisor trap-stack switch + user SP preservation: FAIL\n");
        crate::halt();
    }
    let frame = unsafe { &mut *frame_ptr };
    let from_user = frame.sstatus & (1 << 8) == 0;
    if !from_user {
        crate::console_write("U-mode trap gate: rejected S-origin frame\n");
        crate::halt();
    }
    if !UMODE_STACK_SWITCH_VALIDATED.swap(true, Ordering::AcqRel) {
        crate::console_write("U-mode supervisor trap-stack switch + user SP preservation: PASS\n");
    }

    let interrupt_bit = 1usize << (usize::BITS - 1);
    if frame.scause == (interrupt_bit | 5) {
        return crate::runtime::on_timer(frame_ptr);
    }

    match frame.scause {
        13 => {
            let protected_kernel_page = core::ptr::addr_of!(__kernel_start) as usize;
            if frame.stval != protected_kernel_page {
                crate::console_write("U-mode isolation fault address: FAIL\n");
                crate::halt();
            }
            UMODE_ISOLATION_FAULT.store(true, Ordering::Release);
            frame.sepc = match frame.sepc.checked_add(4) {
                Some(next) => next,
                None => crate::halt(),
            };
            frame_ptr
        }
        8 => match frame.x[17] {
            crate::syscall_abi::EXIT => {
                let args_preserved = frame.x[11] == 0
                    && frame.x[12] == 0x22
                    && frame.x[13] == 0x33
                    && frame.x[14] == 0x44
                    && frame.x[15] == 0x55;
                if frame.x[10] != 0 || !args_preserved
                    || !UMODE_ISOLATION_FAULT.load(Ordering::Acquire)
                    || !crate::runtime::timer_preemption_observed()
                    || !crate::runtime::register_context_observed()
                    || !crate::runtime::ipc_roundtrip_observed()
                    || !crate::runtime::ipc_empty_receive_observed() {
                    crate::console_write("U-mode syscall integration: FAIL\n");
                    crate::halt();
                }
                crate::console_write("RV64 callee-saved register + private stack preservation: PASS\n");
                crate::console_write("RV64 IPC empty receive -EAGAIN: PASS\n");
                crate::console_write("RV64 IPC mailbox send/receive round-trip: PASS\n");
                crate::console_write("U-mode Yield syscall context switch + Sv39 isolation: PASS\n");
                crate::runtime::on_user_exit(frame_ptr)
            },
            crate::syscall_abi::SEND_MESSAGE => crate::runtime::on_user_send_message(frame_ptr),
            crate::syscall_abi::RECEIVE_MESSAGE => crate::runtime::on_user_receive_message(frame_ptr),
            crate::syscall_abi::YIELD => crate::runtime::on_user_yield(frame_ptr),
            crate::syscall_abi::MAP_MEMORY => crate::runtime::on_user_map_memory(frame_ptr),
            crate::syscall_abi::UNMAP_MEMORY => crate::runtime::on_user_unmap_memory(frame_ptr),
            crate::syscall_abi::PROTECT_MEMORY => crate::runtime::on_user_protect_memory(frame_ptr),
            5 => {
                if !crate::runtime::user_timer_preemption_observed() {
                    crate::console_write("U-mode timer preemption + saved-frame resume: FAIL\n");
                    crate::halt();
                }
                crate::console_write("U-mode timer preemption + saved-frame resume: PASS\n");
                frame.x[10] = 0;
                frame.sepc = match frame.sepc.checked_add(4) {
                    Some(next) => next,
                    None => crate::halt(),
                };
                frame_ptr
            },
            99 => {
                frame.x[10] = (-38isize) as usize;
                UNKNOWN_SYSCALL_REJECTED.store(true, Ordering::Release);
                frame.sepc = match frame.sepc.checked_add(4) {
                    Some(next) => next,
                    None => crate::halt(),
                };
                frame_ptr
            },
            6 => {
                if !UNKNOWN_SYSCALL_REJECTED.load(Ordering::Acquire) {
                    crate::console_write("U-mode unknown syscall -ENOSYS: FAIL\n");
                    crate::halt();
                }
                crate::console_write("U-mode unknown syscall -ENOSYS: PASS\n");
                frame.x[10] = 0;
                frame.sepc = match frame.sepc.checked_add(4) {
                    Some(next) => next,
                    None => crate::halt(),
                };
                frame_ptr
            },
            _ => {
                frame.x[10] = (-38isize) as usize;
                frame.sepc = match frame.sepc.checked_add(4) {
                    Some(next) => next,
                    None => crate::halt(),
                };
                frame_ptr
            }
        },
        3 => {
            crate::console_write("Unexpected U-mode breakpoint before Exit syscall\n");
            crate::halt()
        }
        _ => {
            crate::console_write("Unexpected U-mode trap; scause=0x");
            crate::print_hex(frame.scause);
            crate::console_write(" sepc=0x");
            crate::print_hex(frame.sepc);
            crate::console_write("\n");
            crate::halt()
        }
    }
}
