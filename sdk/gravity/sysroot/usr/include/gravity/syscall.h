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

static inline int64_t gravity_syscall4(uint64_t number, uint64_t arg0, uint64_t arg1, uint64_t arg2, uint64_t arg3) {
    register uint64_t a0 __asm__("a0") = arg0;
    register uint64_t a1 __asm__("a1") = arg1;
    register uint64_t a2 __asm__("a2") = arg2;
    register uint64_t a3 __asm__("a3") = arg3;
    register uint64_t a7 __asm__("a7") = number;
    __asm__ volatile("ecall" : "+r"(a0), "+r"(a1), "+r"(a2), "+r"(a3) : "r"(a7) : "memory");
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

static inline int64_t gravity_open(const char *path, uint64_t rights) {
    return gravity_syscall2(GRAVITY_SYS_OPEN, (uint64_t)(uintptr_t)path, rights);
}

static inline int64_t gravity_read(uint64_t handle, void *buffer, uint64_t count, uint64_t offset) {
    return gravity_syscall4(GRAVITY_SYS_READ, handle, (uint64_t)(uintptr_t)buffer, count, offset);
}

static inline int64_t gravity_write(uint64_t handle, const void *buffer, uint64_t count, uint64_t offset) {
    return gravity_syscall4(GRAVITY_SYS_WRITE, handle, (uint64_t)(uintptr_t)buffer, count, offset);
}

static inline int64_t gravity_close(uint64_t handle) {
    return gravity_syscall1(GRAVITY_SYS_CLOSE, handle);
}

static inline int gravity_is_error(int64_t result) {
    return result < 0 && result >= -4095;
}

/* ABI v1 reserves the negative errno range (-4095..-1) for syscall errors. */
static inline int64_t gravity_send_message(uint64_t destination_task, uint64_t payload) {
    if (gravity_is_error((int64_t)payload)) {
        return -GRAVITY_EINVAL;
    }
    return gravity_syscall2(GRAVITY_SYS_SEND_MESSAGE, destination_task, payload);
}

/* On success, returns the payload and writes the sender task ID when non-null.
 * On an empty non-blocking mailbox, returns -GRAVITY_EAGAIN. Payload bit
 * patterns in the reserved negative errno range cannot be sent by this API. */
static inline int64_t gravity_receive_message(uint64_t *sender_task) {
    register uint64_t a0 __asm__("a0") = 0;
    register uint64_t a1 __asm__("a1") = 0;
    register uint64_t a7 __asm__("a7") = GRAVITY_SYS_RECEIVE_MESSAGE;
    __asm__ volatile("ecall" : "+r"(a0), "+r"(a1) : "r"(a7) : "memory");
    if (sender_task != 0 && !gravity_is_error((int64_t)a0)) {
        *sender_task = a1;
    }
    return (int64_t)a0;
}

#ifdef __cplusplus
}
#endif

#endif /* GRAVITY_SYSCALL_H */
