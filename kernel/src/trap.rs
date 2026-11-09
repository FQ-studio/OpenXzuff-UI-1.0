use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

global_asm!(include_str!("trap.S"));

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
            // Return to S-mode with interrupts enabled after sret.
            sstatus: (1 << 8) | (1 << 5),
            scause: 0,
            stval: 0,
        };
        frame.x[2] = stack_top;
        frame.x[3] = gp;
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

extern "C" {
    static __gravity_s_trap_vector: u8;
    fn __gravity_s_trap_roundtrip_test() -> usize;
}

pub fn install() {
    let vector = core::ptr::addr_of!(__gravity_s_trap_vector) as usize;
    unsafe { asm!("csrw stvec, {vector}", vector = in(reg) vector, options(nostack)); }
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
