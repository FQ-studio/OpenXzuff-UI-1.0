# Gravity Kernel PLIC register-layout checkpoint — 2026-11-20

Branch: `feat/fdt-ranges-root-translation`
Base head: `102aac785ea2f192e537ee18b3ad0e740aabfff4`

## Implemented

- Added `kernel/src/plic.rs`, a no_std descriptor for the standard RISC-V PLIC register geometry.
- Validates priority, pending, per-context enable, threshold and claim/complete register windows against an already translated `MmioRegion`.
- Exposes checked register addresses and pending/enable bit masks for source IDs 1 through the supplied `riscv,ndev` value.
- Rejects source counts outside 1..=1023, zero context counts, invalid source/context IDs, arithmetic overflow and undersized resources.
- Adds host regression fixtures for QEMU virt geometry (96 sources, two contexts) and boundary rejection.

## Verification boundary

- The descriptor consumes source/context counts supplied by a future validated FDT interrupt parser; it does not discover PLIC nodes or resolve `interrupts-extended` contexts itself.
- It does not map MMIO, set Sv39 device attributes, perform volatile reads/writes, claim interrupts, configure priorities or enable interrupt delivery.
- No QEMU PLIC interrupt-delivery test or physical-board interrupt test is claimed by this checkpoint.
- GitHub Actions for this revision must complete before the new tests are marked PASS.

## Next

1. Parse PLIC `riscv,ndev`, phandles, `#interrupt-cells` and `interrupts-extended` from the actual FDT.
2. Resolve supervisor external-interrupt context by hart ID and cause.
3. Add a platform interrupt-controller abstraction and test real claim/complete flow in QEMU.
4. Only then connect PLIC delivery to the kernel trap/IRQ dispatch path.
