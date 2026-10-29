# SiFive HiFive Unmatched — RV64 bring-up

- SoC: SiFive FU740-C000 (RV64)
- Console candidate: SiFive UART0, MMIO `0x10010000`, compatible `sifive,uart0`
- Firmware: board firmware/OpenSBI may provide the runtime environment; inspect the actual boot chain before choosing the kernel entry contract.
- Device tree: consume the **actual firmware-provided FDT** first. RAM size and available peripherals must be read from that FDT, not assumed from a fixed-size local blob.
- Required next steps: FDT parser, SiFive UART driver, interrupt-controller/PLIC discovery, timer/SBI contract, then SD/MMC and Ethernet.

**Status: scaffold only — not boot-tested.** No prebuilt DTB is claimed by this file.