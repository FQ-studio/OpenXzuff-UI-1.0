# Gravity Kernel ABI v1 (RV64)

Status: contract primitives implemented; service implementation incomplete.
This file is normative for new OpenXzuff RV64 userspace code. Assigned syscall
numbers and structure layouts are stable within ABI major version 1.

## CPU and procedure boundary

- Target: RV64, little-endian, LP64 integer ABI.
- Standard RISC-V psABI: a0-a7 arguments, a0/a1 return values.
- Stack grows down and is 16-byte aligned at procedure entry and throughout calls.
- s0-s11 are callee-saved; gp and tp are fixed registers.
- Syscall instruction: ecall; syscall number in a7; six arguments in a0-a5.
- Return: non-negative a0 is success; a0 = -errno is failure.
- U-mode ecall is exception cause 8. S-mode ecall is cause 9.

## Address-space contract

- Initial userspace virtual-address scheme: Sv39 lower canonical half.
- User range is [0, 2^38), exclusive upper bound; page size is 4096 bytes.
- Pointer+length must be checked for zero length, overflow and user range.
- Range validation does not prove mapping or access permission. VM copy-in/copy-out
  must walk page tables and check U/R/W permissions before touching user memory.
- W+X mappings are forbidden by ABI v1. Unknown memory flag bits are rejected.

## Stable syscall number table

| Number | Name | Contract |
|---|---|---|
| 0x0000 | AbiInfo | ABI version/features query |
| 0x0100 | TaskOpen | Create task |
| 0x0101 | TaskExit | Exit current task with status |
| 0x0102 | TaskYield | Yield current task |
| 0x0103 | SystemShutdown | Request platform shutdown |
| 0x0104 | SystemOn | Reserved platform power-on request |
| 0x0105 | SystemReboot | Request platform reboot |
| 0x0200 | MemAlloc | Allocate virtual memory |
| 0x0201 | MemMap | Map memory |
| 0x0202 | MemUnmap | Unmap memory |
| 0x0300 | IpcSend | Send IPC message |
| 0x0301 | IpcReceive | Receive IPC message |
| 0x0302 | IpcCall | Send and wait for reply |
| 0x0400 | FbInfo | Query framebuffer |
| 0x0401 | FbMap | Map framebuffer |
| 0x0402 | FbPresent | Present framebuffer |
| 0x0500 | InputRead | Read input events |
| 0x0501 | InputWait | Wait for input |
| 0x0600 | ClockNow | Read monotonic time |
| 0x0601 | ClockSleep | Sleep until/for duration |
| 0x0700 | CapGrant | Grant capability rights |
| 0x0701 | CapRevoke | Revoke capability rights |

## Data types

The repr(C) types in src/abi.rs are the source of truth: AbiInfo,
TaskHandle { id, generation }, UserRange, IpcMessage and ClockTime.
Handles include a generation to prevent stale-ID reuse. Capability rights and
memory flags reject unknown bits. IPC messages carry a sender handle, a 64-bit
tag and six 64-bit payload words.

## Error values

Errors use negative values in a0. Canonical errno values are in
src/abi.rs::errno. Unknown syscall numbers return -ENOSYS.

## Implementation truth table

Definitions are not proof that services exist. At this revision:
- Implemented: stable number/type definitions, syscall register contract,
  negative-errno encoding, Sv39 user-range shape validation, memory/capability
  flag validation, normalized clock-time validation, TaskExit decode and
  platform shutdown/reboot request decode.
- Not yet implemented: U-mode process entry and safe trap stack transition,
  Sv39 page-table manager and user copy-in/out, user task creation/termination
  integration, IPC queues, capability tables, framebuffer/input services,
  monotonic clock service and timer sleep.
- Those syscall IDs deliberately return -ENOSYS until the corresponding
  subsystem is implemented and tested. Do not advertise them as available.

## RISC-V normative references

- RISC-V ELF psABI, Procedure Calling Convention:
  https://docs.riscv.org/reference/abi/v1.0/riscv-cc-procedure-calling-convention.html
- RISC-V Privileged ISA, Supervisor ISA (traps and Sv39):
  https://docs.riscv.org/reference/isa/priv/supervisor.html
- RISC-V Ratified Specifications Library:
  https://docs.riscv.org/
