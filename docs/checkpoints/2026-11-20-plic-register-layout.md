# Gravity Kernel FDT PLIC discovery checkpoint — 2026-11-20

Branch: `feat/fdt-ranges-root-translation`
Current code/test head: `c8197670be4a7b7e4dac102649cf319f3daf16bb`

## Implemented

- FDT device discovery checks exact compatible-list membership and inherited availability.
- Device `reg` resources are translated through each ancestor bus `ranges` property, with missing/ambiguous windows, malformed tuples, address-width overflow and out-of-window resources rejected.
- `MmioRegion` validates known non-zero translated physical windows, minimum extent, power-of-two alignment, exclusive-end overflow and complete 8/16/32/64-bit register access bounds. It does not map or access MMIO.
- `PlicLayout` validates priority, pending, per-context enable, threshold and claim/complete windows, and calculates checked register addresses and pending/enable masks.
- FDT PLIC discovery recognizes `sifive,plic-1.0.0`, `starfive,jh7110-plic` and `riscv,plic0`; it validates `riscv,ndev`, parses `interrupts-extended`, resolves phandles to available interrupt-controller nodes with valid `#interrupt-cells`, and counts contexts.
- Host fixtures cover nested ranges, address-width and overflow rejection, MMIO bounds, PLIC geometry, SiFive/StarFive/generic-only compatible paths, unknown phandles, missing interrupt-cell metadata, disabled PLIC nodes, malformed context lists and invalid source counts.
- QEMU serial checks use observed QEMU virt FDT values: PLIC base `0x0c000000`, size `0x00600000`, `riscv,ndev=95` (`0x5f`) and two contexts.

## Diagnosed and corrected

1. Initial PLIC test compilation failed because the FDT test module omitted `MmioRegion` and `PlicLayout` imports. Fixed in `ef18594`.
2. The StarFive-only compatible regression then failed because the metadata scanner had not been updated alongside the node selector. Added `starfive,jh7110-plic` to both in `d7cf46d`.
3. QEMU boot and host tests passed, but the QEMU workflow assertion failed because an assumed 96-source/4-MiB PLIC did not match the actual firmware FDT. Updated the geometry fixture, register expectations, workflow assertion and board matrix to 95 sources/6 MiB. The observed last source ID is 95; its enable bit is bit 31 of word 2.
4. Added a generic-only `riscv,plic0` fixture in `c819767` to cover the Lichee Pi 4A/TH1520 compatible path.

## Verification evidence

- [RV64 S-Mode Kernel PR run #44686946339](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44686946339), head `65308cc`: **SUCCESS**. Host allocator tests **96/96**; RV64 adapter cross-compile; kernel ELF build; OpenSBI/QEMU smoke and required serial assertions. The PLIC assertion matched `ndev=5f; contexts=2; root=c000000; size=600000`. [ELF, binary and QEMU serial artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44686946339/artifacts/14606737934).
- [RV64 S-Mode Kernel PR run #44686929090](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44686929090), head `55a69cf`: **SUCCESS**, 96/96 host tests, RV64 kernel build and QEMU assertions.
- [VisionFive 2 PR run #44687549543](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44687549543), head `c819767`: **SUCCESS** — host smoke build, strict-warning build, CTest and board-contract checks. This is not a physical-board boot.
- [Win32 Gravity Bridge PR run #44687549527](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44687549527), head `c819767`: **SUCCESS** — host bridge tests and RV64 adapter cross-compile.
- [OpenXzuff UI Core PR run #44686434866](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44686434866), head `9f45303`: **SUCCESS** — root CTest 16/16, Xzuff Go/`.xexe`, Gravity RV64 NDK, Android ARM64 xgo package, Luau source/package smoke, JavaScript 2/2, standard Lua 2/2, OpenXzuff3D 1/1 and rendered-image verification.
- Current-head [RV64 run #44687549591](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44687549591) is still running at the Rust target/QEMU installation step. It contains the added generic-only PLIC test; therefore the expanded 97-test suite is **not yet claimed PASS**.
- Current-head [Core run #44687549588](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44687549588) is still running at Luau runtime build. The earlier complete Core success above is recorded separately.

## Verification boundary

- FDT PLIC parsing validates phandles/specifier widths and counts contexts, but does not map each CPU interrupt-controller phandle to its parent hart ID or select the boot hart's S-mode external-interrupt context.
- PLIC geometry validation does not map MMIO, set Sv39 device-memory attributes, perform volatile IO, configure priorities, claim/complete interrupts or enable interrupt delivery.
- QEMU validates discovery and register geometry, not a real PLIC interrupt round-trip. UART is discovered and bounds-checked only; it is not initialized.
- FDT clock/reset providers, PLIC trap/IRQ dispatch, physical board boot and LAVA validation remain outstanding.
- No physical HiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot is established. QEMU's 15-second assertion-based smoke test is not a long-duration stability test.

## Next

1. Complete current-head RV64 run #44687549591 and verify the generic-only PLIC test plus QEMU.
2. Complete current-head Core run #44687549588 and confirm all required checks for PR #15.
3. Resolve the boot hart's supervisor-external PLIC context and connect claim/complete to trap dispatch.
4. Add safe Sv39 device-MMIO mapping before any volatile register access.
5. Continue per-process address spaces, dynamic process execution, blocking IPC, persistent storage and full executable/runtime integration.
