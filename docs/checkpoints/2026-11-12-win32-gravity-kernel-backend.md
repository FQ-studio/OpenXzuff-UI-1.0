# Win32 Gaming Gravity Kernel Backend — 2026-11-12

## Real RV64 kernel path now present

The repository contains two deliberately separate backends:

1. `gravity_kernel_backend.cpp` is a host-testable C++ policy shim. It can exercise injected Yield/Exit hooks but is not the live kernel.
2. `gravity_kernel_backend_rv64.cpp` is the RV64 U-mode adapter. Its callbacks issue real `ecall` instructions. Those calls enter `kernel/src/trap.rs`, then `kernel/src/runtime.rs`, and reach the Rust Gravity scheduler, IPC mailbox, and Sv39 VM implementation.

The RV64 adapter currently binds:
- Exit (syscall 0)
- SendMessage (syscall 1)
- ReceiveMessage (syscall 2)
- MapMemory (syscall 3)
- Yield (syscall 4)
- UnmapMemory (syscall 7)
- ProtectMemory (syscall 8)

The host ABI v1 table now has optional scalar `message_send` and `message_receive` callbacks appended after the original fields. Validation uses the original v1 prefix size, so older v1 tables remain valid. The RV64 adapter populates these callbacks; the host-only shim leaves them null because the C++ policy dispatcher does not implement IPC.

## Kernel capabilities verified in the source

- Rust U-mode trap handler dispatches the syscall IDs above to concrete runtime functions.
- Scheduler performs round-robin selection and timer-driven context switching.
- SBI TIME timer interrupts preempt S-mode tasks and the built-in U-mode probe task.
- IPC provides bounded, non-blocking scalar mailboxes.
- Sv39 supports the built-in U-mode task's bounded 2 MiB heap arena: map, protect, unmap and physical-page return.
- Kernel/user address spaces are distinct; the current runtime has one built-in U-mode process.

## Remaining blockers — not complete

These capabilities are still absent or limited, and must not be reported as implemented:

- Dynamic task/process creation from arbitrary entry points. The scheduler policy can allocate task IDs, but the live runtime has only three statically provisioned tasks and only one U-mode task context.
- Multiple independent user address spaces and per-process page-table ownership.
- General VFS/file handles, directory operations, async I/O, waitable objects and blocking IPC.
- Network, graphics, audio and input service bindings for the Win32 host.
- PE32/PE32+ loader, imports/relocations, Win32 API/DLL implementation.
- x86/x64-to-RV64 translation, Windows exception/SEH mapping and real Windows application execution.
- Full runtime link of the RV64 adapter into a packaged Win32 runtime. CI compiles the adapter object and separately QEMU-boots the Rust kernel; this is not yet an end-to-end game test.

## CI validation added

The RV64 S-mode kernel workflow now also compiles `gravity_kernel_backend_rv64.cpp` with an RV64 cross-compiler and is triggered by changes under `win32-gaming/**` and `CMakeLists.txt`. Its QEMU stage continues to assert the real Rust kernel's timer, context-switch, IPC and VM smoke tests.

The host bridge workflow still validates only the host ABI/policy shim. Do not treat its 2/2 result as proof of RV64 hardware execution.

## Scope statement

This is a genuine RV64 syscall-to-Rust-kernel integration slice for the currently supported single built-in U-mode task. It is not a complete process subsystem or a runnable Windows/Proton compatibility layer.
