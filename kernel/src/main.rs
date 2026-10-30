#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

mod fdt;
mod memory;

global_asm!(include_str!("entry.S"));

const SBI_LEGACY_CONSOLE_PUTCHAR: usize = 1;
const FDT_MAGIC: u32 = 0xd00d_feed;
const MAX_FDT_SIZE: usize = 2 * 1024 * 1024;

#[inline]
fn console_putchar(byte: u8) {
    unsafe {
        asm!(
            "ecall",
            in("a0") byte as usize,
            in("a7") SBI_LEGACY_CONSOLE_PUTCHAR,
            options(nostack)
        );
    }
}

fn console_write(text: &str) {
    for byte in text.bytes() {
        if byte == b'\n' { console_putchar(b'\r'); }
        console_putchar(byte);
    }
}

fn print_hex(mut value: usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut digits = [0u8; core::mem::size_of::<usize>() * 2];
    let mut i = digits.len();
    if value == 0 { console_putchar(b'0'); return; }
    while value != 0 {
        i -= 1;
        digits[i] = HEX[value & 0xf];
        value >>= 4;
    }
    for byte in &digits[i..] { console_putchar(*byte); }
}

fn be32_raw(ptr: *const u8) -> u32 {
    unsafe { u32::from_be_bytes([ptr.read(), ptr.add(1).read(), ptr.add(2).read(), ptr.add(3).read()]) }
}

#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, fdt_addr: usize) -> ! {
    console_write("\nOpenXzuff UI 1.0 / Gravity Kernel\n");
    console_write("RV64 S-mode entry: OK\n");
    console_write("SBI legacy console: OK\n");
    console_write("Boot hart: ");
    print_hex(hart_id);
    console_write("\nFDT pointer: ");
    print_hex(fdt_addr);
    console_write("\n");

    if fdt_addr == 0 || fdt_addr & 7 != 0 {
        console_write("FDT parser: FAIL (null or unaligned pointer)\n");
        halt();
    }

    // Firmware supplies the pointer. Validate the fixed header before creating
    // a slice, then cap the blob size to avoid an unbounded early-boot read.
    let header = unsafe { core::slice::from_raw_parts(fdt_addr as *const u8, 8) };
    if be32_raw(header.as_ptr()) != FDT_MAGIC {
        console_write("FDT parser: FAIL (bad magic)\n");
        halt();
    }
    let total = be32_raw(unsafe { header.as_ptr().add(4) }) as usize;
    if total < 40 || total > MAX_FDT_SIZE {
        console_write("FDT parser: FAIL (invalid blob size)\n");
        halt();
    }
    let blob = unsafe { core::slice::from_raw_parts(fdt_addr as *const u8, total) };
    let mut map = memory::MemoryMap::new();
    match fdt::parse_memory(blob, &mut map) {
        Ok(parsed_size) => {
            // Protect the linked kernel image and FDT blob from later allocators.
            let kernel_start = unsafe { core::ptr::addr_of!(__kernel_start) as usize };
            let kernel_end = unsafe { core::ptr::addr_of!(__kernel_end) as usize };
            let fdt_end = fdt_addr.checked_add(parsed_size);
            if kernel_start < kernel_end
                && map.reserve(kernel_start, kernel_end).is_ok()
                && fdt_end.is_some()
                && map.reserve(fdt_addr, fdt_end.unwrap()).is_ok()
            {
                console_write("FDT parser: OK\nRAM regions: ");
                print_hex(map.ram_count());
                console_write("\nFDT blob bytes: ");
                print_hex(parsed_size);
                console_write("\nFDT memory map: parsed; allocator integration pending\n");
            } else {
                console_write("FDT parser: FAIL (could not reserve kernel/FDT memory)\n");
                halt();
            }
        }
        Err(_) => {
            console_write("FDT parser: FAIL (invalid structure or memory map)\n");
            halt();
        }
    }

    console_write("Kernel milestone: FDT memory discovery integrated; allocator pending\n");
    halt()
}

extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

fn halt() -> ! {
    loop { unsafe { asm!("wfi", options(nomem, nostack)); } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console_write("\nPANIC: Gravity Kernel halted\n");
    halt()
}
