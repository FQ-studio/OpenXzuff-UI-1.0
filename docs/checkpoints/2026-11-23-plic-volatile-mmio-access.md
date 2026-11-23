# 2026-11-23 — PLIC volatile MMIO register access

## Revision and scope

- Branch: `feat/fdt-ranges-root-translation`
- Code revision verified by RV64 Actions: `62281a5a218cba44b82cbc46f3c76baa02864723`
- PR #15 remains open and targets `feature/win32-gaming-gravity-port`; it is not merged into `main`.

## Implementation committed

- `kernel/src/plic.rs`: added `PlicMmio`, a bounds-checked volatile register access layer over a validated `PlicLayout`.
- Register pointer derivation revalidates the absolute address against the translated `MmioRegion`, width and alignment before dereference.
- RV64 reads/writes are bracketed by `fence iorw, iorw`.
- API now includes priority read/write, threshold read/write, per-context source enable/disable, claim and complete.
- Constructor is `unsafe`: caller must guarantee the complete region is identity-mapped supervisor RW and remains mapped.
- `kernel/src/main.rs`: after Sv39 mapping succeeds, the boot path uses the selected boot-hart S-mode context to read/write back PLIC source-1 priority and the context threshold. It does not enable sources or globally enable external interrupts.
- `.github/workflows/rv64-s-mode-kernel.yml`: QEMU serial assertion requires the actual PLIC register readback probe.

## Exact-head verification

RV64 S-Mode Kernel PR run [#45124305056](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124305056): **SUCCESS** on head `62281a5`.

- Host FDT/MMIO/PLIC/kernel tests: **112 passed, 0 failed**.
- RV64 release kernel ELF build: success; output identified as statically linked ELF64 RISC-V.
- RV64 Win32 Gravity ecall adapter cross-compile: success.
- OpenSBI + QEMU smoke: success. Serial output includes:
  - `FDT PLIC S-mode context: PASS; hart=0; context=1; cause=9`
  - `PLIC MMIO access: PASS; context=1; priority1=0; threshold=7`
  - existing UART/PLIC Sv39 mappings, timer preemption, context switching, IPC and U-mode Sv39 assertions.
- Artifact ID `14794544741` includes the kernel ELF/binary, QEMU serial log and DTB/DTS.
- QEMU ended through the workflow's expected 15-second timeout after required assertions; this is not a long-duration stability test.

Additional exact-head checks:
- VisionFive 2 run [#45124305083](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124305083): **SUCCESS**, 16/16 host CTests and board contract checks only; no physical-board boot.
- Win32 Gravity bridge run [#45124305155](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124305155): host bridge job and RV64 adapter cross-compile both succeeded.
- OpenXzuff UI Runtime/Core run [#45124305047](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124305047): still running at checkpoint time. Root OS-contract CTest passed 16/16; Gravity RV64 NDK build passed; Android ARM64 xgo cross-compile was in progress. Do not claim the full Core workflow passed until the run completes.

## Verification boundary

The QEMU smoke probe performs real volatile read/write on the mapped PLIC priority-1 and selected-context threshold registers. The current QEMU probe does not enable a source, set `sie.SEIE`, read the claim register, complete an interrupt, or demonstrate a delivered external interrupt.

Host backing-memory tests cover:
- priority and threshold read/write;
- enable bitmap read/modify/write preserving unrelated bits;
- claim and complete address operations;
- invalid source/context rejection.

Those tests validate the driver logic against ordinary host memory; they do not emulate PLIC hardware. `set_source_enabled`, `claim` and `complete` remain unused by the boot path. The enable-bitmap read/modify/write API requires caller-side serialization if multiple harts can update the same word concurrently.

The UART is still discovered and mapped, not initialized through a concrete UART driver. No physical HiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot is established.

The RV64 build currently emits 32 Rust dead-code/unused warnings. The new source-enable/claim/complete methods account for some of the intentionally not-yet-integrated API warnings; do not suppress warnings globally.


## Follow-up head and current checks

The branch head advanced to `c559082f5d60b7b1d6c4ade3bb78c921b58e3a2d` after this checkpoint. That follow-up changes only PLIC module/safety/concurrency documentation; it does not alter executable register-access logic.

The immediately preceding code-bearing revision is covered by RV64 run [#45124585200](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124585200): **SUCCESS**, 112/112 host tests, RV64 ELF, and QEMU readback `priority1=0; threshold=7`. The exact current PLIC-code head `c559082` rerun [#45124724649](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124724649) subsequently completed **SUCCESS**: 112/112 host tests, RV64 ELF build and QEMU PLIC readback `priority1=0; threshold=7`. The final branch head is now `600eeb9`, a checkpoint-documentation-only commit on top of that verified code; it does not change executable code.

Current-head supporting runs:
- VisionFive 2 [#45124724673](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124724673): host smoke and board contract checks completed successfully.
- Win32 Gravity bridge [#45124724567](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124724567): host and RV64 adapter jobs completed successfully.
- OpenXzuff UI Runtime/Core [#45124724591](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45124724591): still in progress at the Luau runtime build step; do not claim full Core completion yet.

## Next work

1. Confirm the exact-head OpenXzuff Runtime/Core workflow.
2. Parse the UART interrupt specifier and implement the narrow polling-console HAL for the selected UART IP.
3. Add a QEMU-triggered external-interrupt round-trip using a known device source, then connect PLIC claim/complete and S-mode trap dispatch.
4. Keep FDT discovery closed unless a new regression is observed; next progress must be in device-driver execution, not repeated parser work.
