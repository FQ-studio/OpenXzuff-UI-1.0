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

The separate Rust image in `kernel/` currently boots in S-mode under OpenSBI/QEMU and initializes the FDT memory map, physical page allocator, and fixed kernel heap. That milestone does not mean the C++ U-mode trap contract is wired into the Rust boot image or that user processes can run.
