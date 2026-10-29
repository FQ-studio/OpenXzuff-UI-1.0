# StarFive VisionFive 2 — RV64 bring-up

- SoC: StarFive JH7110 (RV64).
- Console candidate: NS16550-compatible UART0 at MMIO `0x10000000`; expected console is commonly 115200 baud. Confirm from the actual firmware FDT before using it.
- Firmware: U-Boot's documented VisionFive 2 path uses OpenSBI's generic platform. The documented build settings include `FW_TEXT_START=0x40000000` and `FW_OPTIONS=0`.
- Boot contract: Gravity Kernel expects S-mode entry through a compatible firmware/boot stage, with boot hart ID in `a0` and the firmware-provided FDT address in `a1`. U-Boot may be an intermediate stage; do not assume OpenSBI jumps directly to Gravity Kernel.
- Device tree: parse the actual FDT passed by the boot stage first. Do not hard-code RAM size or assume all VisionFive 2 revisions have identical DTBs.

## Firmware CI artifact

The `OpenSBI RV64 Firmware` GitHub Actions workflow builds a separate OpenSBI v1.9 `fw_dynamic` artifact with the VisionFive 2 boot-address setting. The artifact is only OpenSBI firmware; it does **not** include SPL, U-Boot, a board DTB, or Gravity Kernel.

## Bring-up checklist

1. Build the VisionFive 2 OpenSBI artifact in CI.
2. Verify the intended boot chain and board revision (v1.2a/v1.3b DTB selection can differ).
3. Confirm the kernel receives the expected `a0`/`a1` boot parameters and parses the supplied FDT safely.
4. Verify UART output, then timer/interrupt-controller initialization and multi-hart behavior.
5. Record serial logs and repeat the boot smoke test before marking the board supported.

**Status: firmware build configured; board boot is not yet verified.**
