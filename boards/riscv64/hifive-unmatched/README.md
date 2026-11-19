# SiFive HiFive Unmatched — RV64 bring-up

- SoC: SiFive FU740-C000 (RV64).
- UART compatible list: `sifive,fu740-c000-uart`, `sifive,uart0`.
- UART0 candidate MMIO: `0x10010000`; confirm from firmware FDT.
- Interrupt controller: SiFive PLIC compatible `sifive,plic-1.0.0`;
  discover contexts and source count from FDT.
- Firmware: inspect the actual board boot chain and SBI services before
  selecting the kernel entry contract.
- Device tree: consume the actual firmware-provided FDT; RAM size and
  peripheral resources must not be hard-coded.

## Bring-up order

1. Parse memory/reserved-memory and hart/timebase information from FDT.
2. Discover UART compatible/reg and implement the SiFive UART polling driver.
3. Discover PLIC contexts and wire external interrupt dispatch.
4. Verify SBI TIME, then SD/MMC and Ethernet dependencies.

**Status: scaffold only — not boot-tested.**
