# Gravity user-space syscall ABI (RV64)

`syscall.rs` defines the first user-facing wrapper layer for Gravity's RV64
syscall convention:

| Operation | ID | Current kernel status |
|---|---:|---|
| `task_open(entry, argument)` | `0x0100` | Not implemented; kernel returns `-ENOSYS` |
| `task_exit(status)` | `0x0101` | Exit request is decoded, but currently falls back to a kernel halt |
| `task_yield()` | `0x0102` | Not implemented; kernel returns `-ENOSYS` |

The RV64 wrapper places the syscall number in `a7`, arguments in `a0-a5`,
executes `ecall`, and reads the return value from `a0`. Non-RV64 builds use
a fail-closed stub returning `-38` (`ENOSYS`) so the ABI module can be
host-compiled for basic tests.

**Important:** these wrappers do not by themselves create user tasks or make
user-mode scheduling operational. The kernel still needs validated user entry
addresses and stacks, user context/trap-frame switching, and scheduler-aware
exit/yield handling before these calls can safely perform their requested
operations. Do not pass arbitrary pointers to the kernel-thread scheduler.
