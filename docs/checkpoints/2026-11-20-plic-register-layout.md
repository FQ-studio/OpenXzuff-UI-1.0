# Gravity Kernel FDT PLIC discovery checkpoint — 2026-11-20

Branch: `feat/fdt-ranges-root-translation`
Implementation head before test-import fix: `ed67a20f0bbcb4e6013b699182a44a2ad00822d9`\n\nLatest code fix before this checkpoint update: `d7cf46dcea92014ddd7146387ead1372fcdf23c9`.

## Implemented

- Added `kernel/src/plic.rs`, a `no_std` descriptor for standard RISC-V PLIC register geometry.
- Validates priority, pending, per-context enable, threshold and claim/complete register windows against an already translated `MmioRegion`.
- Exposes checked register addresses and pending/enable bit masks for source IDs 1 through the supplied `riscv,ndev` value.
- Rejects invalid source/context counts and IDs, arithmetic overflow, and undersized resources.
- Added FDT PLIC discovery in `kernel/src/fdt.rs`: exact compatible matching, inherited availability, `riscv,ndev`, `interrupts-extended` parsing, phandle resolution, interrupt-controller availability and `#interrupt-cells` validation.
- Recognizes SiFive/QEMU, StarFive JH7110 and generic RISC-V PLIC compatible strings in both node selection and metadata validation.\n- Counts the resolved PLIC interrupt contexts and reports the QEMU virt candidate during boot.\n- Adds a StarFive-only compatible fixture to prevent selection/metadata matcher drift.
- Adds host fixtures for QEMU virt geometry, unknown phandles, missing `#interrupt-cells`, disabled PLIC nodes, malformed context lists and invalid source counts.
- Adds QEMU serial assertions for PLIC discovery and register-window bounds.

## Verification

- Earlier revision `eeecc4c` completed the RV64 kernel build and QEMU smoke run before the PLIC discovery integration.
- Revision `ed67a20` failed the host allocator-test compile because the new PLIC integration test module omitted imports for `MmioRegion` and `PlicLayout`. The RV64 adapter cross-compile succeeded; kernel ELF build and QEMU were skipped.
- Fixed the test imports in commit `ef18594`.
- GitHub Actions were triggered for `ef18594`; record final results only after the latest RV64, Core, VisionFive 2 and Win32 runs complete.

## Verification boundary

- The PLIC descriptor and FDT discovery do not map MMIO, set Sv39 device attributes, perform volatile reads/writes, configure priorities, claim/complete interrupts or enable interrupt delivery.
- `interrupts-extended` phandles and specifier widths are validated and context entries counted, but the active hart's supervisor-external context is not selected.
- No QEMU PLIC interrupt-delivery test or physical-board interrupt test is claimed.
- UART is still discovered and bounds-checked only; it is not initialized by this path.
- No physical HiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot is established.

## Next

1. Complete the `ef18594` Actions runs and fix any new regression before integrating this PR.
2. Resolve the PLIC supervisor-external context for the boot hart using CPU topology and interrupt specifiers.
3. Add a platform interrupt-controller interface and test real claim/complete flow in QEMU.
4. Map device MMIO with explicit Sv39 device attributes before exposing volatile register access.
5. Connect PLIC delivery to the kernel trap/IRQ dispatch path and validate timer/PLIC interaction.
