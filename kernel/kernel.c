/* OpenXzuff UI native kernel: RV64 early memory discovery and trap setup. */
#include <stddef.h>
#include <stdint.h>
#include "gravity/boot_alloc.h"
#include "gravity/fdt.h"
#include "gravity/memory_map.h"
#include "gravity/syscall.h"

#if defined(__GNUC__)
#define NORETURN __attribute__((noreturn))
#else
#define NORETURN
#endif

#if defined(__riscv)
extern void gravity_trap_entry(void);
extern char __kernel_start[];
extern char __kernel_end[];

static void early_console_putc(char ch)
{
    register long a0 __asm__("a0") = (long)ch;
    register long a7 __asm__("a7") = 1;
    __asm__ volatile ("ecall" : "+r"(a0) : "r"(a7) : "memory");
}

static void early_console_write(const char *s)
{
    while (*s) early_console_putc(*s++);
}

static void early_console_hex(uint64_t value)
{
    static const char digits[] = "0123456789abcdef";
    int shift;
    early_console_write("0x");
    for (shift = 60; shift >= 0; shift -= 4)
        early_console_putc(digits[(value >> shift) & 0xfu]);
}

static void install_supervisor_trap_vector(void)
{
    uintptr_t address = (uintptr_t)gravity_trap_entry;
    __asm__ volatile ("csrw stvec, %0" :: "r"(address) : "memory");
}

static int syscall_dispatch_self_test(void)
{
    gravity_syscall_args_t args = {0, 0, 0, 0, 0, 0};
    intptr_t result = gravity_syscall_dispatch(0x7fff, &args);
    return result == -(intptr_t)GRAVITY_ENOSYS;
}
#endif

/*
 * SBI/OpenSBI RISC-V boot convention: a0=hart ID, a1=physical FDT address.
 * This bring-up build assumes the firmware supplies a valid FDT pointer.
 */
NORETURN void kernel_main(uintptr_t hart_id, const void *fdt_blob)
{
#if defined(__riscv)
    gravity_memory_map_t memory_map;
    gravity_boot_allocator_t boot_allocator;
    size_t fdt_size = 0;
    size_t usable_bytes;
    int allocator_ready = 0;

    (void)hart_id;
    early_console_write("\r\nOpenXzuff UI native kernel: RV64 entry reached.\r\n");
    gravity_memory_map_init(&memory_map);

    if (gravity_fdt_add_memory(fdt_blob, 16u * 1024u * 1024u,
                               &memory_map, &fdt_size) == 0) {
        /* The linker bounds include .bss and the reserved bootstrap stack. */
        if (gravity_memory_map_reserve(&memory_map,
                (uintptr_t)__kernel_start, (uintptr_t)__kernel_end) != 0) {
            early_console_write("Gravity memory map: failed to reserve kernel image.\r\n");
        } else if (fdt_size != 0 &&
                   gravity_memory_map_reserve(&memory_map,
                       (uintptr_t)fdt_blob,
                       (uintptr_t)fdt_blob + fdt_size) != 0) {
            early_console_write("Gravity memory map: failed to reserve FDT.\r\n");
        } else {
            usable_bytes = gravity_memory_map_usable_bytes(&memory_map);
            early_console_write("Gravity FDT RAM discovery: PASS; usable bytes=");
            early_console_hex((uint64_t)usable_bytes);
            early_console_write(".\r\n");
            if (memory_map.count != 0 &&
                gravity_boot_alloc_init(&boot_allocator,
                    memory_map.regions[0].start,
                    memory_map.regions[0].end) == 0) {
                allocator_ready = 1;
                early_console_write("Gravity boot allocator: initialized from reserved-safe RAM range.\r\n");
            } else {
                early_console_write("Gravity boot allocator: no usable RAM range.\r\n");
            }
        }
    } else {
        early_console_write("Gravity FDT RAM discovery: FAIL; allocator disabled.\r\n");
    }

    install_supervisor_trap_vector();
    if (syscall_dispatch_self_test())
        early_console_write("Gravity syscall dispatcher self-test: PASS.\r\n");
    else
        early_console_write("Gravity syscall dispatcher self-test: FAIL.\r\n");
    early_console_write("Gravity user-mode trap entry installed; user tasks are not implemented yet.\r\n");
    (void)allocator_ready;
#elif defined(__aarch64__)
    (void)hart_id;
    (void)fdt_blob;
#endif

    for (;;) {
#if defined(__riscv) || defined(__aarch64__)
        __asm__ volatile ("wfi");
#else
        __asm__ volatile ("" ::: "memory");
#endif
    }
}
