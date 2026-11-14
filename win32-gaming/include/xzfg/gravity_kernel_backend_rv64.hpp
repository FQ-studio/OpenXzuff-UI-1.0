#ifndef XZFG_GRAVITY_KERNEL_BACKEND_RV64_HPP
#define XZFG_GRAVITY_KERNEL_BACKEND_RV64_HPP

#include "xzfg/gravity_host_abi.h"

#if !defined(__riscv) || !defined(__riscv_xlen) || __riscv_xlen != 64
#error "The Gravity RV64 backend must be compiled for RV64 user mode"
#endif

#ifdef __cplusplus
extern "C" {
#endif

/* Populate ABI v1 with callbacks that issue real U-mode ecall instructions.
 * Yield, Exit, scalar IPC, and bounded user-VM calls enter the live Rust
 * kernel/src/trap.rs -> gravity_u_trap_handler syscall path. Task creation,
 * VFS handles, and waitables remain optional and unbound. */
int xzfg_gravity_host_api_v1_bind_rv64(xzfg_gravity_host_api_v1 *api);

#ifdef __cplusplus
}
#endif

#endif
