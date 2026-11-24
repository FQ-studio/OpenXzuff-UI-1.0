# RV64 Serial Drivers

This directory contains native early-console driver code. These are not Linux
drivers and must not depend on Linux APIs.

## Board-to-UART match table

| Priority | Board / SoC | Compatible candidates | Driver class | Current implementation boundary |
|---|---|---|---|---|
| 1 | SiFive HiFive Unmatched / FU740-C000 | `sifive,fu740-c000-uart`, fallback `sifive,uart0` | `SiFiveUart` | Bounded TX polling only |
| 2 | StarFive VisionFive 2 / JH7110 | `starfive,jh7110-uart`, fallback `snps,dw-apb-uart` | `Ns16550Uart` | 32-bit register access and bounded TX polling only |
| 3 | Sipeed Lichee Pi 4A / TH1520 | `snps,dw-apb-uart` | `Ns16550Uart` | 32-bit register access and bounded TX polling only |

The compatible strings are match candidates, not proof that a device is enabled
or is the active console. The firmware-provided FDT remains authoritative for
`stdout-path`, `status`, `reg`, `reg-shift`, `reg-io-width`, clocks, resets
and interrupt routing. MMIO values in board notes are candidates only.

## Code and build status

- `uart.cpp` is compiled into the host-side `xzuff_os_base` CMake target.
- Both concrete classes currently implement transmit-byte polling only.
- Poll loops have a fixed iteration bound; this is not a calibrated time limit.
- `Ns16550Uart` uses 32-bit MMIO accesses and a register shift supplied by its
  caller. It does not parse the FDT or configure JH7110/TH1520 clocks and resets.
- The caller must validate and map the MMIO range as device memory before
  constructing a driver. No physical UART bring-up or board boot is claimed.

## Required integration work

1. Extend FDT discovery to return the matching node and validated
   `compatible`, `status`, `reg`, `reg-shift`, `reg-io-width`,
   `clock-frequency`, `stdout-path` and interrupt-parent data.
2. Resolve parent-bus `#address-cells`, `#size-cells` and `ranges` before
   creating an MMIO capability.
3. Add board-specific clock/reset setup where required, then initialize the
   selected UART through a HAL-owned device-memory mapping.
4. Add receive, flush and time-bounded timeout behavior, plus host tests using
   an injectable MMIO/register interface.
5. Validate serial output on physical hardware before changing any board's
   status to supported.

The driver-match and HAL priorities are recorded in
`boards/riscv64/DRIVER_HAL_MATRIX.md`. No Linux kernel code is copied into
this directory.

## Gravity Kernel QEMU 16550 implementation

`kernel/src/uart16550.rs` now provides a separate no_std, byte-wide 16550
polling HAL for the QEMU `ns16550a` device. It validates all eight register
offsets against the translated `MmioRegion`, uses volatile 8-bit accesses
with RISC-V I/O fences, and provides bounded TX polling, non-blocking RX,
and flush. A boot smoke probe switches the UART into internal loopback and
requires TX byte `0x5a` to be received unchanged before returning to normal
transmit mode. Host tests cover window bounds and invalid divisor handling.

This kernel implementation is intentionally limited to QEMU `ns16550a`.
It does not replace the existing C++ host-side `uart.cpp`, and does not yet
claim SiFive/JH7110/TH1520 support, FDT clock/reset setup, or UART IRQ delivery.
