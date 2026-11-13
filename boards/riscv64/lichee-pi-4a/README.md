# Sipeed Lichee Pi 4A — RV64 bring-up

- SoC: T-Head TH1520 (RV64, C910 cores).
- UART0 console compatible: `snps,dw-apb-uart`.
- UART0 candidate MMIO: `0xffe7014000`; firmware FDT is authoritative.
- System console is UART0, typically 115200 baud; electrical I/O is 1.8 V.
- UART clock dependency: TH1520 clock provider (commonly compatible
  `thead,light-fm-ree-clk`); do not access UART before required clock setup.
- Interrupt controller candidate compatible: `riscv,plic0`; discover the
  actual node, contexts and interrupt-parent from the supplied DTB.
- TH1520 reserved-memory ranges must be excluded from allocatable RAM.

## Bring-up order

1. Parse actual firmware FDT and reserved-memory before allocator initialization.
2. Resolve UART0 reg, status, clock and interrupt dependencies.
3. Implement DesignWare APB UART with board-specific clock/reset integration.
4. Verify serial output and SBI TIME; then PLIC routing.
5. Add storage and Ethernet only after their FDT dependencies are supported.

**Status: scaffold only — not boot-tested.**
