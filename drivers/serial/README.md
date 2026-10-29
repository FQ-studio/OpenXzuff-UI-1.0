# RV64 Serial Drivers

This directory tracks OpenXzuff UI's **native** early-console drivers. These are not Linux drivers and must not depend on Linux APIs.

## Board-to-UART match table

| Board | SoC | UART compatible | Known UART0 MMIO base | Driver state |
|---|---|---|---:|---|
| SiFive HiFive Unmatched | SiFive FU740-C000 | `sifive,uart0` | `0x10010000` | Planned; register-level implementation and hardware test pending |
| StarFive VisionFive 2 | StarFive JH7110 | `ns16550a` / StarFive UART binding | `0x10000000` | Planned; register-level implementation and hardware test pending |
| Sipeed Lichee Pi 4A | T-Head TH1520 | `snps,dw-apb-uart` | `0xffe7014000` | Planned; register-level implementation and hardware test pending |

The addresses and compatible strings are discovery hints, not proof that a driver works. Prefer the firmware-provided FDT's `stdout-path` and UART node over hard-coded board assumptions. Validate the register layout, access width, clock, interrupt and pin configuration before enabling a driver.

## Implementation order

1. FDT parser extracts `compatible`, `reg`, `reg-shift`, `reg-io-width`, `clock-frequency`, and `stdout-path`.
2. Match SiFive UART and DesignWare APB/16550-compatible UART drivers.
3. Add timeout-bounded transmit and a serial smoke test.
4. Test on physical hardware before changing any board's status to supported.

No Linux kernel code is copied into this directory.