# StarFive VisionFive 2 — RV64 bring-up

- SoC: StarFive JH7110 (RV64)
- Console candidate: UART0 at MMIO `0x10000000`; upstream board DTS uses `/soc/serial@10000000:115200`.
- Firmware: the usual U-Boot S-mode path requires prior-stage M-mode firmware (commonly OpenSBI). OpenXzuff UI must define and verify its own entry/boot contract.
- Device tree: consume the **actual firmware-provided FDT** first; don't hard-code RAM size or assume every VisionFive 2 board has identical memory.
- Required next steps: NS16550-compatible UART driver, JH7110 clock/reset discovery, PLIC/timer, then SD/MMC, PCIe/USB and Ethernet.

**Status: scaffold only — not boot-tested.** No prebuilt DTB is claimed by this file.