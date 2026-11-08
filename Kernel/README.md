# OpenXzuff UI — Kernel boundary

This directory defines the first host-testable contract layer for the planned Gravity Kernel RV64 microkernel target. It is not a bootable kernel and does not contain a scheduler, page tables, trap entry, or hardware drivers yet.

## Scope
- Typed syscall identifiers and a host-testable dispatcher for Exit and Yield.
- An RV64 S-mode trap-frame memory-layout contract with compile-time offset/alignment checks.
- A minimal IPC message shape.
- Overflow-safe user-address range validation for a future kernel implementation.
- No Linux, Flutter, Python, or third-party runtime dependency.

These are host-testable policy and layout contracts, not a hardware trap path. IPC, memory mapping, scheduler internals, trap entry, context switching, and physical boot remain unimplemented.
