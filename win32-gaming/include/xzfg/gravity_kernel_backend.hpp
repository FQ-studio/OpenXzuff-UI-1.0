#ifndef XZFG_GRAVITY_KERNEL_BACKEND_HPP
#define XZFG_GRAVITY_KERNEL_BACKEND_HPP

#include "openxzuff/kernel/syscall.hpp"
#include "xzfg/gravity_host_abi.h"

namespace xzfg {

/* Bind the host ABI to the existing, hook-driven Gravity syscall dispatcher.
 * The caller owns hooks and must keep them alive for the API table lifetime.
 * Services without a matching implemented Gravity syscall return -ENOSYS. */
bool gravity_host_api_v1_bind_syscall_hooks(
    xzfg_gravity_host_api_v1 *api,
    openxzuff::kernel::SyscallHooks *hooks) noexcept;

} // namespace xzfg

#endif
