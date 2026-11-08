use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

global_asm!(include_str!("trap.S"));

#[repr(C, align(16))]
struct TrapFrame {
    x: [usize; 32],
    sepc: usize,
    sstatus: usize,
    scause: usize,
    stval: usize,
}

const _: [(); 288] = [(); core::mem::size_of::<TrapFrame>()];
const _: [(); 16] = [(); core::mem::align_of::<TrapFrame>()];

static BREAKPOINT_COUNT: AtomicUsize = AtomicUsize::new(0);
static FRAME_VALID: AtomicBool = AtomicBool::new(false);

extern "C" {
    static __gravity_s_trap_vector: u8;
    fn __gravity_s_trap_roundtrip_test() -> usize;
}

/// Install the direct-mode S-mode trap vector. This early-boot image runs only
/// in S-mode; U-mode entry will need a separate supervisor-stack switch.
pub fn install() {
    let vector = unsafe { core::ptr::addr_of!(__gravity_s_trap_vector) as usize };
    unsafe {
        asm!("csrw stvec, {vector}", vector = in(reg) vector, options(nostack));
    }
}

/// Exercise a delegated breakpoint and verify the assembly entry/return path
/// preserved x1-x31, x0, the interrupted stack pointer, and the Rust ABI.
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
pub extern "C" fn gravity_s_trap_handler(frame_ptr: *mut TrapFrame) {
    let frame = unsafe { &mut *frame_ptr };

    // Exception code 3 is a breakpoint. No interrupts are enabled yet.
    if frame.scause == 3 {
        let expected_stack = (frame_ptr as usize).checked_add(288);
        let valid = frame.x[0] == 0
            && expected_stack == Some(frame.x[2])
            && frame.x[2] & 0x0f == 0
            && frame.sstatus & (1 << 8) != 0; // SPP says the trap came from S-mode.
        FRAME_VALID.store(valid, Ordering::Release);

        match frame.sepc.checked_add(4) {
            Some(next_pc) => frame.sepc = next_pc,
            None => crate::halt(),
        }
        BREAKPOINT_COUNT.fetch_add(1, Ordering::Release);
        return;
    }

    crate::console_write("Unexpected S-mode trap; scause=0x");
    crate::print_hex(frame.scause);
    crate::console_write(" sepc=0x");
    crate::print_hex(frame.sepc);
    crate::console_write(" stval=0x");
    crate::print_hex(frame.stval);
    crate::console_write("\n");
    crate::halt();
}
