#ifndef GRAVITY_SYSCALL_H
#define GRAVITY_SYSCALL_H

#include <stdint.h>

/*
 * Gravity Kernel syscall ABI draft for RV64.
 * a7 = call number; a0-a5 = arguments; a0 = signed result.
 * Negative results are -errno. These IDs are reserved ABI values, not a
 * claim that the corresponding kernel services are implemented yet.
 */
enum gravity_syscall_id {
    GRAVITY_TASK_CREATE = 0x0100,
    GRAVITY_TASK_EXIT   = 0x0101,
    GRAVITY_TASK_YIELD  = 0x0102,

    GRAVITY_MEM_ALLOC   = 0x0200,
    GRAVITY_MEM_MAP     = 0x0201,
    GRAVITY_MEM_UNMAP   = 0x0202,

    GRAVITY_IPC_SEND    = 0x0300,
    GRAVITY_IPC_RECEIVE = 0x0301,
    GRAVITY_IPC_CALL    = 0x0302,

    GRAVITY_FB_INFO     = 0x0400,
    GRAVITY_FB_MAP      = 0x0401,
    GRAVITY_FB_PRESENT  = 0x0402,

    GRAVITY_INPUT_READ  = 0x0500,
    GRAVITY_INPUT_WAIT  = 0x0501,

    GRAVITY_CLOCK_NOW   = 0x0600,
    GRAVITY_CLOCK_SLEEP = 0x0601,

    GRAVITY_CAP_GRANT   = 0x0700,
    GRAVITY_CAP_REVOKE  = 0x0701
};

enum gravity_errno {
    GRAVITY_EPERM  = 1,
    GRAVITY_ENOENT = 2,
    GRAVITY_EIO    = 5,
    GRAVITY_EAGAIN = 11,
    GRAVITY_ENOMEM = 12,
    GRAVITY_EFAULT = 14,
    GRAVITY_EINVAL = 22,
    GRAVITY_ENOSYS = 38
};

typedef struct gravity_syscall_args {
    uintptr_t a0, a1, a2, a3, a4, a5;
} gravity_syscall_args_t;

/* Returns zero or a negative Gravity errno. Unknown/unimplemented calls: -ENOSYS. */
intptr_t gravity_syscall_dispatch(uintptr_t number,
                                  const gravity_syscall_args_t *args);

#endif
