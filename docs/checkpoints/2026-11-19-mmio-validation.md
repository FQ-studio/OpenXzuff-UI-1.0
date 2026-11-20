# Gravity Kernel MMIO validation checkpoint — 2026-11-19

Branch: `feat/fdt-ranges-root-translation`

Checkpoint head before this document: `f07bd78fa982ed44913286bb1e477aa0a5999432`

## Implemented

- Added `kernel/src/mmio.rs`, a no_std checked MMIO resource descriptor.
- The descriptor accepts a translated FDT resource with a known non-zero size, checks minimum extent, power-of-two base alignment and exclusive-end overflow.
- Register-address calculation checks supported access widths (8/16/32/64-bit), natural alignment and complete in-window bounds.
- Added subrange containment checks.
- The descriptor returns physical addresses only. It does not create a virtual mapping, set device-memory page attributes, expose volatile pointers or perform register IO.
- RV64 boot diagnostics now validate that the first 32-bit UART register fits in the translated resource window before reporting the candidate as validated.
- A size-less compatible candidate is skipped so the compatible fallback list can continue.
- Added host regression coverage and a QEMU serial assertion for the MMIO validation marker.
- Updated the RV64 driver/HAL matrix to document the exact validation boundary.

## Verification

At checkpoint creation, the Actions runs triggered by these commits were still queued or in progress. No PASS claim is made here. Record the final run IDs and conclusions only after the latest head completes:

- RV64 S-Mode Kernel push / PR checks
- VisionFive 2 Kernel Readiness checks
- OpenXzuff UI Runtime, OS Contracts and 3D
- Win32 Gaming Gravity Backend Bridge

## Remaining limitations

- MMIO is not mapped into Sv39 and no volatile access is performed.
- UART is not initialized by this new descriptor.
- FDT clock/reset properties and PLIC interrupt routing remain unimplemented.
- No physical-board boot is established by QEMU or host contract tests.
