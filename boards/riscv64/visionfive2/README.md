# StarFive VisionFive 2 — RV64 bring-up

- SoC: StarFive JH7110 (RV64).
- UART0 DT binding: `starfive,jh7110-uart`, `snps,dw-apb-uart`.
- UART0 candidate MMIO: `0x10000000`; use the actual firmware FDT's `reg`.
- Register layout: `reg-io-width = 4`, `reg-shift = 2` in the JH7110 DTS.
- Dependencies: UART core/APB clocks and resets must be enabled through the
  platform clock/reset provider before polling I/O.
- Interrupt controller: JH7110 PLIC compatible `starfive,jh7110-plic`,
  `sifive,plic-1.0.0`; discover context wiring and source count from FDT.
- Firmware: U-Boot's documented path uses OpenSBI generic platform. U-Boot may
  be an intermediate stage; do not assume OpenSBI jumps directly to Gravity.
- Boot contract: S-mode entry receives the boot hart ID in `a0`; the FDT address in `a1` is passed to the kernel.

## Firmware artifact

The OpenSBI RV64 workflow builds a separate OpenSBI v1.9 `fw_dynamic` artifact.
It does not include SPL, U-Boot, board DTB or Gravity Kernel.

## Bring-up gates

1. Select the correct board-revision DTB and boot chain.
2. Parse FDT compatible, status, reg, clocks, resets and interrupt-parent.
3. Initialize clock/reset dependencies, then UART and serial smoke test.
4. Verify SBI TIME and PLIC interrupt routing.
5. Record physical board revision and serial log before marking support.

**Status: firmware build configured; physical board boot not verified.**
