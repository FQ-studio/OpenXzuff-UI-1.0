#include <stdint.h>
#include "gravity/syscall.h"

typedef struct gravity_trap_frame {
    uintptr_t x[32];
    uintptr_t sepc;
    uintptr_t sstatus;
} gravity_trap_frame_t;

static void sbi_console_putc(char ch)
{
    register long a0 __asm__("a0") = (long)ch;
    register long a7 __asm__("a7") = 1;
    __asm__ volatile ("ecall" : "+r"(a0) : "r"(a7) : "memory");
}

static void console_write(const char *s)
{
    while (*s) sbi_console_putc(*s++);
}

static void console_hex(uintptr_t value)
{
    static const char digits[] = "0123456789abcdef";
    for (int shift = (int)(sizeof(uintptr_t) * 8) - 4; shift >= 0; shift -= 4)
        sbi_console_putc(digits[(value >> shift) & 0xf]);
}

__attribute__((noreturn))
static void trap_halt(void)
{
    for (;;) __asm__ volatile ("wfi");
}

/* Called only by gravity_trap_entry; frame layout is shared with trap.S. */
void gravity_trap_handler(gravity_trap_frame_t *frame)
{
    uintptr_t cause;
    __asm__ volatile ("csrr %0, scause" : "=r"(cause));

    /* Exception code 9: environment call from Supervisor mode. */
    if (cause == 9) {
        gravity_syscall_args_t args = {
            frame->x[10], frame->x[11], frame->x[12],
            frame->x[13], frame->x[14], frame->x[15]
        };
        frame->x[10] = (uintptr_t)gravity_syscall_dispatch(frame->x[17], &args);
        frame->sepc += 4; /* ECALL is a 32-bit instruction. */
        return;
    }

    console_write("\r\nGravity Kernel: fatal supervisor trap scause=0x");
    console_hex(cause);
    console_write(" sepc=0x");
    console_hex(frame->sepc);
    console_write("\r\n");
    trap_halt();
}
