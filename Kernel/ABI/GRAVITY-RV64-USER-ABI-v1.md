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
- ReceiveMessage returns the sender task ID in a1; all other a1-a5 registers remain unchanged. Because a0 is also the error channel, scalar payloads whose signed value is -1 through -4095 are reserved in ABI v1 and must not be sent.
- The saved user PC advances by four bytes after the 32-bit ecall instruction.
- -11 (-EAGAIN) means a non-blocking receive found no message or a mailbox is full; -22 (-EINVAL) means an invalid argument; -14 (-EFAULT) means an invalid user address; -38 (-ENOSYS) means unknown or not implemented.

| ID | Name | Current RV64 trap/runtime status |
|---:|---|---|
| 0 | Exit | Implemented for the current U-mode test task; marks it Exited and schedules the next task. General resource reaping is not implemented |
| 1 | SendMessage | Implemented for the built-in U-mode probe task; sends one scalar machine word to a task FIFO mailbox and returns 0, -EAGAIN, or -EINVAL |
| 2 | ReceiveMessage | Implemented as non-blocking receive; returns payload in a0 and sender task ID in a1, or -EAGAIN when empty |
| 3 | MapMemory | Implemented for the built-in U-mode probe: first-fit or hinted mapping in a bounded 2 MiB heap; zero-filled frames; READ/WRITE/EXECUTE/USER flags; W+X and write-only requests rejected |
| 4 | Yield | Implemented for the current U-mode test task; saves its trap frame, rotates the run queue, and resumes it later |
| 7 | UnmapMemory | Implemented for pages in the built-in 2 MiB user heap; preflights ownership, invalidates mappings, scrubs frames and returns them to the physical allocator |
| 8 | ProtectMemory | Implemented for mapped heap pages; accepts R, RW or X user permissions and rejects W+X |

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

The QEMU probe uses dedicated U-mode text and stack pages plus a single statically provisioned user address space. The kernel and probe roots share supervisor identity mappings, while U permission is granted only to the probe's RX text, RW stack and explicitly allocated heap leaves. The linked kernel image uses section-aware W^X permissions: supervisor RX for .text, supervisor R for .rodata, and supervisor RW for .data/.bss.

The built-in probe has a bounded 2 MiB heap arena. MapMemory allocates zero-filled physical frames from the active 1 GiB identity window; ProtectMemory changes permissions only on mapped heap pages; UnmapMemory preflights the full range and allocator ownership, switches to the supervisor root, clears mappings, scrubs frames, and returns them to the allocator. The heap VA is derived from the bootstrap window base plus a 256 MiB offset, rather than hard-coded to QEMU's 0x80000000 RAM base. User leaf permissions enforce W^X. The QEMU probe checks W+X rejection for both MapMemory and ProtectMemory, verifies an invalid oversized unmap leaves the first page mapped, and verifies a second unmap returns -EFAULT.

This is not a general virtual-memory manager. Dynamic per-process roots, page-table-page allocation/reclamation, arbitrary process mappings, demand paging, copyin/copyout, ASIDs, remote TLB shootdown, SMP-safe page-table mutation, blocking IPC, endpoint permissions and general task resource reaping remain unimplemented.
