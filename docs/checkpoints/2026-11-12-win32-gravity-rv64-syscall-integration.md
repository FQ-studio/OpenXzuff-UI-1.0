# Win32 Gaming — live Gravity RV64 syscall bridge

Date: 2026-11-12

## Implemented

- Added a dedicated RV64 U-mode C++ adapter. Its callbacks issue actual `ecall` instructions; they do not route through the host-testable C++ mock dispatcher.
- Published stable syscall IDs in `gravity_host_abi.h` and mirrored them in `kernel/src/syscall_abi.rs`.
- Routed the Rust U-mode trap dispatcher through named ABI constants.
- Connected Yield (4) and Exit (0) to the existing task scheduler path.
- Implemented a bounded Sv39 user heap arena at `0x90000000`, with a 2 MiB maximum range for the current built-in U-mode process.
- Implemented MapMemory (3), UnmapMemory (7), and ProtectMemory (8).
- Physical pages are allocated only from the kernel identity-mapped direct-map window, zeroed while the kernel page-table root is active, mapped with U-mode permissions, and returned to the physical page allocator on unmap.
- Added an RV64 assembly probe that checks invalid protection rejection, map, user read/write, read-only protection, and unmap.
- Added a host allocator test for bounded direct-map allocation and CMake exposure for the RV64 adapter.

## Current boundary

This is a live kernel syscall bridge for the single built-in U-mode process, not yet a general multi-process VM subsystem.

Still unavailable:
- task creation and independent user address spaces;
- VFS/service handles and file read/write/close;
- waitable objects and blocking synchronization;
- networking, graphics, audio, and input services;
- PE32/PE32+ loader and Win32 API surface;
- x86/x64-to-RV64 instruction translation.

The adapter leaves unavailable callbacks null. The memory arena is fixed at 2 MiB and currently supports only the one built-in U-mode task.

## Validation state

- The earlier RV64 S-mode/QEMU baseline passed before the dynamic user-VM changes.
- The new QEMU memory-map/protect/unmap probe and the RV64 adapter cross-build have been added to CI. Their results must be checked before marking this checkpoint passed.
- Host mock tests are supplemental and are not evidence of live kernel execution.
