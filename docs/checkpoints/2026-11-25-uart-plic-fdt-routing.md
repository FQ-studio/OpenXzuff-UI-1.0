# 2026-11-25 — FDT UART-to-PLIC interrupt routing

## Scope

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feat/fdt-ranges-root-translation`
- Parent revision: `5f1f94df62d77ec2619d502fd37ee58a58c32006`
- PR #15 remains open and targets `feature/win32-gaming-gravity-port`.

## Implemented

- Preserve the discovered PLIC node phandle in `FdtPlicInfo`.
- Add `find_device_interrupt` for a single-cell PLIC interrupt on a selected compatible device.
- Resolve an explicit or inherited `interrupt-parent`.
- Require the selected parent to equal the discovered PLIC phandle and resolve to an available interrupt-controller with `#interrupt-cells = <1>`.
- Reject missing/duplicate metadata, zero phandles, wrong controller parents, zero source IDs, truncated or multi-cell specifiers, disabled nodes and ambiguous compatible matches.
- Add QEMU UART route diagnostics and a mandatory serial assertion for PLIC phandle 3, source ID 10.
- Add host regression fixtures for explicit and inherited parent routing, wrong/unknown parents and malformed widths.

## Verification boundary

This change resolves firmware routing metadata only. It does **not** register a UART handler, enable the UART IER or PLIC source bit, enable `sie.SEIE`, inject serial input, or prove an external interrupt claim/handler/complete round-trip. Those remain the next implementation step.

No physical-board boot or long-duration stability claim is made.
