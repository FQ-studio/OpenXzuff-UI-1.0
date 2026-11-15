# 2026-11-15 Afternoon Integration Checkpoint

## Repository and source revision

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feature/win32-gaming-gravity-port`
- Parent revision before this integration: `4469eed65b0aa72b78251ece4d2a9529d339bc5c`
- This checkpoint records the integration commit built on that parent. The commit SHA and its Actions results are recorded in the post-validation update.

## Changes integrated in this session

1. **FDT RAM-node status filtering**
   - Missing `status` remains enabled.
   - Only `status = "okay"` or `"ok"` contributes a RAM range.
   - Explicit `disabled`, `reserved`, or other values are ignored.
   - Host fixtures cover disabled/reserved rejection and okay/ok acceptance.
   - The newer FDT `has_compatible` helper from the feature branch is preserved; the older FDT file was not copied wholesale.

2. **Gravity SDK / RV64 NDK**
   - Added ABI v1 constants and inline RV64 wrappers for Exit, SendMessage, ReceiveMessage and Yield.
   - Added compile-only `abi_smoke.c`.
   - Added CMake RV64 freestanding toolchain and object-only example.
   - The SDK deliberately does not expose general Map/Unmap/Protect wrappers: the live kernel VM calls are restricted to the built-in U-mode probe and one bounded heap.

3. **Xzuff Go code-generation correction**
   - Imported the signed-i32 boundary fix using ADDIW to sign-extend the LUI result.
   - Added the INT32_MAX self-test already present in the verified main-branch compiler source.
   - Updated compiler documentation to match the actual supported subset.

4. **CI and documentation**
   - Core workflow now builds the Gravity RV64 SDK object and verifies an `ecall` in disassembly.
   - SDK/NDK source changes trigger the Core workflow; Win32 bridge source changes also trigger it.
   - ABI and architecture documentation record the real compile-only boundaries.

## Validation status

- The feature branch parent had successful RV64, VisionFive 2 host-readiness, Core and Win32 bridge Actions for commit `4469eed65b0aa72b78251ece4d2a9529d339bc5c`.
- The source integration recorded here has **not** been declared PASS until Actions for this new commit complete.
- Required checks: host Rust tests, RV64 cross-build, QEMU serial assertions, host CMake/CTest, Gravity NDK cross-compile/disassembly, Android ARM64 Xzuff Go build, Luau/JS/Lua/3D smoke tests.
- Physical board boot is not part of these CI results.

## Known remaining work

- Full FDT device-node discovery: parent cell inheritance, `ranges`, translated `reg`, clocks, resets, interrupts and status.
- Validated MMIO mapping, HAL device lifecycle, UART initialization/receive/flush and PLIC routing.
- Dynamic task frame/stack allocation, scheduler slot reuse, independent per-process Sv39 roots, VM teardown and safe user copy-in/copy-out.
- VFS/handle integration, persistent block storage, mount namespaces and file permissions.
- Blocking IPC, wait queues, networking, graphics/audio/input services.
- General `.xexe` loader/linker and Win32 PE/DLL/API compatibility runtime.
- Repeated QEMU stress tests and physical-board/LAVA validation.

A successful compile-only SDK test or QEMU smoke test does not establish a complete or production-stable OS.
