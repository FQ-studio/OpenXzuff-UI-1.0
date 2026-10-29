#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(include_str!("entry.S"));

const SBI_LEGACY_CONSOLE_PUTCHAR: usize = 1;

#[inline]
fn console_putchar(byte: u8) {
    // SBI legacy console_putchar: a0 = character, a7 = extension ID 1.
    // Kept intentionally tiny for the first S-mode bring-up milestone.
    unsafe {
        asm!(
            "ecall",
            in("a0") byte as usize,
            in("a7") SBI_LEGACY_CONSOLE_PUTCHAR,
            lateout("a0") _,
            options(nostack)
        );
    }
}

fn console_write(text: &str) {
    for byte in text.bytes() {
        if byte == b'\n' {
            console_putchar(b'\r');
        }
        console_putchar(byte);
    }
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
    console_write("\nFDT parser integration: pending\n");
    console_write("Kernel milestone: S-mode bring-up only\n");

    loop {
        unsafe { asm!("wfi", options(nomem, nostack)); }
    }
}

fn print_hex(mut value: usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut digits = [0u8; core::mem::size_of::<usize>() * 2];
    let mut i = digits.len();
    if value == 0 {
        console_putchar(b'0');
        return;
    }
    while value != 0 {
        i -= 1;
        digits[i] = HEX[value & 0xf];
        value >>= 4;
    }
    for byte in &digits[i..] {
        console_putchar(*byte);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console_write("\nPANIC: Gravity Kernel halted\n");
    loop {
        unsafe { asm!("wfi", options(nomem, nostack)); }
    }
}
