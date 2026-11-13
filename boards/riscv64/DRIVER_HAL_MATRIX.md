# RV64 Driver and HAL target matrix

Gravity Kernel is RV64-only. Device discovery is driven by the firmware-provided
DTB/FDT. Addresses in board notes are candidates only; they never replace
parsing the active DTB's `compatible`, `status`, `reg`, bus `ranges`,
`interrupt-parent` and interrupt specifier.

## Board targets

| Priority | Board / SoC | UART driver match | Interrupt discovery |
|---|---|---|---|
| 1 | SiFive HiFive Unmatched / FU740-C000 | `sifive,uart0` | Discover PLIC and hart/local timer topology from FDT |
| 2 | StarFive VisionFive 2 / JH7110 | `ns16550a` (NS16550-compatible) | Discover PLIC or APLIC/IMSIC nodes and routing from FDT |
| 3 | Sipeed Lichee Pi 4A / TH1520 | `snps,dw-apb-uart` | Discover TH1520 interrupt and timer topology from FDT |

QEMU `virt` remains the CI target; its UART and interrupt controller do not
prove that a physical-board driver works.

## Driver implementation order

1. FDT node discovery: exact compatible-list matching, status, inherited bus
   cell widths, `reg` translation through `ranges`, and interrupt-parent resolution.
2. UART console: separate SiFive, NS16550 and DesignWare APB implementations
   behind one narrow polling-console interface.
3. Timer: SBI TIME first; direct ACLINT only when FDT and privilege contract allow it.
4. Interrupt controller: PLIC for initial FU740/JH7110 paths where their DTBs
   describe it; APLIC/IMSIC only when a target DTB requires it.
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

The kernel currently parses FDT RAM ranges and timebase-frequency. The new
`has_compatible` primitive matches exact NUL-delimited compatible entries.
It does not yet resolve node paths, bus `ranges`, `status`, clocks, resets,
MMIO translations or interrupt routing. No physical UART driver or board boot
is claimed by this matrix.
