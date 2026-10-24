# Gravity Kernel RV64 User ABI — Draft 1

Status: initial compiler/loader contract draft. The kernel trap entry and user-mode loader are not yet implemented, so this document defines the intended contract, not proof that programs already execute on hardware.

## Target

- Architecture: RISC-V 64-bit (RV64), little-endian.
- ISA baseline: RV64I; generated code in the initial compiler subset uses only `addi` and `ecall`.
- User entry: loader sets the PC to the executable's entry offset after mapping the code segment.
- General register convention: use the standard RISC-V psABI for ordinary function calls. The syscall register convention below is specific to Gravity Kernel.
- Stack, heap, TLS and argument-vector setup are not yet specified; programs must not assume they exist.

## System call convention, ABI version 1

- Execute `ecall` to request a kernel service.
- `a7` contains the numeric syscall ID.
- `a0` through `a5` carry up to six arguments.
- For calls that return normally, `a0` carries the result; error encoding and preservation of other registers still need a formal kernel implementation.
- `Exit = 0`: `a0` is the process exit status. This call does not return.
- `SendMessage = 1`, `ReceiveMessage = 2`, `MapMemory = 3`, `Yield = 4`, matching `Kernel/include/openxzuff/kernel/contracts.hpp`.

## `.xexe` format version 1

All integer fields are little-endian. Header is exactly 32 bytes:

| Offset | Width | Field | Value/meaning |
|---:|---:|---|---|
| 0 | 8 | magic | bytes `58 5A 58 45 58 45 31 00` (`XZXEXE1\\0`) |
| 8 | 2 | format_version | 1 |
| 10 | 2 | header_size | 32 |
| 12 | 2 | architecture | 1 = RV64 |
| 14 | 2 | abi_version | 1 = Gravity Kernel ABI v1 |
| 16 | 8 | entry_offset | byte offset from file start; v1 compiler uses 32 |
| 24 | 4 | code_size | code bytes immediately following header |
| 28 | 4 | flags | reserved; must be zero |

For this minimal compiler subset, the file is the header followed by a single executable code segment. The loader must reject truncated files, unknown versions, wrong architecture/ABI, nonzero reserved flags, entry offsets outside the code, and code sizes that exceed the file.

## Initial compiler-generated exit stub

For `function main() returns i32 { return N; }`, where N is -2048..2047:

1. `addi a0, zero, N` sets the process exit status.
2. `addi a7, zero, 0` explicitly selects Gravity Kernel `Exit` (syscall ID 0).
3. `ecall` requests the kernel service.

The compiler does not depend on the initial value of `a7`.

## Not implemented

This ABI draft does not imply that trap handling, process isolation, `.xexe` loading, memory mapping, scheduler integration, syscall validation, or physical RV64 boot already works. Those are kernel milestones and need independent tests.