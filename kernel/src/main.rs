#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
use core::arch::{asm, global_asm};

mod app;
#[cfg(not(test))]
mod context;
mod fdt;
mod memory;
mod syscall;
mod task;

#[cfg(not(test))]
global_asm!(include_str!("../arch/riscv/entry.S"));
#[cfg(not(test))]
global_asm!(include_str!("../arch/riscv/trap.S"));
#[cfg(not(test))]
global_asm!(include_str!("../arch/riscv/context.S"));

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
        match syscall::dispatch(frame.x[17], args) {
            syscall::SyscallOutcome::Return(value) => {
                frame.x[10] = value as usize;
                // ECALL is a 32-bit instruction in this RV64 ABI.
                frame.sepc = frame.sepc.wrapping_add(4);
                return;
            }
            syscall::SyscallOutcome::ExitRequested { status } => {
                // Safe fallback until the process manager/scheduler exists.
                // Do not return to the task that explicitly requested exit.
                console_write("\\r\\nGravity TaskExit requested; no scheduler is available yet, halting. status=");
                console_hex(status as usize);
                console_write("\\r\\n");
                halt();
            }
        }
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
pub unsafe extern "C" fn memset(dst: *mut core::ffi::c_void, value: i32, count: usize) -> *mut core::ffi::c_void {
    let out = dst.cast::<u8>();
    let mut i = 0;
    while i < count { core::ptr::write_volatile(out.add(i), value as u8); i += 1; }
    dst
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, count: usize) -> *mut core::ffi::c_void {
    let out = dst.cast::<u8>();
    let input = src.cast::<u8>();
    let mut i = 0;
    while i < count { core::ptr::write_volatile(out.add(i), core::ptr::read_volatile(input.add(i))); i += 1; }
    dst
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, count: usize) -> *mut core::ffi::c_void {
    let out = dst.cast::<u8>();
    let input = src.cast::<u8>();
    if (out as usize) <= (input as usize) {
        let mut i = 0;
        while i < count { core::ptr::write_volatile(out.add(i), core::ptr::read_volatile(input.add(i))); i += 1; }
    } else {
        let mut i = count;
        while i > 0 { i -= 1; core::ptr::write_volatile(out.add(i), core::ptr::read_volatile(input.add(i))); }
    }
    dst
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const core::ffi::c_void, b: *const core::ffi::c_void, count: usize) -> i32 {
    let left = a.cast::<u8>();
    let right = b.cast::<u8>();
    let mut i = 0;
    while i < count {
        let av = core::ptr::read_volatile(left.add(i));
        let bv = core::ptr::read_volatile(right.add(i));
        if av != bv { return av as i32 - bv as i32; }
        i += 1;
    }
    0
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { halt() }

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
        let header = unsafe { core::slice::from_raw_parts(fdt_blob, 40) };
        let parsed = fdt::declared_size(header, 16 * 1024 * 1024)
            .and_then(|size| unsafe { fdt::parse_memory(core::slice::from_raw_parts(fdt_blob, size), &mut map) });
        match parsed {
            Ok(fdt_size) => {
                let kernel_start = unsafe { core::ptr::addr_of!(__kernel_start) as usize };
                let kernel_end = unsafe { core::ptr::addr_of!(__kernel_end) as usize };
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

    let trap_addr = gravity_trap_entry as *const () as usize;
    unsafe { asm!("csrw stvec, {0}", in(reg) trap_addr, options(nostack)); }
    if syscall::dispatch(0x7fff, [0; 6])
        == syscall::SyscallOutcome::Return(-syscall::ENOSYS)
        && syscall::dispatch(syscall::Syscall::TaskExit as usize, [0; 6])
            == (syscall::SyscallOutcome::ExitRequested { status: 0 }) {
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
