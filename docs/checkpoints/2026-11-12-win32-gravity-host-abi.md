# Win32 Gaming — Gravity Host ABI v1 checkpoint

Date: 2026-11-12
Branch: `feature/win32-gaming-gravity-port`
Pull request: #12

## Implemented in this checkpoint

- Added `win32-gaming/include/xzfg/gravity_host_abi.h`.
- Defined a versioned, C-compatible ABI table (`XZFG_GRAVITY_HOST_ABI_VERSION=1`).
- Added callback slots for task yield/exit/create, virtual memory map/unmap/protect, opaque VFS/service handles, and waitable objects.
- Added status values aligned with the existing Gravity error conventions (`-EINVAL`, `-EFAULT`, `-ENOSYS`, `-ENOMEM`, `-EAGAIN`).
- Added API shape validation for version, structure size and mandatory callbacks.
- Added a host mock contract test at `win32-gaming/tests/gravity_host_abi_test.cpp`.
- Registered the test as `win32-gravity-host-abi-contract` in CTest.

## Existing ABI facts inspected

- The repository's C++ syscall contract currently declares Exit=0, SendMessage=1, ReceiveMessage=2, MapMemory=3, Yield=4.
- The C++ dispatch implementation currently implements Exit and Yield hooks; SendMessage, ReceiveMessage and MapMemory return `-ENOSYS` in that host-testable dispatcher.
- The RV64 Rust runtime has working test paths for Yield, Exit, SendMessage and ReceiveMessage, but that does not yet provide a complete general-purpose process/VM/VFS API.
- This host ABI is an adapter contract and does not invent syscall numbers or claim that missing kernel services already exist.

## Validation status

- No local CMake build or CTest was executed in this environment.
- No GitHub Actions run was returned for the latest PR commit at checkpoint time.
- No RV64 cross-build or QEMU test was executed for Win32 Gaming.
- No PE loader, Windows API implementation, Wine/Proton port, or x86/x64-to-RV64 translator is implemented.
- This is a host-mock ABI contract milestone, not a working Windows game runtime.

## Next steps

1. Implement the host mock test in CI and fix any compiler/test failures.
2. Implement the Gravity kernel-side adapter only against existing documented kernel APIs; identify missing syscalls/services explicitly.
3. Add VFS/handle and waitable-object service contracts before implementing Win32 file and synchronization APIs.
4. Implement PE32/PE32+ parser with malformed-input tests.
5. Design and benchmark x86/x64-to-RV64 translation separately.
