/* OpenXzuff UI native kernel: architecture-independent entry point. */
#include <stdint.h>
#include "gravity/syscall.h"

#if defined(__GNUC__)
#define NORETURN __attribute__((noreturn))
#else
#define NORETURN
#endif

#if defined(__riscv)
extern void gravity_trap_entry(void);

/* SBI legacy console_putchar (extension ID 0x1), for early bring-up only. */
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

static void install_supervisor_trap_vector(void)
{
    uintptr_t address = (uintptr_t)gravity_trap_entry;
    __asm__ volatile ("csrw stvec, %0" :: "r"(address) : "memory");
}

/* Test the dispatcher without pretending a user process already exists. */
static int syscall_dispatch_self_test(void)
{
    gravity_syscall_args_t args = {0, 0, 0, 0, 0, 0};
    intptr_t result = gravity_syscall_dispatch(0x7fff, &args);
    return result == -(intptr_t)GRAVITY_ENOSYS;
}
#endif

/* The architecture entry stub calls this after establishing a valid stack. */
NORETURN void kernel_main(void)
{
#if defined(__riscv)
    early_console_write("\r\nOpenXzuff UI native kernel: RV64 entry reached.\r\n");
    install_supervisor_trap_vector();
    if (syscall_dispatch_self_test()) {
        early_console_write("Gravity syscall dispatcher self-test: PASS.\r\n");
    } else {
        early_console_write("Gravity syscall dispatcher self-test: FAIL.\r\n");
    }
    early_console_write("Gravity user-mode trap entry installed; user tasks are not implemented yet.\r\n");
#elif defined(__aarch64__)
    /* No console is assumed until the UEFI/GOP or serial adapter is implemented. */
#endif

    for (;;) {
#if defined(__riscv) || defined(__aarch64__)
        __asm__ volatile ("wfi");
#else
        __asm__ volatile ("" ::: "memory");
#endif
    }
}
