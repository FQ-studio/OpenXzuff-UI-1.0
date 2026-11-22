# 2026-11-23 — PLIC S-mode context selection

## Scope

Resolve PLIC `interrupts-extended` CPU interrupt-controller phandles to architectural hart IDs and select the S-mode external-interrupt context for the boot hart supplied by OpenSBI in `a0`.

## Changes

- Added CPU-node recognition for `/cpus/cpu@*` entries.
- Require explicit `/cpus/#address-cells` (one or two cells) and `#size-cells = <0>`.
- Resolve hart IDs from each CPU node's `reg`; unit-address text is not used as the hart ID.
- Require the referenced child controller to be enabled, marked `interrupt-controller`, compatible with `riscv,cpu-intc`, and to declare exactly one interrupt cell.
- Added `FdtPlicContext` and `find_plic_context_for_hart`.
- Preserve original PLIC context indexes when OpenSBI masks an M-mode entry with `0xffffffff`.
- Early boot validates the selected context's threshold and claim/complete register addresses before reporting it.
- QEMU workflow asserts the one-hart `virt` selection: hart 0, context 1, cause 9.

## Regression coverage

- Two-hart context list selects hart 0 at context 1 and hart 1 at context 3.
- OpenSBI-masked M-mode context does not shift the S-mode context index.
- Duplicate S-mode contexts for one hart are rejected.
- A hart without an S-mode PLIC context returns `None`.

## Verification

CI is triggered by this checkpoint commit. Host tests, RV64 cross-compilation, kernel ELF build, OpenSBI/QEMU serial assertions, VisionFive 2 host checks and Core workflow are pending for this exact revision. Earlier PR #15 evidence is not reused as proof for this change.

## Boundary

This change selects and validates a context only. It does not configure PLIC priorities, enable source bits, perform volatile MMIO claim/complete, set `sie.SEIE`, or prove end-to-end external-interrupt delivery. No physical-board boot is claimed.
