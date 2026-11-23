# 2026-11-24 — PLIC trap dispatch foundation

## Repository state before this change

- Repository: FQ-studio/OpenXzuff-UI-1.0
- Branch: feat/fdt-ranges-root-translation
- Parent revision: 4cb5b2099f4fc3d5dbe99ad0d3fcb11ada87e3bd
- PR #15 remains open and targets feature/win32-gaming-gravity-port.
- main is unchanged.
- Exact-parent RV64, VisionFive 2 readiness, Win32 bridge and Core workflows completed successfully on 2026-11-23. QEMU verified PLIC context selection and priority/threshold volatile readback, but did not deliver an external interrupt.

## Implemented

- Added kernel/src/irq.rs with a fixed 1024-slot handler table and a single-hart S-mode controller.
- Installation sets the selected context threshold to zero and verifies volatile readback.
- Boot-time registration writes priority, stores the callback, then enables its source.
- Dispatch claims one source, invokes its handler, masks unowned/unhandled sources, and completes the claim.
- Added S-mode and U-origin trap handling for supervisor external interrupt cause 9.
- Runtime startup enables sie.SEIE only after at least one device handler is registered. No handler is registered yet, so external interrupts remain disabled.
- Added host-backed tests for registered dispatch, unregistered-source masking/bit preservation, completion, and idle claim.
- QEMU boot assertions verify dispatcher installation and threshold-zero readback.

## Verification boundary

This is dispatch infrastructure, not end-to-end interrupt delivery. QEMU installs the controller but deliberately registers no device handler and does not enable sie.SEIE. Host tests use ordinary RAM as a fake register window; they validate control flow and register sequencing, not PLIC hardware semantics. UART interrupt parsing/driver support, QEMU serial-input injection, and an actual external-interrupt round-trip remain outstanding.

SMP-safe registration and enable-word locking are not implemented; this controller is explicitly single-hart.
