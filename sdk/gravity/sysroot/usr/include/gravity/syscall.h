#ifndef GRAVITY_SYSCALL_H
#define GRAVITY_SYSCALL_H

#include <gravity/abi.h>

#if !defined(__riscv) || !defined(__riscv_xlen) || __riscv_xlen != 64
#error "Gravity syscall wrappers require an RV64 target compiler"
#endif

#ifdef __cplusplus
extern "C" {
#endif

static inline int64_t gravity_syscall0(uint64_t number) {
    register uint64_t a0 __asm__("a0") = 0;
    register uint64_t a7 __asm__("a7") = number;
    __asm__ volatile("ecall" : "+r"(a0) : "r"(a7) : "memory");
    return (int64_t)a0;
}

static inline int64_t gravity_syscall1(uint64_t number, uint64_t arg0) {
    register uint64_t a0 __asm__("a0") = arg0;
    register uint64_t a7 __asm__("a7") = number;
    __asm__ volatile("ecall" : "+r"(a0) : "r"(a7) : "memory");
    return (int64_t)a0;
}

static inline int64_t gravity_syscall2(uint64_t number, uint64_t arg0, uint64_t arg1) {
    register uint64_t a0 __asm__("a0") = arg0;
    register uint64_t a1 __asm__("a1") = arg1;
    register uint64_t a7 __asm__("a7") = number;
    __asm__ volatile("ecall" : "+r"(a0), "+r"(a1) : "r"(a7) : "memory");
    return (int64_t)a0;
}

static inline int64_t gravity_exit(int32_t status) {
    return gravity_syscall1(GRAVITY_SYS_EXIT, (uint64_t)(int64_t)status);
}

static inline int64_t gravity_yield(void) {
    return gravity_syscall0(GRAVITY_SYS_YIELD);
}

static inline int64_t gravity_send_message(uint64_t destination_task, uint64_t payload) {
    return gravity_syscall2(GRAVITY_SYS_SEND_MESSAGE, destination_task, payload);
}

/* On success, returns the payload and writes the sender task ID when non-null.
 * On an empty non-blocking mailbox, returns -GRAVITY_EAGAIN. */
static inline int64_t gravity_receive_message(uint64_t *sender_task) {
    register uint64_t a0 __asm__("a0") = 0;
    register uint64_t a1 __asm__("a1") = 0;
    register uint64_t a7 __asm__("a7") = GRAVITY_SYS_RECEIVE_MESSAGE;
    __asm__ volatile("ecall" : "+r"(a0), "+r"(a1) : "r"(a7) : "memory");
    if (sender_task != 0 && (int64_t)a0 >= 0) {
        *sender_task = a1;
    }
    return (int64_t)a0;
}

static inline int gravity_is_error(int64_t result) {
    return result < 0 && result >= -4095;
}

#ifdef __cplusplus
}
#endif

#endif /* GRAVITY_SYSCALL_H */
