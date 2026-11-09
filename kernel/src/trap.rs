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

#[repr(align(4096))]
struct UModeStack([u8; 4096]);
static mut UMODE_STACK: UModeStack = UModeStack([0; 4096]);
#[repr(align(4096))]
struct SupervisorTrapStack([u8; 4096]);
static mut SUPERVISOR_TRAP_STACK: SupervisorTrapStack = SupervisorTrapStack([0; 4096]);

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
pub(crate) unsafe fn create_user_task_frame() -> *mut TrapFrame {
    let user_base = core::ptr::addr_of_mut!(UMODE_STACK.0).cast::<u8>();
    let user_top = user_base.add(4096) as usize;
    let supervisor_base = core::ptr::addr_of_mut!(SUPERVISOR_TRAP_STACK.0).cast::<u8>();
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

#[no_mangle]
pub extern "C" fn gravity_u_trap_handler(frame_ptr: *mut TrapFrame) -> *mut TrapFrame {
    let frame = unsafe { &mut *frame_ptr };
    let from_user = frame.sstatus & (1 << 8) == 0;
    if !from_user {
        crate::console_write("U-mode trap gate: rejected S-origin frame\n");
        crate::halt();
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
            0 => crate::runtime::on_user_exit(frame_ptr),
            1 => {
                // ABI v1 SendMessage is reserved until the IPC transport exists.
                if frame.x[10] != 0x4752_4156 {
                    crate::console_write("U-mode SendMessage arguments: FAIL\n");
                    crate::halt();
                }
                frame.x[10] = (-38isize) as usize;
                frame.sepc = match frame.sepc.checked_add(4) {
                    Some(next) => next,
                    None => crate::halt(),
                };
                frame_ptr
            }
            4 => crate::runtime::on_user_yield(frame_ptr),
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
            let args_preserved = frame.x[11] == 0x11
                && frame.x[12] == 0x22
                && frame.x[13] == 0x33
                && frame.x[14] == 0x44
                && frame.x[15] == 0x55;
            if frame.x[10] == 0 && args_preserved
                && UMODE_ISOLATION_FAULT.load(Ordering::Acquire)
                && crate::runtime::timer_preemption_observed()
                && crate::runtime::register_context_observed() {
                crate::console_write("RV64 callee-saved register + private stack preservation: PASS\n");
                crate::console_write("U-mode SendMessage -ENOSYS + Yield syscall context switch + Sv39 isolation: PASS\n");
                frame.sepc = match frame.sepc.checked_add(4) {
                    Some(next) => next,
                    None => crate::halt(),
                };
                return frame_ptr;
            }
            crate::console_write("U-mode syscall return ABI: FAIL\n");
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
