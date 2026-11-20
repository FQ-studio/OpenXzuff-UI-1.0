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
4. Interrupt controller: PLIC first for these three initial board families.
   The current parser recognizes the QEMU/SiFive compatible strings, validates
   inherited node availability, reads `riscv,ndev`, resolves every
   `interrupts-extended` phandle through an available interrupt-controller
   node with a valid `#interrupt-cells`, and counts the resolved contexts.
   The PLIC geometry descriptor checks priority, pending, enable and per-context
   threshold/claim windows against the translated MMIO resource. Hart-specific
   S-mode context selection and actual interrupt delivery remain next.
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

The kernel parses FDT RAM ranges and timebase-frequency. Device discovery
checks exact compatible-list membership and inherited availability status.
`find_compatible_node_translated` returns the first selected node's raw
parent-bus `reg` tuple and its root-address translation through every ancestor
bus `ranges` property. Missing intermediate `ranges`, ambiguous windows,
malformed tuples, address-width overflow and out-of-window resources fail
closed. A missing `reg` remains explicit; zero size-cells yields an address
with unknown size.

The new kernel MMIO descriptor accepts only a translated resource with a known,
non-zero size, checked exclusive end, caller-specified minimum extent and
power-of-two base alignment. Register-address validation accepts explicit
8/16/32/64-bit widths and checks natural alignment plus full access bounds.
These descriptors return physical addresses only: they do not map memory,
set Sv39 device attributes, perform volatile IO, or establish that a platform
resource is safe to claim. Those operations remain responsibilities of the
future VM/platform HAL layer. PLIC parsing validates the phandle references and
specifier lengths in `interrupts-extended`, but does not select the current
hart's supervisor-external context or connect claim/complete to trap dispatch.
Clock/reset providers and UART register initialization remain unresolved.
The metadata scanners require one root node, balanced nesting and a terminal
`FDT_END`; the timebase scanner also rejects duplicate
`/cpus/timebase-frequency` properties.

QEMU `virt` smoke boot discovers the `ns16550a` UART candidate at root
address `0x10000000`, size `0x100`. CI validates that the first 32-bit
register lies within the translated window, but deliberately does not map the
window or initialize/access the UART. The same FDT smoke path discovers the
PLIC at root address `0x0c000000`, size `0x600000`, with `riscv,ndev=95`
and two resolved interrupt contexts; CI validates register-window geometry
only, not interrupt delivery. No physical UART/PLIC driver or board boot is
claimed by this matrix.
