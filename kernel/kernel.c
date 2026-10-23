/* OpenXzuff UI native kernel: architecture-independent entry point. */
#include <stdint.h>

#if defined(__GNUC__)
#define NORETURN __attribute__((noreturn))
#else
#define NORETURN
#endif

#if defined(__riscv)
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
#endif

/* The architecture entry stub calls this after establishing a valid stack. */
NORETURN void kernel_main(void)
{
#if defined(__riscv)
    early_console_write("\r\nOpenXzuff UI native kernel: RV64 entry reached.\r\n");
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
