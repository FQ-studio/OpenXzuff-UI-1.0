#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

mod fdt;
mod mmio;
mod mmio_map;
mod plic;
mod irq;
mod uart16550;
mod memory;
mod page_alloc;
mod heap_alloc;
mod trap;
mod scheduler;
mod process;
mod task_manager;
mod handle;
mod vfs;
mod ipc;
mod runtime;
mod vm;
mod vm_policy;
mod syscall_abi;

pub(crate) static mut PAGE_ALLOCATOR: page_alloc::PageAllocator = page_alloc::PageAllocator::new();
static mut HEAP_ALLOCATOR: heap_alloc::HeapAllocator = heap_alloc::HeapAllocator::new();
#[repr(align(16))]
struct HeapArena([u8; 64 * 1024]);
static mut HEAP_ARENA: HeapArena = HeapArena([0; 64 * 1024]);

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

pub(crate) fn console_write(text: &str) {
    for byte in text.bytes() {
        if byte == b'\n' { console_putchar(b'\r'); }
        console_putchar(byte);
    }
}

pub(crate) fn print_hex(mut value: usize) {
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

/// Discover a known UART node and validate its translated register window.
fn report_uart_candidate(blob: &[u8]) -> Option<(mmio::MmioRegion, bool)> {
    let compatibles: [&[u8]; 5] = [
        b"sifive,fu740-c000-uart",
        b"sifive,uart0",
        b"starfive,jh7110-uart",
        b"snps,dw-apb-uart",
        b"ns16550a",
    ];
    for compatible in compatibles {
        match fdt::find_compatible_node_translated(blob, compatible) {
            Ok(Some(node)) => {
                if let (Some(bus), Some(root)) = (node.bus_reg, node.root_reg) {
                    if root.size.is_some() {
                        let region = match mmio::MmioRegion::from_translated_fdt_region(root, 4, 4) {
                            Ok(region) => region,
                            Err(_) => {
                                console_write("FDT UART MMIO window: REJECTED; not mapped\n");
                                return None;
                            }
                        };
                        if region.register_address(0, 4).is_err() {
                            console_write("FDT UART MMIO window: REJECTED; not mapped\n");
                            return None;
                        }
                        console_write("FDT UART compatible-node discovery: PASS; ancestor ranges translated\n");
                        console_write("FDT UART bus address: ");
                        print_hex(bus.bus_address as usize);
                        console_write("; root address: ");
                        print_hex(region.base() as usize);
                        console_write("; size: ");
                        print_hex(region.size() as usize);
                        console_write("\n");
                        console_write("FDT UART MMIO window: PASS; 32-bit base register in bounds; pending Sv39 mapping\n");
                        return Some((region, compatible == &b"ns16550a"[..]));
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                let code = match error {
                    fdt::FdtError::Header => 1,
                    fdt::FdtError::Bounds => 2,
                    fdt::FdtError::Structure => 3,
                    fdt::FdtError::Cells => 4,
                    fdt::FdtError::Memory => 5,
                    fdt::FdtError::Reservation => 6,
                    fdt::FdtError::Timebase => 7,
                    fdt::FdtError::Translation => 8,
                    fdt::FdtError::Interrupts => 9,
                };
                console_write("FDT UART compatible-node discovery: ERROR code=");
                print_hex(code);
                console_write("\n");
                return None;
            }
        }
    }
    console_write("FDT UART compatible-node discovery: NOT FOUND\n");
    None
}

/// Discover and validate the PLIC register layout without touching MMIO.
fn report_plic_candidate(blob: &[u8], boot_hart_id: usize) -> Option<(mmio::MmioRegion, plic::PlicLayout, Option<fdt::FdtPlicContext>)> {
    match fdt::find_plic_info_with_stage(blob) {
        Ok(Some(info)) => {
            let region = match mmio::MmioRegion::from_translated_fdt_region(info.root_reg, 8, 4) {
                Ok(region) => region,
                Err(_) => {
                    console_write("FDT PLIC MMIO window: REJECTED; not mapped\n");
                    return None;
                }
            };
            let layout = match plic::PlicLayout::new(region, info.source_count, info.context_count) {
                Ok(layout) => layout,
                Err(_) => {
                    console_write("FDT PLIC register layout: REJECTED; not mapped\n");
                    return None;
                }
            };
            let last_context = info.context_count - 1;
            let endpoints_valid = layout.priority_address(1).is_ok()
                && layout.priority_address(info.source_count).is_ok()
                && layout.pending_bit(info.source_count).is_ok()
                && layout.enable_bit(last_context, info.source_count).is_ok()
                && layout.threshold_address(last_context).is_ok()
                && layout.claim_complete_address(last_context).is_ok();
            if !endpoints_valid {
                console_write("FDT PLIC register layout: REJECTED; not mapped\n");
                return None;
            }
            let selected_context = match fdt::find_plic_context_for_hart(blob, boot_hart_id as u64) {
                Ok(context) => context,
                Err(_) => {
                    console_write("FDT PLIC S-mode context: REJECTED; delivery disabled\n");
                    return None;
                }
            };
            if let Some(context) = selected_context {
                if context.context_index >= info.context_count
                    || layout.threshold_address(context.context_index).is_err()
                    || layout.claim_complete_address(context.context_index).is_err()
                {
                    console_write("FDT PLIC S-mode context: REJECTED; delivery disabled\n");
                    return None;
                }
            }
            console_write("FDT PLIC discovery: PASS; ndev=");
            print_hex(info.source_count as usize);
            console_write("; contexts=");
            print_hex(info.context_count as usize);
            console_write("; root=");
            print_hex(region.base() as usize);
            console_write("; size=");
            print_hex(region.size() as usize);
            console_write("\n");
            console_write("FDT PLIC register layout: PASS; priority/pending/enable/threshold/claim in bounds; pending Sv39 mapping\n");
            match selected_context {
                Some(context) => {
                    console_write("FDT PLIC S-mode context: PASS; hart=");
                    print_hex(context.hart_id as usize);
                    console_write("; context=");
                    print_hex(context.context_index as usize);
                    console_write("; cause=");
                    print_hex(context.cause as usize);
                    console_write("\n");
                }
                None => console_write("FDT PLIC S-mode context: NOT FOUND; delivery disabled\n"),
            }
            Some((region, layout, selected_context))
        }
        Ok(None) => {
            console_write("FDT PLIC discovery: NOT FOUND\n");
            None
        }
        Err((stage, error, detail)) => {
            console_write("FDT PLIC discovery: REJECTED; stage=");
            print_hex(stage as usize);
            console_write("; error=");
            print_hex(error as usize);
            console_write("; detail=");
            print_hex(detail);
            console_write("\n");
            None
        }
    }
}

fn be32_raw(ptr: *const u8) -> u32 {
    unsafe { u32::from_be_bytes([ptr.read(), ptr.add(1).read(), ptr.add(2).read(), ptr.add(3).read()]) }
}

#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, fdt_addr: usize) -> ! {
    trap::install();
    console_write("S-mode trap vector: installed\n");
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
    let uart_candidate = report_uart_candidate(blob);
    let uart_region = uart_candidate.map(|(region, _)| region);
    let ns16550_region = uart_candidate.and_then(|(region, is_ns16550)| {
        if is_ns16550 { Some(region) } else { None }
    });
    let plic_candidate = report_plic_candidate(blob, hart_id);
    let plic_region = plic_candidate.map(|(region, _, _)| region);
    let timebase_hz = match fdt::timebase_frequency(blob) {
        Ok(value) => value,
        Err(_) => { console_write("FDT timebase-frequency: FAIL\n"); halt(); }
    };
    let timer_quantum = match runtime::configure_timebase(timebase_hz) {
        Ok(value) => value,
        Err(_) => { console_write("Timer configuration: FAIL (invalid timebase)\n"); halt(); }
    };
    console_write("FDT timebase-frequency: parsed; 10 ms quantum ticks=");
    print_hex(timer_quantum);
    console_write("\n");

    let kernel_start = core::ptr::addr_of!(__kernel_start) as usize;
    let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
    let text_start = core::ptr::addr_of!(__text_start) as usize;
    let text_end = core::ptr::addr_of!(__text_end) as usize;
    let rodata_start = core::ptr::addr_of!(__rodata_start) as usize;
    let rodata_end = core::ptr::addr_of!(__rodata_end) as usize;
    let data_start = core::ptr::addr_of!(__data_start) as usize;
    let data_end = core::ptr::addr_of!(__data_end) as usize;
    let bss_start = core::ptr::addr_of!(__bss_start) as usize;
    let bss_end = core::ptr::addr_of!(__bss_end) as usize;
    let user_code_start = core::ptr::addr_of!(__umode_user_start) as usize;
    let user_code_end = core::ptr::addr_of!(__umode_user_end) as usize;
    let (user_stack_start, user_stack_end) = trap::user_stack_bounds();
    let fdt_end = match fdt_addr.checked_add(total) {
        Some(end) => end,
        None => { console_write("Sv39 page tables: FAIL (FDT range overflow)\n"); halt(); }
    };
    let supervisor_stack_starts = trap::supervisor_trap_stack_starts();
    if !trap::supervisor_trap_stacks_layout_valid() {
        console_write("Per-task supervisor trap stacks: FAIL (layout)\n");
        halt();
    }
    match unsafe { vm::enable_bootstrap(
        kernel_start, kernel_end, fdt_addr, fdt_end,
        user_code_start, user_code_end, user_stack_start, user_stack_end,
        text_start, text_end, rodata_start, rodata_end,
        data_start, data_end, bss_start, bss_end,
    ) } {
        Ok(()) if vm::active() && vm::roots_are_distinct()
            && supervisor_stack_starts.iter().all(|&stack_start| {
                let stack_end = match stack_start.checked_add(4096) {
                    Some(end) => end,
                    None => return false,
                };
                unsafe {
                    vm::verify_isolation(
                        user_code_start, user_code_end,
                        user_stack_start, user_stack_end,
                        kernel_start, stack_start, stack_end,
                        text_start, text_end, rodata_start, rodata_end,
                        data_start, data_end, bss_start, bss_end,
                    )
                }
            }) => {
                for (region, label) in [(uart_region, "UART"), (plic_region, "PLIC")] {
                    if let Some(region) = region {
                        match vm::map_kernel_mmio_region(region) {
                            Ok(()) if vm::verify_kernel_mmio_region(region) => {
                                console_write("FDT ");
                                console_write(label);
                                console_write(" MMIO mapping: PASS; identity; supervisor-only; non-executable; shared kernel/user roots\n");
                            }
                            _ => {
                                console_write("FDT ");
                                console_write(label);
                                console_write(" MMIO mapping: FAIL\n");
                                halt();
                            }
                        }
                    }
                }
                if let Some(region) = ns16550_region {
                    let mut uart = match unsafe { uart16550::Uart16550::new(region) } {
                        Ok(uart) => uart,
                        Err(_) => {
                            console_write("UART 16550 loopback: FAIL; invalid mapped register window\n");
                            halt();
                        }
                    };
                    match uart.loopback_probe(0x5a) {
                        Ok(received) => {
                            console_write("UART 16550 loopback: PASS; tx=5a; rx=");
                            print_hex(received as usize);
                            console_write("\n");
                        }
                        Err(_) => {
                            console_write("UART 16550 loopback: FAIL; register round-trip\n");
                            halt();
                        }
                    }
                }
                if let Some((_, layout, Some(context))) = plic_candidate {
                    let controller = match unsafe {
                        plic::PlicMmio::new(layout, context.context_index)
                    } {
                        Ok(controller) => controller,
                        Err(_) => {
                            console_write("PLIC MMIO access: FAIL; invalid mapped context\n");
                            halt();
                        }
                    };
                    let readback = controller.priority(1).and_then(|priority| {
                        controller.set_priority(1, priority)?;
                        let priority_after = controller.priority(1)?;
                        let threshold = controller.threshold()?;
                        controller.set_threshold(threshold)?;
                        let threshold_after = controller.threshold()?;
                        if priority != priority_after || threshold != threshold_after {
                            return Err(plic::PlicError::ReadbackMismatch);
                        }
                        Ok((priority_after, threshold_after))
                    });
                    match readback {
                        Ok((priority, threshold)) => {
                            console_write("PLIC MMIO access: PASS; context=");
                            print_hex(context.context_index as usize);
                            console_write("; priority1=");
                            print_hex(priority as usize);
                            console_write("; threshold=");
                            print_hex(threshold as usize);
                            console_write("\n");
                        }
                        Err(_) => {
                            console_write("PLIC MMIO access: FAIL; register readback mismatch\n");
                            halt();
                        }
                    }
                }
                if let Some((_, layout, Some(context))) = plic_candidate {
                    match unsafe { irq::install(layout, context.context_index) } {
                        Ok(()) => {
                            console_write("PLIC external interrupt dispatcher: PASS; context=");
                            print_hex(context.context_index as usize);
                            console_write("; threshold=0; slots=1024\n");
                        }
                        Err(_) => {
                            console_write("PLIC external interrupt dispatcher: FAIL\n");
                            halt();
                        }
                    }
                }
                console_write("Sv39 bootstrap page tables: PASS; U-mode mappings isolated\n");
                console_write("Sv39 kernel/user address spaces: PASS; distinct roots and U-only probe mappings\n");
                console_write("Sv39 kernel W^X section permissions: PASS; text=RX rodata=R data=RW\n");
                console_write("Per-task supervisor trap stacks: PASS; slots=16; supervisor-only\n");
            }
        _ => { console_write("Sv39 kernel/user address spaces: FAIL\n"); halt(); }
    }
    let blob = unsafe { core::slice::from_raw_parts(fdt_addr as *const u8, total) };
    let mut map = memory::MemoryMap::new();
    match fdt::parse_memory(blob, &mut map) {
        Ok(parsed_size) => {
            // Protect the linked kernel image and FDT blob from later allocators.
            let kernel_start = core::ptr::addr_of!(__kernel_start) as usize;
            let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
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
                console_write("\nFDT memory map: parsed\n");
                let allocator = core::ptr::addr_of_mut!(PAGE_ALLOCATOR);
                let page_count = unsafe { (*allocator).init(&map) };
                match page_count {
                    Ok(count) if count > 0 => {
                        let managed = unsafe { (*allocator).managed_pages() };
                        let available = unsafe { (*allocator).available_pages() };
                        if managed != count || available != count {
                            console_write("Physical page allocator: FAIL (initial accounting)\n");
                            halt();
                        }
                        console_write("Physical page allocator: OK; managed pages: ");
                        print_hex(count);
                        console_write("\n");
                        match unsafe { (*allocator).alloc() } {
                            Some(page) => {
                                console_write("PPA alloc/free smoke test: ");
                                let freed = unsafe { (*allocator).free(page) }.is_ok();
                                let restored = unsafe { (*allocator).available_pages() } == count;
                                if freed && restored {
                                    console_write("PASS\n");
                                } else {
                                    console_write("FAIL (free rejected)\n");
                                    halt();
                                }
                            }
                            None => {
                                console_write("PPA alloc/free smoke test: FAIL (empty)\n");
                                halt();
                            }
                        }
                        // Use a dedicated static arena: do not cast physical page addresses to pointers.
                        let heap = core::ptr::addr_of_mut!(HEAP_ALLOCATOR);
                        let arena = unsafe { core::ptr::addr_of_mut!(HEAP_ARENA.0).cast::<u8>() };
                        if unsafe { (*heap).init(arena, 64 * 1024) }.is_err() {
                            console_write("Kernel heap allocator: FAIL (init)\n");
                            halt();
                        }
                        console_write("Kernel heap allocator: OK; arena bytes: 65536\n");
                        match unsafe { (*heap).alloc(256) } {
                            Some(ptr) => {
                                let passed = unsafe { (*heap).free(ptr) }.is_ok()
                                    && unsafe { (*heap).available_bytes() } == 64 * 1024;
                                if passed {
                                    console_write("Heap alloc/free smoke test: PASS\n");
                                } else {
                                    console_write("Heap alloc/free smoke test: FAIL\n");
                                    halt();
                                }
                            }
                            None => {
                                console_write("Heap alloc/free smoke test: FAIL (empty)\n");
                                halt();
                            }
                        }
                    }
                    _ => {
                        console_write("Physical page allocator: FAIL\n");
                        halt();
                    }
                }
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

    trap::smoke_test();
    if scheduler::smoke_test() {
        console_write("Cooperative scheduler state test: PASS\n");
    } else {
        console_write("Cooperative scheduler state test: FAIL\n");
        halt();
    }
    console_write("Kernel milestone: launching real timer-preemptive task test\n");
    runtime::start()
}

extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
    static __umode_user_start: u8;
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
    static __data_start: u8;
    static __data_end: u8;
    static __bss_start: u8;
    static __bss_end: u8;
    static __umode_user_end: u8;
}

pub(crate) fn halt() -> ! {
    loop { unsafe { asm!("wfi", options(nomem, nostack)); } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console_write("\nPANIC: Gravity Kernel halted\n");
    halt()
}
