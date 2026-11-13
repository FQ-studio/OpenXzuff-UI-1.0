# RV64 Driver and HAL target matrix

Gravity Kernel is RV64-only. Device discovery uses the firmware-provided DTB/FDT.
MMIO addresses in board notes are candidates only; the active DTB is authoritative
for `compatible`, `status`, `reg`, bus `ranges`, clocks, resets and interrupts.

## Board targets

| Priority | Board / SoC | UART compatible match | Interrupt-controller compatible match |
|---|---|---|---|
| 1 | SiFive HiFive Unmatched / FU740-C000 | `sifive,fu740-c000-uart`, fallback `sifive,uart0` | `sifive,plic-1.0.0` |
| 2 | StarFive VisionFive 2 / JH7110 | `starfive,jh7110-uart`, fallback `snps,dw-apb-uart` | `starfive,jh7110-plic`, fallback `sifive,plic-1.0.0` |
| 3 | Sipeed Lichee Pi 4A / TH1520 | `snps,dw-apb-uart` | `riscv,plic0` |

These are driver-match candidates from published board DTS/bindings. The
firmware-supplied board DTB still determines which node is enabled and which
UART is the active console. QEMU `virt` remains the CI target and does not
prove physical-board driver operation.

## Driver implementation order

1. FDT node discovery: exact compatible-list matching, `status`, inherited
   bus cell widths, `reg` translation through `ranges`, and interrupt-parent
   resolution.
2. UART console: SiFive UART IP and DesignWare APB UART implementations behind
   one narrow polling-console interface. JH7110 requires its clock/reset and
   register-width/register-shift properties to be honoured.
3. Timer: SBI TIME first; direct ACLINT only when FDT and privilege contract allow it.
4. Interrupt controller: PLIC first for these three initial board families;
   parse each DTB's compatible, context/interrupt wiring and source count.
5. Storage/network/display only after concrete compatible, clocks, resets, DMA
   and interrupt dependencies are parsed.

## HAL contract

The HAL exposes validated capabilities, not raw board addresses:

- `UartConsole`: initialize, write byte, read byte, flush.
- `Timer`: frequency, set deadline, acknowledge/dispatch.
- `InterruptController`: register source, enable, claim, complete.
- `MmioRegion`: validated mapped range and explicit access width.
- `BlockDevice`, `NetworkDevice`, `DisplayDevice`: only after concrete
  drivers and lifecycle/error contracts exist.

Drivers receive validated resources from the FDT/platform layer. They must not
parse arbitrary DTB offsets themselves or assume another board's MMIO address.

## Validation boundary

The kernel parses FDT RAM ranges and timebase-frequency. `has_compatible`
matches exact NUL-delimited compatible entries, but does not yet return the
matching node path or resolve bus `ranges`, `status`, clocks, resets, MMIO
translations or interrupt routing. No physical UART driver or board boot is
claimed by this matrix.
