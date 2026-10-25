#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
use core::arch::{asm, global_asm};

mod fdt;
mod memory;
mod syscall;

#[cfg(not(test))]
global_asm!(include_str!("../arch/riscv/entry.S"));
#[cfg(not(test))]
global_asm!(include_str!("../arch/riscv/trap.S"));

#[cfg(not(test))]
extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
    fn gravity_trap_entry();
}

#[cfg(not(test))]
fn console_putc(ch: u8) {
    unsafe { asm!("ecall", inlateout("a0") ch as usize => _, in("a7") 1usize, options(nostack)); }
}

#[cfg(not(test))]
fn console_write(s: &str) { for b in s.bytes() { console_putc(b); } }

#[cfg(not(test))]
fn console_hex(mut n: usize) {
    let mut buf = [0u8; 2 + core::mem::size_of::<usize>() * 2];
    let digits = b"0123456789abcdef";
    buf[0] = b'0'; buf[1] = b'x';
    for i in (2..buf.len()).rev() { buf[i] = digits[n & 0xf]; n >>= 4; }
    for b in buf { console_putc(b); }
}

#[repr(C)]
pub struct TrapFrame { pub x: [usize; 32], pub sepc: usize, pub sstatus: usize }

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn gravity_trap_handler(frame: &mut TrapFrame) {
    let cause: usize;
    unsafe { asm!("csrr {0}, scause", out(reg) cause, options(nomem, nostack)); }
    if cause == 8 {
        let args = [frame.x[10], frame.x[11], frame.x[12], frame.x[13], frame.x[14], frame.x[15]];
        frame.x[10] = syscall::dispatch(frame.x[17], args) as usize;
        frame.sepc = frame.sepc.wrapping_add(4);
        return;
    }
    console_write("\r\nGravity Kernel: fatal trap scause=");
    console_hex(cause);
    console_write(" sepc=");
    console_hex(frame.sepc);
    console_write("\r\n");
    halt();
}

#[cfg(not(test))]
fn halt() -> ! { loop { unsafe { asm!("wfi", options(nomem, nostack)); } } }

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn kernel_main(_hart_id: usize, fdt_blob: *const u8) -> ! {
    use memory::{BootAllocator, MemoryMap};

    console_write("\r\nOpenXzuff UI native kernel: RV64 entry reached.\r\n");
    let mut map = MemoryMap::new();
    let mut allocator = BootAllocator::default();
    let mut allocator_ready = false;

    if fdt_blob.is_null() {
        console_write("Gravity FDT RAM discovery: FAIL; allocator disabled.\r\n");
    } else {
        // Firmware supplies a physical FDT pointer; cap reads to the boot contract
        // and validate the header's total size before trusting any parsed offsets.
        let parsed = unsafe { fdt::parse_memory(core::slice::from_raw_parts(fdt_blob, 16 * 1024 * 1024), &mut map) };
        match parsed {
            Ok(fdt_size) => {
                let kernel_start = core::ptr::addr_of!(__kernel_start) as usize;
                let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
                let fdt_start = fdt_blob as usize;
                let fdt_end = fdt_start.checked_add(fdt_size);
                let reserve_kernel = map.reserve(kernel_start, kernel_end);
                let reserve_fdt = match fdt_end { Some(end) => map.reserve(fdt_start, end), None => Err(()) };
                if reserve_kernel.is_err() || reserve_fdt.is_err() {
                    console_write("Gravity FDT RAM discovery: FAIL; reservation error.\r\n");
                } else if map.count == 0 {
                    console_write("Gravity FDT RAM discovery: FAIL; no usable RAM.\r\n");
                } else {
                    console_write("Gravity FDT RAM discovery: PASS; usable bytes=");
                    console_hex(map.usable_bytes());
                    console_write(".\r\n");
                    let first = map.regions[0];
                    if allocator.init(first.start, first.end).is_ok() {
                        allocator_ready = true;
                        console_write("Gravity boot allocator: initialized from reserved-safe RAM range.\r\n");
                    } else {
                        console_write("Gravity boot allocator: no usable RAM range.\r\n");
                    }
                }
            }
            Err(_) => console_write("Gravity FDT RAM discovery: FAIL; malformed FDT or unsupported layout.\r\n"),
        }
    }

    let trap_addr = gravity_trap_entry as usize;
    unsafe { asm!("csrw stvec, {0}", in(reg) trap_addr, options(nostack)); }
    if syscall::dispatch(0x7fff, [0; 6]) == -(syscall::ENOSYS as isize) {
        console_write("Gravity syscall dispatcher self-test: PASS.\r\n");
    } else {
        console_write("Gravity syscall dispatcher self-test: FAIL.\r\n");
    }
    if allocator_ready {
        console_write("Gravity user-mode trap entry installed; user tasks are not implemented yet.\r\n");
    } else {
        console_write("Gravity user-mode trap entry installed; boot allocator disabled pending memory validation.\r\n");
    }
    halt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trap_frame_layout_matches_assembly_contract() {
        assert_eq!(core::mem::offset_of!(TrapFrame, sepc), 256);
        assert_eq!(core::mem::offset_of!(TrapFrame, sstatus), 264);
        assert_eq!(core::mem::size_of::<TrapFrame>(), 272);
    }
}
