/* OpenXzuff UI native kernel: architecture-independent entry point. */
#include <stdint.h>

#if defined(__GNUC__)
#define NORETURN __attribute__((noreturn))
#else
#define NORETURN
#endif

/* The architecture entry stub calls this after establishing a valid stack. */
NORETURN void kernel_main(void)
{
    /* Deliberately minimal until console, memory, and platform APIs exist. */
    for (;;) {
#if defined(__riscv)
        __asm__ volatile ("wfi");
#elif defined(__aarch64__)
        __asm__ volatile ("wfi");
#else
        __asm__ volatile ("" ::: "memory");
#endif
    }
}
