# 2026-11-15 Afternoon Integration Checkpoint

## Repository state

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feature/win32-gaming-gravity-port`
- Latest code commit: `4fcb7fe34da3d984de5a46a29a110bd1c525dd10`
- The feature branch includes a merge commit bringing in `main` at `ead5ccec0d675dd6274b814379219ec05ba0fdd6`.
- PR #12 remains open. GitHub reports the branch mergeable; no PR merge was performed.

## Changes integrated

1. **FDT memory-node status filtering**
   - Missing `status` means enabled.
   - Only `okay` and `ok` memory nodes contribute RAM.
   - `disabled`, `reserved` and other explicit states are ignored.
   - Host tests cover disabled/reserved rejection and okay/ok acceptance.
   - The feature branch's FDT compatible-string helper was preserved.

2. **Gravity SDK / RV64 NDK**
   - Added the ABI v1 header constants, errno values, and inline wrappers for Exit, SendMessage, ReceiveMessage and Yield.
   - Added the freestanding RV64 CMake toolchain and compile-only `abi_smoke.c`.
   - SDK and NDK workflow compiles an RV64 object and verifies `ecall` in disassembly.
   - Map/Unmap/Protect constants are present, but no general VM wrappers are exposed because the live handlers still target the built-in U-mode probe and its single 2 MiB heap.

3. **Xzuff Go code generation**
   - Corrected signed-i32 RV64 code generation to use ADDIW after LUI where required.
   - Added the INT32_MAX boundary self-test.
   - Updated compiler documentation to describe the supported one-function integer-return subset.

4. **Win32 bridge / SDK ABI integration**
   - `gravity_host_abi.h` now imports the shared Gravity SDK ABI header.
   - Win32 syscall IDs and protection flags map directly to `GRAVITY_SYS_*` and `GRAVITY_PROT_*` constants.
   - CMake host/RV64 targets and both adapter workflows include the SDK sysroot.
   - Host tests add compile-time assertions for all syscall and protection mappings.
   - Win32 bridge workflow now runs when kernel ABI or SDK source changes.

5. **Branch synchronization**
   - Imported the six main-branch SDK/NDK/compiler workflow changes into the feature branch.
   - Created a merge commit with main as second parent; branch compare is 111 commits ahead, 0 behind.

## Verified Actions evidence

### Latest Gravity Kernel code commit `4fcb7fe34da3d984de5a46a29a110bd1c525dd10`

- [RV64 S-Mode Kernel #43688775193](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688775193): **SUCCESS**.
  - RV64 adapter compile: passed.
  - Rust host tests: **60 passed, 0 failed**.
  - Kernel ELF build: passed.
  - OpenSBI + QEMU serial assertions: passed for FDT memory parsing, Sv39 W^X, trap save/restore, timer preemption, U-mode timer resume, IPC, Map/Protect/Unmap, Yield and Exit.
  - QEMU is stopped by the 15-second smoke-test timeout after required markers; this is not long-duration stability evidence.
  - [Kernel ELF/binary/QEMU log artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688775193/artifacts/14150434921).

- [VisionFive 2 Kernel Readiness #43688775224](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688775224): **SUCCESS**; strict host build and **16/16 CTest** passed. No physical board boot is claimed.

- [Win32 Gaming Gravity Backend Bridge #43688775230](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688775230): **SUCCESS**; host bridge CTest **2/2**, RV64 adapter cross-compile passed, disassembly contains **8 ecall** instructions.

### Core / SDK validation

- [Core #43688540402](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688540402): **SUCCESS** on the immediately preceding revision.
  - Root CTest **16/16**.
  - Xzuff Go generated a 44-byte `.xexe` returning status 42; host loader validation passed.
  - Gravity NDK object compile and `ecall` disassembly check passed.
  - Android ARM64 Xzuff Go / xgo cross-build and package passed.
  - Luau build and package smoke passed; JavaScript **2/2**, Lua **2/2**, OpenXzuff3D **1/1**, render verification passed.
  - [Core smoke artifacts](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688540402/artifacts/14150619731).

- [Core #43688775201](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688775201): **SUCCESS** on the latest shared-SDK/Win32 integration code commit. Root CTest **16/16**, 44-byte `.xexe` return-42 fixture and host loader validation, Gravity NDK object build plus `ecall` disassembly, Android ARM64 Xzuff Go cross-build/package, Luau build/source/package smoke, JavaScript **2/2**, Lua **2/2**, OpenXzuff3D **1/1**, render smoke and image verification passed. [Latest Core smoke artifacts](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688775201/artifacts/14152110000).

## Remaining work

- Full FDT device-node discovery, parent cell inheritance, `ranges`, translated MMIO `reg`, clocks, resets and interrupt routing.
- HAL MMIO validation, UART initialization/receive/flush, PLIC routing and physical-board/LAVA testing.
- Dynamic process/thread execution with allocated trap frames and stacks, independent per-process Sv39 roots, VM teardown and safe user copy-in/copy-out.
- VFS/handle integration, persistent storage, mount namespaces, permissions and device object lifetime.
- Blocking IPC, wait queues and shared-memory channels.
- General `.xexe` linker/loader, PE32/PE32+ loader, Win32 API/DLL compatibility and x86/x64-to-RV64 translation.
- Networking and graphics/audio/input service integration.

The current tests establish specific host, cross-build and QEMU smoke-test behavior only. They do not establish physical-board support, production stability, general Windows compatibility or 100% OpenXzuff Core completion.
