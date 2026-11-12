# Win32 Gaming Gravity Kernel Backend Bridge — 2026-11-12

## Implemented in this checkpoint

- Added `gravity_kernel_backend.hpp/.cpp` as a C++ host bridge from the Win32 Gaming ABI v1 table to the existing hook-driven Gravity syscall dispatcher.
- `task_yield` dispatches the existing syscall ID 4 and forwards through the scheduler hook.
- `task_exit` dispatches the existing syscall ID 0 and forwards the signed 32-bit exit status through the process-exit hook.
- Added an explicit backend test for Yield, Exit, absent hooks, and unsupported memory operations.
- Added a dedicated GitHub Actions workflow that configures CMake, builds the ABI/backend host tests, and runs the matching CTest cases.

## Deliberate limits

This is a host-testable bridge to the existing C++ syscall policy dispatcher. It is **not yet a direct binding to the live Rust RV64 trap/runtime path**.

The current syscall contract only assigns:
- 0 — Exit
- 1 — SendMessage
- 2 — ReceiveMessage
- 3 — MapMemory
- 4 — Yield

The C++ dispatcher currently implements Exit and Yield through injected hooks. SendMessage, ReceiveMessage, and MapMemory return `-ENOSYS` in that dispatcher. The live Rust RV64 probe has a limited working path for scalar IPC, Yield, and Exit, but does not yet provide a general user-process VM API.

Accordingly:
- `memory_map` and `memory_unmap` validate arguments, then return `-ENOSYS`.
- `task_create`, memory protection, VFS handles, and waitables remain unavailable.
- No Linux syscall, Linux kernel, glibc, or Linux device-node dependency is introduced.
- No x86/x64 instruction translation or Windows PE execution is implemented by this bridge.

## Validation

The dedicated workflow was created and triggered for PR #12 at head commit `ec4096b9ef1d7bb82a7d6ae0ad0bd692e9edd8f2`. At checkpoint authoring time its status was still in progress; no PASS claim is made until the job completes successfully.

The workflow tests the host ABI and C++ dispatcher bridge only. It is not an RV64/QEMU or physical-board integration test.
