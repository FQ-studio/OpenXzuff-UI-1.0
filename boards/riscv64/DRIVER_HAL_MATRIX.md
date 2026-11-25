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
   threshold/claim windows against the translated MMIO resource. Boot-hart
   S-mode context selection is implemented: CPU interrupt-controller phandles
   resolve through CPU `reg` to architectural hart IDs, cause 9 is selected,
   and OpenSBI-masked M-mode entries retain their original context indexes.
   A bounds-checked volatile register layer now exposes priority, threshold,
   source-enable, claim and complete operations with RISC-V I/O fences. Exact
   head QEMU CI reads and writes back priority 1 and the selected context's
   threshold after Sv39 mapping. Host backing-memory tests cover enable-bit
   read/modify/write and claim/complete API behavior; these host tests are not
   physical PLIC tests. The QEMU boot probe does not enable a source, set
   `sie.SEIE`, claim/complete a hardware interrupt, or prove end-to-end
   external-interrupt delivery. Concurrent enable-bitmap updates still require
   caller-side serialization.
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
The initial VM/HAL mapping layer now maps validated UART and PLIC windows
below 1 GiB into both kernel and probe-user Sv39 roots using identity-mapped
4 KiB leaves with supervisor RW, no U permission and no execute permission.
The mapper verifies every page in both roots and rejects conflicting mappings
or table-pool exhaustion. Base Sv39 has no standard cacheability PTE field;
device classification/order still depends on platform PMA and explicit I/O
fences. The mapper itself does not perform register IO. Separate driver layers
now provide bounds-checked volatile PLIC register access and a byte-wide
QEMU 16550 polling HAL. The PLIC trap dispatcher is installed, but no UART
handler is registered and no end-to-end external interrupt is delivered.
Clock/reset providers and non-QEMU UART initialization remain unresolved.
The metadata scanners require one root node, balanced nesting and a terminal
`FDT_END`; the timebase scanner also rejects duplicate
`/cpus/timebase-frequency` properties.

QEMU `virt` smoke boot discovers the `ns16550a` UART candidate at root
address `0x10000000`, size `0x100`, maps its covering page into both Sv39
roots, and runs the kernel's byte-wide 16550 polling driver in internal
loopback mode. The exact-head QEMU assertion requires a volatile TX-to-RX
round-trip of byte `0x5a`. This proves the QEMU 16550 register path only;
it does not validate the SiFive UART or JH7110/TH1520 UART variants,
configure baud from a board clock, or deliver a UART interrupt. The kernel
HAL exposes bounded polling `write_byte`, non-blocking `read_byte`, and
`flush` operations for the 16550 implementation.
It also discovers the PLIC at root address `0x0c000000`, size `0x600000`,
with `riscv,ndev=95` and two resolved interrupt contexts, then maps all 1536
covering pages into both roots. CI verifies identity translation,
supervisor-only access and NX permissions for each mapped page, PLIC
priority/threshold volatile readback, and the UART loopback probe. It does
not register a UART IRQ handler or prove end-to-end PLIC interrupt delivery.
No physical-board UART/PLIC driver operation or board boot is claimed.


## 2026-11-25 — QEMU 16550 RX interrupt integration

The QEMU `ns16550a` path now connects the parsed UART PLIC source to the
single-hart dispatcher. After both Sv39 roots contain the validated
supervisor-only UART mapping, the kernel registers a handler for the FDT
source ID, sets PLIC priority 1, enables that source in the boot hart's S-mode
context, and enables only the UART receiver-data-available interrupt.

The handler checks the UART IIR cause and reads RBR to acknowledge one received
byte. The QEMU workflow now connects to the serial TCP backend, waits for the
kernel's ready marker, injects byte `0x5a`, and requires the trap path to
dispatch, read the byte and complete the PLIC claim. The success marker is
reported later from task context.

This is an end-to-end QEMU UART RX -> PLIC -> S-mode external trap -> handler
-> claim completion smoke test when the exact-head workflow passes. It is
single-hart and one-byte only. It does not establish FIFO draining under burst
traffic, SMP-safe enable-word locking, other UART IP variants, physical-board
operation or long-duration stability.


Exact-head verification for this QEMU-only UART RX/PLIC path:
- Code head `55bd3784500553a453748b1fb4edd34edf5971f7`, RV64 S-Mode Kernel PR run #45559688659: **SUCCESS**.
- Host tests: **130/130**; RV64 adapter cross-compile and ELF64 kernel build succeeded.
- QEMU injected one serial byte `0x5a` and passed
  `UART PLIC RX IRQ round-trip: PASS; source=a; byte=5a`, followed by
  the existing timer-preemption, IPC, Sv39 and final U-mode exit assertions.
- [QEMU serial log and kernel artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45559688659/artifacts/14996488588).

This does not establish physical-board UART/PLIC operation, FIFO burst robustness,
SMP safety or long-duration stability.
