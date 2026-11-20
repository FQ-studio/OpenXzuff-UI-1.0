# Gravity Kernel FDT PLIC discovery checkpoint — 2026-11-20

Branch: `feat/fdt-ranges-root-translation`
Initial implementation head: `ed67a20f0bbcb4e6013b699182a44a2ad00822d9`
PLIC geometry code: `e7f1df4a64e65ac04e294999a5ebf97c8934cef2`
QEMU FDT fixture correction: `480953866aa755c49ac01e366f6fca9db979a348`
Workflow assertion correction: `203db21dd982258c130e336b66ad2ea8cf64c994`
Board matrix correction: `2a71817b4c60fa24c8f540ff2db58545390db8c3`

## Implemented

- Added `kernel/src/plic.rs`, a `no_std` descriptor for standard RISC-V PLIC register geometry.
- Validates priority, pending, per-context enable, threshold and claim/complete register windows against an already translated `MmioRegion`.
- Exposes checked register addresses and pending/enable bit masks for source IDs 1 through the supplied `riscv,ndev` value.
- Rejects invalid source/context counts and IDs, arithmetic overflow, and undersized resources.
- Added FDT PLIC discovery in `kernel/src/fdt.rs`: exact compatible matching, inherited availability, `riscv,ndev`, `interrupts-extended` parsing, phandle resolution, interrupt-controller availability and `#interrupt-cells` validation.
- Recognizes SiFive/QEMU, StarFive JH7110 and generic RISC-V PLIC compatible strings in both node selection and metadata validation.
- Counts resolved PLIC interrupt contexts and reports the QEMU virt candidate during boot.
- Adds host fixtures for QEMU virt geometry, StarFive-only compatible discovery, unknown phandles, missing `#interrupt-cells`, disabled PLIC nodes, malformed context lists and invalid source counts.
- Adds QEMU serial assertions for PLIC discovery and register-window bounds.

## Verification and diagnosed failures

- Earlier revision `eeecc4c` completed the RV64 kernel build and QEMU smoke run before the PLIC discovery integration.
- Revision `ed67a20` failed the host allocator-test compile because the new PLIC integration test module omitted imports for `MmioRegion` and `PlicLayout`. The RV64 adapter cross-compile succeeded; kernel ELF build and QEMU were skipped.
- Added the missing imports in commit `ef18594`.
- The first test run after the import fix executed 96 tests: 95 passed and the StarFive-only compatible test failed. Diagnosis: node selection recognized `starfive,jh7110-plic`, but the second PLIC metadata scanner still accepted only SiFive and generic RISC-V strings.
- Added StarFive matching to the metadata scanner in commit `d7cf46d`.
- The next QEMU run passed host tests and kernel ELF build, then failed at a workflow assertion. The actual QEMU FDT reported `riscv,ndev=0x5f` (95), base `0x0c000000`, size `0x00600000` (6 MiB), and two contexts. The old assertion incorrectly expected 96 sources and a 4 MiB window.
- Updated PLIC geometry tests, FDT fixtures, workflow serial assertion and board matrix to match the observed QEMU FDT. The last valid source ID is 95; its enable bit is bit 31 of word 2.
- GitHub Actions for the corrected source and latest documentation commits are still running. Do not mark the updated suite PASS until the current RV64 run completes through QEMU and all required serial assertions.

## Verification boundary

- The PLIC descriptor and FDT discovery do not map MMIO, set Sv39 device attributes, perform volatile reads/writes, configure priorities, claim/complete interrupts or enable interrupt delivery.
- `interrupts-extended` phandles and specifier widths are validated and context entries counted, but the active hart's supervisor-external context is not selected.
- No QEMU PLIC interrupt-delivery test or physical-board interrupt test is claimed.
- UART is discovered and bounds-checked only; it is not initialized by this path.
- No physical HiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot is established.

## Next

1. Complete the latest RV64 host tests, kernel ELF build and QEMU assertions on the corrected QEMU PLIC values.
2. Resolve the PLIC supervisor-external context for the boot hart using CPU topology and interrupt specifiers.
3. Add a platform interrupt-controller interface and test real claim/complete flow in QEMU.
4. Map device MMIO with explicit Sv39 device attributes before exposing volatile register access.
5. Connect PLIC delivery to the kernel trap/IRQ dispatch path and validate timer/PLIC interaction.
