#include "gravity/syscall.h"

/*
 * Deliberately fail closed until each service has a real implementation,
 * validated user-memory access, and capability checks. Do not report success
 * for reserved syscall IDs.
 */
intptr_t gravity_syscall_dispatch(uintptr_t number,
                                  const gravity_syscall_args_t *args)
{
    (void)number;
    (void)args;
    return -(intptr_t)GRAVITY_ENOSYS;
}
