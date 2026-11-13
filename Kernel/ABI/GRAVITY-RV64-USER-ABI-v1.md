# Gravity Kernel RV64 User ABI — Draft 1

Status: ABI v1 contract with a limited live RV64 trap/runtime probe. The supervisor trap entry, mixed S/U-mode frame restore, bootstrap Sv39 mappings, timer preemption, Yield, and a bounded scalar-message IPC round-trip execute under OpenSBI/QEMU. A general executable loader and application process model are still not implemented.

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
- Returning calls preserve a1 through a5 unless the syscall explicitly defines a secondary output. The caller treats a0 as clobbered.
- ReceiveMessage returns the sender task ID in a1; all other a1-a5 registers remain unchanged.
- The saved user PC advances by four bytes after the 32-bit ecall instruction.
- -11 (-EAGAIN) means a non-blocking receive found no message or a mailbox is full; -22 (-EINVAL) means an invalid argument; -14 (-EFAULT) means an invalid user address; -38 (-ENOSYS) means unknown or not implemented.

| ID | Name | Current RV64 trap/runtime status |
|---:|---|---|
| 0 | Exit | Implemented for the current U-mode test task; marks it Exited and schedules the next task. General resource reaping is not implemented |
| 1 | SendMessage | Implemented for the built-in U-mode probe task; sends one scalar machine word to a task FIFO mailbox and returns 0, -EAGAIN, or -EINVAL |
| 2 | ReceiveMessage | Implemented as non-blocking receive; returns payload in a0 and sender task ID in a1, or -EAGAIN when empty |
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

For function main() returns i32 { return N; }, where N is any signed 32-bit value:

1. If N fits a signed 12-bit immediate, addi a0, zero, N sets the process exit status.
2. Otherwise, the compiler uses lui and, when the low 12 bits are nonzero, addiw a0, a0, low. ADDIW sign-extends the resulting 32-bit value to RV64; this is required for values near INT32_MAX.
3. addi a7, zero, 0 selects Gravity Kernel Exit (syscall ID 0).
4. ecall requests the kernel service.

The compiler does not depend on the initial value of a7. The live hardware path implements Exit for the built-in U-mode test task; general process cleanup and resource reaping remain incomplete.

## Implementation boundary

The QEMU probe uses a dedicated U-mode text page and user stack. The bootstrap now builds two distinct Sv39 roots: a supervisor-only kernel root and a probe-user root that shares supervisor identity mappings but grants U permission only to the probe RX text and RW stack leaves. The linked kernel image uses section-aware W^X leaves in both roots: supervisor RX for .text, supervisor R for .rodata, and supervisor RW for .data/.bss; the remaining bootstrap identity map is supervisor RW/NX. The scheduler switches `satp` to the selected task's root before restoring its frame. The probe verifies that U-mode cannot load from the supervisor-only kernel image or access the supervisor trap stack, is preempted by the SBI timer, sends one scalar request through a bounded FIFO mailbox to task 0, yields to let task 0 receive it and enqueue a reply, then receives that reply before Exit. Mailboxes hold eight machine-word messages per task and are protected against same-hart timer preemption while being mutated. The user root is still a statically provisioned single test address space: dynamic per-process roots, page-table-page allocation/reclamation, arbitrary process mappings, user-buffer copying, blocking IPC, endpoint permissions, SMP-safe queue locking, MapMemory syscall, and general task resource reaping are not implemented.
