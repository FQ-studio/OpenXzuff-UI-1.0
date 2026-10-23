# OpenXzuff UI — Kernel boundary

This directory defines the first host-testable contract layer for the planned Gravity Kernel RV64 microkernel target. It is not a bootable kernel and does not contain a scheduler, page tables, trap entry, or hardware drivers yet.

## Scope
- Typed syscall identifiers for the OS/user boundary.
- A minimal IPC message shape.
- Overflow-safe user-address range validation for a future kernel implementation.
- No Linux, Flutter, Python, or third-party runtime dependency.

These contracts must be mapped to the real Gravity Kernel ABI only after its source and calling conventions are available. These identifiers are not an implemented syscall table.
