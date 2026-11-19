# Gravity Kernel + OpenXzuff UI Core checkpoint — 2026-11-19 afternoon

## Repository state

- Integration branch: `feature/win32-gaming-gravity-port`
- Integration branch head before this checkpoint: `922042d23def1b6bbbdae48d75eb8c176ac4be85`
- Stacked FDT/MMIO work: PR #15, branch `feat/fdt-ranges-root-translation`
- PR #15 head after the regression fix: `102aac785ea2f192e537ee18b3ad0e740aabfff4`
- PR #15 base: `feature/win32-gaming-gravity-port`
- PR #15 remains open and is not yet integrated into the integration branch.
- PR #12 remains open against `main`; no merge to `main` was performed.

## Work performed in this session

The current PR #15 adds:
- FDT compatible-node discovery with inherited availability checks.
- Device `reg` translation through each ancestor bus `ranges` to the root address space.
- Fail-closed handling for absent intermediate ranges, malformed/overlapping windows, address-width violations and arithmetic overflow.
- A no_std `MmioRegion` descriptor for known, non-zero translated resources.
- Explicit 8/16/32/64-bit register-address validation, alignment and full-window bounds.
- A QEMU assertion that the first UART register is within the translated window.

The MMIO descriptor only validates physical addresses. It does not map MMIO, set Sv39 device-memory attributes, perform volatile IO or initialize UART.

## Failure diagnosis and repair

On the previous PR #15 head `90c6a3d2f40a6ae2f1de6e8e8698640f252f491a`, RV64 run #44425102880 failed at host allocator tests:
- 85 tests ran.
- 84 passed, 1 failed.
- Failure: `mmio_register_access_must_fit_entirely_inside_window`.
- Actual result: `Err(MisalignedOffset)`.
- Expected result: `Err(OutOfBounds)`.
- Cause: `register_address()` checked alignment before verifying that the complete access fit in the window.

Fixes on PR #15:
- Commit `36eb8ea3f18562804a8c0a122a9ab669c29725fd`: perform checked access-end and window-bound checks before offset alignment.
- Commit `102aac785ea2f192e537ee18b3ad0e740aabfff4`: add an explicit unaligned-but-out-of-window regression case at offset `0xff`, width 4.

This makes out-of-window accesses consistently return `OutOfBounds`, including overflowing offsets, before alignment classification.

## Latest GitHub Actions evidence on PR #15 head 102aac785

Completed successfully:
- OpenXzuff UI Runtime, OS Contracts and 3D PR run #44471438100.
  - Root CTest: 16/16.
  - Xzuff Go compiler and `.xexe` fixture checks passed.
  - Gravity RV64 NDK example cross-build passed.
  - Android ARM64 xgo cross-build/package passed.
  - Luau build and source/package smoke passed.
  - JavaScript tests: 2/2.
  - Standard Lua tests: 2/2.
  - OpenXzuff3D tests: 1/1; rendered image verification passed.
- VisionFive 2 Readiness PR run #44471437953.
  - Host CTest: 16/16.
  - Strict-warning build and source/board-contract checks passed.
- Win32 Gravity Bridge PR run #44471434999.
  - Host bridge CTest: 2/2.
  - Real RV64 adapter cross-compile passed.

The equivalent push runs also completed successfully:
- Core #44471430944.
- VisionFive 2 #44471431299.

Still pending at checkpoint:
- RV64 S-Mode Kernel PR run #44471434893.
- RV64 S-Mode Kernel push run #44471430945.
- Both jobs were still at `Install Rust target and QEMU`; host allocator tests, kernel ELF build and QEMU had not yet run on head `102aac785`.
- Therefore the new MMIO regression fix is not yet claimed as passing, and no new QEMU result is claimed for this head.

## Existing kernel/runtime boundary

The current runtime remains a fixed three-task QEMU smoke configuration (two S-mode tasks and one U-mode test task). It is not yet a general dynamic process runtime. The user address-space implementation remains a single bounded bootstrap user heap, not independent per-process Sv39 roots.

The existing CI/QEMU tests cover the implemented boot, trap, scheduler, timer-preemption, syscall, IPC and memory-policy paths. They do not establish long-duration stability or physical-board boot.

## Next actions

1. Wait for RV64 run #44471434893 on head `102aac785`.
2. If host tests pass, verify RV64 adapter cross-compile, kernel ELF build and every required QEMU serial assertion.
3. Only after all relevant checks pass, merge PR #15 into `feature/win32-gaming-gravity-port` (not into `main`).
4. Verify the resulting integration-branch head and rerun PR #12 checks.
5. Continue with a real Sv39 MMIO mapping/device-memory contract, UART initialization/read/flush, clocks/resets and PLIC; do not treat the current descriptor as a driver.
6. Continue independent per-process page tables, dynamic task contexts, blocking IPC, persistent storage, executable loading and the remaining OS services.

## Evidence links

- PR #15: https://github.com/FQ-studio/OpenXzuff-UI-1.0/pull/15
- Failing run and diagnosis: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44425102880
- Latest RV64 pending PR run: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44471434893
- Latest Core PR run: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44471438100
- Latest VisionFive 2 PR run: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44471437953
- Latest Win32 Bridge PR run: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44471434999
