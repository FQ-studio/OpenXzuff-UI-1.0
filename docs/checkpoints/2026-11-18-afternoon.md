# 2026-11-18 afternoon — Gravity Kernel / OpenXzuff UI Core integration

## Repository state at checkpoint

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Integration branch: `feature/win32-gaming-gravity-port`
- PR #12 is open against `main`; this checkpoint does not merge PR #12.
- Branch code head before this checkpoint: `acd593768b9e52fa416502efc582be52c444598c`.
- Stacked PR #15, `Translate FDT device resources through ancestor ranges`, targets this branch. It is **not yet merged**.
- PR #15 latest code head: `787d564245f202d42564a6121081f522673f75d5`.

## Work/review this session

- Reviewed PR #15 source and its host/QEMU evidence.
- The translated discovery API returns both raw parent-bus `reg` and root-translated `reg`; it applies ancestor `ranges`, treats an explicit empty `ranges` as identity, and fails closed on missing mappings, malformed tuples, out-of-window and ambiguous mappings.
- Kernel UART boot diagnostics now require translated resource discovery. This remains discovery only; no MMIO access or UART initialization is performed.
- Review found missing address-cell-width validation for resource endpoints and ranges window endpoints, plus checked parent-window overflow. Added checks in `kernel/src/fdt.rs`.
- Added three regression tests: child one-cell window overflow, identity resource crossing child width, and 64-bit parent-window overflow.
- First validation of the new tests was **77 passed, 1 failed** because the child-window-overflow fixture had one missing parent-address cell (malformed ranges tuple). The fixture was corrected in commit `787d564`. This first failure is not counted as a pass; the rerun is pending.

## Evidence

### Prior PR #15 code head `2bfca14636561fd9adb17a2207956b3699efbc5d`

- [RV64 S-Mode Kernel PR run #44201411279](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44201411279) — SUCCESS; 75 host tests, RV64 adapter and kernel ELF build, QEMU/OpenSBI assertions.
- [VisionFive 2 PR run #44201411010](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44201411010) — SUCCESS; CTest 16/16.
- [Win32 Gravity Bridge PR run #44201411456](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44201411456) — SUCCESS; host bridge and RV64 adapter jobs.
- [OpenXzuff Core PR run #44201411091](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44201411091) — SUCCESS on the earlier head; host/runtime smoke pipeline completed.
- QEMU serial evidence from [RV64 push run #44201389335](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44201389335) includes translated UART address `0x10000000`, size `0x100`, timer preemption, U-mode saved-frame resume, IPC, Yield/Exit and Sv39 assertions.

### Latest PR #15 head `787d564`

- [VisionFive 2 PR run #44251247177](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44251247177) — SUCCESS.
- [Win32 Gravity Bridge PR run #44251245058](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44251245058) — SUCCESS; host and cross-compile jobs.
- [RV64 S-Mode Kernel PR run #44251245074](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44251245074) — IN PROGRESS at checkpoint; latest head tests and QEMU are not verified yet.
- [OpenXzuff Core PR run #44251245070](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44251245070) — IN PROGRESS at checkpoint; latest head Core result not verified yet.
- Duplicate push runs were also pending. Do not report latest head as fully green until the PR runs complete.

## Integration decision

PR #15 is not merged because its latest RV64/QEMU and Core checks are pending. The prior translation implementation has green evidence, but the address-width hardening patch still requires rerun proof. Merge only after the latest checks complete successfully.

## Remaining scope

- FDT `ranges` translation is in a review PR; MMIO mapping, UART init/read/flush, clocks/resets, PLIC routing and real driver operation remain incomplete.
- Physical HiFive Unmatched, VisionFive 2 and Lichee Pi 4A boot; LAVA.
- Dynamic task stacks/frames and independent per-process Sv39 roots, safe user-copy, VM teardown.
- Blocking IPC, wait queues, VFS/handles and persistent storage.
- General `.xexe` loader/linker, PE32/PE32+, Win32 API/DLL and x86/x64-to-RV64 translation.
- Full GUI, input/audio and service integration.

No 100% completion, production stability, physical-board boot or Windows game execution is claimed.
