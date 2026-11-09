# OpenXzuff UI — Gravity Kernel contract layer

This directory contains the first host-testable C++ contract layer for the planned Gravity Kernel RV64 microkernel. It is not a bootable kernel and is separate from the Rust RV64 S-mode bring-up image under `kernel/`.

## Implemented and tested on the host

- Typed Gravity syscall IDs and a dispatcher for `Exit` and `Yield` through injected hooks.
- `-ENOSYS` results for unknown or not-yet-implemented IPC and memory-mapping calls, and when a required hook is absent.
- An RV64 S-mode trap-frame memory-layout contract with compile-time offset, size, and alignment checks.
- A host-testable U-mode `ecall` dispatch policy: syscall ID from `a7`, arguments from `a0..a5`, return value in `a0`, and a four-byte `sepc` advance for returning calls. Non-returning `Exit` does not advance the saved PC.
- An SBI call wrapper with a host fallback, plus smoke-test coverage.

The root CMake build registers the syscall, trap-frame, and U-mode dispatch tests. These tests validate C++ policy and data layout only; they do not execute privileged instructions.

## Not implemented by this directory

- Assembly trap entry, register save/restore, `stvec` installation, or `sret` return.
- User/supervisor privilege transitions or execution of a user `.xexe`.
- A scheduler, context switching, process lifecycle, page tables, or user-memory validation in the trap path.
- Kernel IPC transport, physical device drivers, or a hardware boot path.

## Separate Rust RV64 S-mode bring-up image

The executable Rust image in lowercase `kernel/` is separate from this C++ contract layer. Its current OpenSBI/QEMU milestone includes:

- S-mode entry, FDT parsing, a physical-page allocator, and a fixed 64 KiB kernel heap.
- A direct-mode `stvec` trap vector with a 288-byte, 16-byte-aligned frame. The assembly entry saves/restores the integer GPR set and preserves `sepc`/`sstatus` across a Rust handler call before returning with `sret`.
- A delegated-breakpoint smoke test that checks the register round-trip (including `sp`, `gp`, and `tp`), the zero register, stack alignment, and SPP state. The handler advances the saved PC by the decoded 16-bit or 32-bit instruction length.
- A fixed two-task S-mode timer-preemption smoke runtime. It programs the SBI TIME timer, handles supervisor timer interrupts, saves the interrupted `TrapFrame`, rotates the round-robin scheduler, and restores the next task's saved stack/register context through `sret`. QEMU emits `RV64 timer preemption + stack context switch: PASS` only after the second task has executed.

These QEMU checks cover only the S-mode breakpoint and this fixed two-task timer-preemption path. They do not provide U-mode entry, an `sscratch` supervisor-stack switch, user-process execution, or a connection from this Rust handler to the C++ U-mode `ecall` dispatcher. General task lifecycle and scheduling APIs, page faults, non-timer interrupt sources, page-table isolation, IPC, and physical-board boot remain unimplemented.
