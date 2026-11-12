# Work-session checkpoint — 2026-11-12 afternoon

## Repository and validated source

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- RV64 kernel source revision validated by CI: `0f8d39591b6b8b9a650fe3ea8c17a647b9984d7d`.
- The W^X design note and architecture-status update are documentation-only commits after that source revision.

## Changes made

1. `kernel/src/vm.rs`: added page-by-page permission verification. Every 4 KiB page in `.text`, `.rodata`, `.data` and `.bss` is checked in both the kernel and probe-user Sv39 roots.
2. The verifier models the deliberate U-mode exceptions inside `.text` (probe code, U-RX) and `.bss` (private user stack, U-RW); all remaining pages retain supervisor-only permissions.
3. The complete U-mode code and user-stack ranges are checked, as is the full page of each per-task supervisor trap stack in both roots.
4. The verifier checks kernel text RX, rodata R, data/BSS RW, and rejects W+X or U permission where not allowed.
5. `VmError::UserRegionTooLarge` is now used when the U-mode code or stack crosses the supported 2 MiB L1 region, removing its previous dead-code warning.
6. `kernel/src/main.rs` now passes linker section end boundaries and each supervisor trap-stack end to the verifier.
7. Added `docs/design/rv64-sv39-wx.md` and updated `docs/ARCHITECTURE_STATUS.md` with the actual mapping contract and validation scope.

## RV64 build and QEMU evidence

[RV64 S-Mode Kernel run #156](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461040) completed successfully on 2026-11-12 for source revision `0f8d39591b6b8b9a650fe3ea8c17a647b9984d7d`.

- Host Rust tests: **23 passed, 0 failed**.
- RV64 cross-build target: `riscv64imac-unknown-none-elf`; kernel ELF build succeeded.
- OpenSBI + QEMU `virt`, 256 MiB, one hart; all required serial assertions passed.
- No Rust compiler warnings were present in the successful run's log.
- QEMU was terminated by the workflow's 15-second smoke-test timeout after the required assertions had been emitted. This is not a long-duration stability test.

Relevant serial markers:

- `Sv39 bootstrap page tables: PASS; U-mode mappings isolated`
- `Sv39 kernel/user address spaces: PASS; distinct roots and U-only probe mappings`
- `Sv39 kernel W^X section permissions: PASS; text=RX rodata=R data=RW`
- `Per-task supervisor trap stacks: PASS; slots=16; supervisor-only`
- `U-mode timer preemption + saved-frame resume: PASS`
- `U-mode unknown syscall -ENOSYS: PASS`
- `U-mode Yield syscall context switch + Sv39 isolation: PASS`
- `U-mode Exit syscall + scheduler task exit: PASS`

The [run #156 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461040/artifacts/13920398891) contains the kernel ELF, raw binary and QEMU serial log.

## Host Core and board-contract checks

[VisionFive 2 Kernel Readiness run #49](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461210) completed successfully for source revision `0f8d3959`: CMake build, strict-warning compilation, CTest **14/14**, and source-level boot-contract checks passed. The workflow explicitly states that these checks do not prove physical board boot or full FDT correctness.

[OpenXzuff UI Runtime, OS Contracts and 3D run #164](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43124443998) is the latest completed Core host baseline at the time this checkpoint was drafted: root CTest **14/14**, Xzuff Go `.xexe` fixture, Android ARM64 NDK cross-build/package, Luau source/package smoke tests, JavaScript **2/2**, standard Lua **2/2**, OpenXzuff3D **1/1**, and rendered-scene verification passed. A newer Core run #166 for the full source revision `0f8d3959` was still in progress; its result must be checked before claiming the Core workflow has passed on this revision.

## Regression diagnosis

- RV64 runs #149 and #150 failed cross-build after intermediate W^X commits changed function signatures before all call sites were updated; they also exposed an unsafe-call error in the permission lookup path.
- Runs #151–#153 still failed on the unsafe call to `lookup_flags`. The helper was correctly marked unsafe, and run #154 then passed on the resulting source.
- Run #155 was an intermediate commit where `verify_isolation` had the new range-based signature but `main.rs` still used the old arguments. The immediately following commit updated the call site; final-source run #156 built and booted successfully.
- These intermediate failed runs remain failed in GitHub history; they are not relabeled. The current validated source revision is the one used by successful run #156.

## Implemented but bounded

- Sv39 remains a static identity map for one 1 GiB window; the kernel image and U-mode probe regions are constrained to the supported 2 MiB L1 region.
- The probe-user root is a single static test address space, not a dynamic per-process VM manager.
- Sixteen supervisor trap-stack slots are allocated and checked, but only one U-mode task and one hart are currently exercised.
- QEMU validation does not establish physical SiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot, LAVA results, SMP correctness, or production readiness.

## Next work

1. Create a second U-mode task with its own user stack and prove real U-to-U scheduling across separate trap-stack slots.
2. Expand GPR/CSR sentinel checks across ECALL, timer preemption, Yield and context restoration.
3. Add per-hart trap-stack ownership and SMP-safe scheduler/IPC synchronization.
4. Add negative tests for invalid U-mode addresses, malformed syscall arguments and S-mode trap paths.
5. Evolve the bootstrap into per-process Sv39 address spaces with page allocation, safe copy-in/copy-out and teardown.
6. Validate physical-board boot/timer behavior separately from QEMU and host contract tests.