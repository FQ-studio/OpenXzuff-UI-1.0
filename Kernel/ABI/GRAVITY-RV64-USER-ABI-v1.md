# Gravity Kernel RV64 User ABI — Draft 1

Status: ABI v1 contract with a limited live RV64 trap/runtime probe. The supervisor trap entry, mixed S/U-mode frame restore, bootstrap Sv39 mappings, timer preemption, and Yield syscall path execute under OpenSBI/QEMU. A general executable loader and application process model are still not implemented.

## Target

- Architecture: RISC-V 64-bit (RV64), little-endian.
- ISA baseline: RV64I; generated code in the initial compiler subset uses only addi and ecall.
- User entry: a future loader sets the PC to the executable's entry offset after mapping the code segment.
- General register convention: standard RISC-V psABI for ordinary function calls.
- General stack, heap, TLS, argv, and environment setup are not yet specified.

## System call convention, ABI version 1

- Execute ecall to request a kernel service.
- a7 contains the numeric syscall ID.
- a0 through a5 carry up to six arguments.
- For a returning call, a0 carries either a non-negative result or a negative error number (-errno). Values from -1 through -4095 are reserved for errors.
- ABI v1 preserves a1 through a5 across a returning system call. The caller treats a0 as clobbered.
- The saved user PC advances by four bytes after the 32-bit ecall instruction.
- -22 (-EINVAL) means an invalid argument; -14 (-EFAULT) means an invalid user address; -38 (-ENOSYS) means unknown or not implemented.

| ID | Name | Current RV64 trap/runtime status |
|---:|---|---|
| 0 | Exit | -ENOSYS; process teardown/reaping is not wired to the hardware path |
| 1 | SendMessage | -ENOSYS; exercised by the U-mode QEMU probe |
| 2 | ReceiveMessage | -ENOSYS |
| 3 | MapMemory | -ENOSYS |
| 4 | Yield | Implemented for the current U-mode test task; saves its trap frame, rotates the run queue, and resumes it later |

The host-testable C++ dispatcher in Kernel/src/syscall.cpp remains a policy boundary with injected hooks. The live RV64 implementation is the Rust trap path in kernel/src/trap.rs and kernel/src/runtime.rs.

## .xexe format version 1

All integer fields are little-endian. Header is exactly 32 bytes:

| Offset | Width | Field | Value/meaning |
|---:|---:|---|---|
| 0 | 8 | magic | bytes 58 5A 58 45 58 45 31 00 (XZXEXE1\0) |
| 8 | 2 | format_version | 1 |
| 10 | 2 | header_size | 32 |
| 12 | 2 | architecture | 1 = RV64 |
| 14 | 2 | abi_version | 1 = Gravity Kernel ABI v1 |
| 16 | 8 | entry_offset | byte offset from file start; v1 compiler uses 32 |
| 24 | 4 | code_size | code bytes immediately following header |
| 28 | 4 | flags | reserved; must be zero |

The loader must reject truncated files, unknown versions, wrong architecture/ABI, nonzero reserved flags, entry offsets outside the code, and code sizes that exceed the file.

## Initial compiler-generated exit stub

For function main() returns i32 { return N; }, where N is -2048..2047:

1. addi a0, zero, N sets the process exit status.
2. addi a7, zero, 0 selects Gravity Kernel Exit (syscall ID 0).
3. ecall requests the kernel service.

The compiler does not depend on the initial value of a7. The live hardware path currently returns -ENOSYS for Exit because process teardown is not implemented.

## Implementation boundary

The QEMU probe uses a dedicated U-mode text page and user stack. It verifies that U-mode cannot load from the supervisor-only kernel image, that SendMessage returns -ENOSYS, and that Yield performs a real mixed-privilege context switch. This does not yet imply a general .xexe loader, process isolation for arbitrary applications, user pointer validation/copying, IPC transport, memory mapping syscall, or task exit/reaping.
