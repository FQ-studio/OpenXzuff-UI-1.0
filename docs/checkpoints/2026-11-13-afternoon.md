# Afternoon integration and verification checkpoint — 2026-11-13

## Repository state

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Working branch: `feature/win32-gaming-gravity-port`
- Open PR: [#12 — Win32 Gaming: define Linux-free Gravity-native port](https://github.com/FQ-studio/OpenXzuff-UI-1.0/pull/12)
- Source revision used for the latest RV64/QEMU verification: `8d1f689b11f76adeeb7804bf8d837c3f0f8b758d`.
- Base `main` remains at `6d39ac9c87c075f6f1105767c6d2acb49710136f`; this work has **not** been merged.

## Integration present on the branch

### Gravity Kernel / RV64

- Mixed S-mode and U-mode execution with saved trap frames and per-task supervisor trap stacks.
- Round-robin scheduling, SBI TIME preemption, task Yield and Exit paths.
- Scalar non-blocking IPC through task mailboxes.
- Sv39 kernel/user roots, section-aware kernel W^X mappings and U-mode isolation checks.
- A bounded 2 MiB heap for the single built-in U-mode probe task, with MapMemory, ProtectMemory and UnmapMemory syscall paths.
- Unmap preflights the whole range, checks allocator ownership, clears mappings, scrubs frames and returns pages to the physical allocator.
- This is still a bring-up kernel, not a general process/VM implementation.

### Win32 Gaming Gravity bridge

- Versioned C ABI v1 host contract and host-side syscall-hook shim.
- RV64 adapter uses real `ecall` instructions for Exit, Send/ReceiveMessage, MapMemory, Yield, UnmapMemory and ProtectMemory.
- Host tests validate the ABI and hook dispatch; a separate RV64 cross-compile checks the U-mode adapter.
- This is an integration slice only. There is no PE loader, x86/x64 translator, full Win32 API/DLL runtime, dynamic task creation, VFS handle backend or Windows game execution.

### Driver / DTB / HAL targets

The target order is recorded in `boards/riscv64/DRIVER_HAL_MATRIX.md`:

1. **SiFive HiFive Unmatched / FU740-C000** — UART candidates `sifive,fu740-c000-uart` and `sifive,uart0`; PLIC `sifive,plic-1.0.0`.
2. **StarFive VisionFive 2 / JH7110** — UART candidates `starfive,jh7110-uart` and `snps,dw-apb-uart`; PLIC candidates `starfive,jh7110-plic` and `sifive,plic-1.0.0`.
3. **Sipeed Lichee Pi 4A / TH1520** — UART `snps,dw-apb-uart`; PLIC `riscv,plic0`.

Current implementation boundary:

- The Rust FDT parser reads RAM ranges, the FDT memreserve map and `timebase-frequency`. Its exact NUL-delimited `has_compatible` helper is host-tested.
- FDT discovery does not yet return a node path or resolve `status`, inherited bus cell widths, `ranges`, MMIO translations, clocks, resets or interrupt routing.
- `SiFiveUart` and `Ns16550Uart` are compiled into `xzuff_os_base`; current code is bounded transmit polling only. It is not yet selected or initialized from a validated FDT node.
- `sys/hal` and `sys/driver` remain scaffolds. No physical UART/PLIC driver or physical-board boot is claimed.
- `drivers/serial/README.md` was aligned with the board matrix and now records these limits and the required integration order.

## Verified CI evidence

### RV64 cross-build and QEMU

[RV64 S-Mode Kernel run #241](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378365921) and [PR check #242](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378375169) completed successfully for source revision `8d1f689`.

- RV64 adapter cross-compile: passed.
- Rust host allocator/FDT/scheduler/IPC/VM policy tests: **36 passed, 0 failed**.
- RV64 kernel ELF build: passed.
- OpenSBI v1.3 + QEMU `virt`, one hart: passed the workflow's serial assertions.
- Serial assertions included S-mode trap save/restore, scheduler context switching, timer preemption, U-mode saved-frame resume, U-mode syscall/IPC, Sv39 Map/Protect/Unmap, W^X rejection and Exit.
- QEMU was intentionally stopped by the workflow's 15-second timeout after the required markers appeared. This is a smoke test, not a long-duration stability test.
- [RV64 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378365921/artifacts/14015750722) contains the ELF/binary and serial log.

The U-mode timer probe's busy-wait was increased from 20,000,000 to 100,000,000 iterations after run #239 exposed a timing-sensitive failure: the probe could reach its verification ecall before a timer interrupt had preempted that U-mode context. Run #241 and PR check #242 passed with the longer observation window. Repeated stress runs are still needed before calling the timer path stable.

### Host/Core and bridge checks

[VisionFive 2 Kernel Readiness run #149](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378365955) and [PR check #150](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378375188) passed:

- Strict-warning host build passed.
- CTest: **16/16 passed**.
- Documented boot-register and FDT source checks passed.
- These are host/contract checks; they do not prove VisionFive 2 hardware boot.

[Win32 Gaming Gravity Backend Bridge PR check #81](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378375199) passed:

- Host bridge CTest: **2/2 passed**.
- RV64 adapter cross-compile passed.
- Disassembly contained **8** `ecall` instructions.

[OpenXzuff UI Runtime, OS Contracts and 3D run #259](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378102570) passed on the same Core source tree before the final assembly-only timer-probe adjustment:

- Root CTest: **16/16 passed**.
- Xzuff Go compiler and generated `.xexe` fixture validation passed.
- Xzuff Go / xgo Android ARM64 cross-build and toolchain packaging passed.
- Luau source/package smoke checks passed.
- JavaScript: **2/2**; standard Lua: **2/2**; OpenXzuff3D: **1/1**.
- Rendered-scene smoke test and image verification passed.
- New Core runs #261/#262 for revision `8d1f689` were still running when this checkpoint was written; consult their run pages before making a newer Core CI claim.

[OpenSBI RV64 Firmware run #7](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43378218067) completed successfully and builds a separate OpenSBI v1.9 `fw_dynamic` artifact. It does not package U-Boot, a board DTB or Gravity Kernel.

## Failures diagnosed and corrected

1. VisionFive readiness runs #140–#146 exposed stale exact-text assertions in the workflow: the README used different wording and split the FDT-address phrase across lines. The board boot contract was clarified and kept on one line; the workflow's readiness wording check was aligned with the actual status text. Runs #149 and #150 now pass.
2. RV64 run #239 failed the U-mode timer-preemption assertion even though the kernel ELF and host tests built. The U-mode probe's observation window was too short for a deterministic asynchronous-preemption check on every QEMU run. The loop was extended; runs #241 and #242 now pass. Keep the earlier failed run in CI history rather than relabeling it.

## Remaining work

1. Implement bounded FDT node discovery with parent-cell inheritance, `ranges`, `status`, `reg`, clocks/resets and interrupt-parent resolution.
2. Add validated MMIO/HAL capabilities and initialize the correct SiFive/DesignWare UART from the active firmware DTB.
3. Add receive/flush paths and time-based polling deadlines; add injectable MMIO tests.
4. Implement PLIC discovery and interrupt routing before storage/network/display drivers.
5. Add a second U-mode task with a separate address space, dynamic task/process lifecycle, safe copy-in/copy-out, VM teardown and SMP-safe synchronization.
6. Continue repeated QEMU stress runs, then validate physical HiFive Unmatched, VisionFive 2 and Lichee Pi 4A boards separately. LAVA and physical-board serial evidence are not present in this checkpoint.

## Warnings / caveats

- RV64 build reports one non-fatal dead-code warning for `fdt::has_compatible`; it is exercised by host tests but not yet called by the boot runtime.
- Vendored Luau `Ast.h` reports two non-fatal unused-parameter warnings.
- GitHub Actions also reports Node.js 20 deprecation warnings for third-party actions.
- No claim of production stability, general Win32 compatibility or physical-board support is made.
