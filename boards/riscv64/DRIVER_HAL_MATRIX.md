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

1. **FDT discovery and resource validation.** Exact compatible-list matching,
   inherited `status`, bus cell widths, `reg` translation through ancestor
   `ranges`, interrupt-parent resolution, and checked MMIO windows are present.
2. **UART.** The QEMU `ns16550a` driver supports byte-wide volatile register
   access, polling console operations, RX interrupt identification and bounded
   FIFO draining. SiFive UART and DesignWare APB UART variants, board-clock
   baud calculation, and JH7110 clock/reset/register-shift handling remain
   separate drivers and are not claimed implemented.
3. **Timer.** SBI TIME is used for the current QEMU S-mode timer path. Direct
   ACLINT support remains conditional on the active FDT and privilege contract.
4. **PLIC.** FDT discovery, boot-hart S-mode context selection, translated
   MMIO mapping, bounds-checked volatile register access, source enable,
   priority, claim/complete and the S-mode external-trap dispatcher are
   implemented. The dispatcher and handler table are single-hart; SMP requires
   IRQ-safe serialization for registration and enable-word read/modify/write.

## HAL contract

The HAL exposes validated capabilities, not raw board addresses:

- `UartConsole`: bounded polling write/read/flush for the QEMU 16550 path.
- `Timer`: frequency and SBI TIME deadline programming.
- `InterruptController`: register source, enable, claim, complete and dispatch.
- `MmioRegion`: validated translated range and explicit register access width.
- `BlockDevice`, `NetworkDevice`, `DisplayDevice`: not implemented as
  hardware-backed drivers yet.

Drivers receive validated resources from the FDT/platform layer. They must not
parse arbitrary DTB offsets themselves or assume another board's MMIO address.

## Validation boundary

The FDT parser rejects unavailable nodes, malformed cell geometry, missing
intermediate `ranges`, ambiguous translation windows, address-width overflow,
out-of-window resources, invalid interrupt-parent phandles and malformed
interrupt specifiers. The boot-hart S-mode context is selected by resolving
CPU interrupt-controller phandles to CPU `reg` hart IDs and cause 9, while
preserving context indexes when OpenSBI masks an M-mode entry.

The MMIO descriptor requires a known non-zero resource size, checked exclusive
end, caller-specified minimum extent and power-of-two base alignment. Register
access validates natural alignment and full access bounds. The initial Sv39
device mapper installs identity-mapped 4 KiB leaves into both kernel and probe
user roots with supervisor RW, U=0 and X=0. Base Sv39 has no standard
cacheability PTE field; device classification depends on platform PMA and
explicit I/O fences.

## QEMU evidence and limitations

On QEMU `virt`, OpenSBI transfers to the kernel in S-mode on hart 0. The active
FDT reports the `ns16550a` UART at root address `0x10000000`, size `0x100`,
and the PLIC at `0x0c000000`, size `0x600000`, with 95 sources and two
interrupt contexts. The kernel selects context 1 for S-mode external interrupt
cause 9, maps the UART and PLIC windows into both bootstrap roots, and performs
volatile PLIC priority/threshold readback.

The UART source is resolved from FDT (source 10 on this QEMU FDT), registered
with the single-hart PLIC dispatcher, and enabled only after the handler is
installed. The UART RX handler services receiver-data, character-timeout and
line-status causes, drains a caller-bounded batch from the 16550 FIFO, and
publishes bytes to a single-producer/single-consumer queue for task-context
consumption. It does not print or allocate inside the interrupt handler.

The previous exact-head QEMU test verified one injected byte end-to-end:
`55bd3784500553a453748b1fb4edd34edf5971f7`, run
[#45559688659](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45559688659),
host tests 130/130 and marker
`UART PLIC RX IRQ round-trip: PASS; source=a; byte=5a`.

The bounded four-byte RX burst is verified on code head
`79c004ff3af532091dac8842b197090b8306b196` by [RV64 PR run
#45720938345](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938345):
host tests **136/136**, RV64 adapter cross-compile, ELF64 RISC-V kernel build,
and OpenSBI/QEMU serial marker
`UART PLIC RX IRQ burst: PASS; source=a; count=4; bytes=5a,6b,7c,8d`.
The kernel ELF and serial log are in
[artifact 15070889449](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938345/artifacts/15070889449).
This proves one injected four-byte burst, not sustained-throughput or
continuous-overrun behavior.

These tests remain single-hart QEMU smoke tests. They do not establish
sustained UART throughput, FIFO overflow immunity under continuous traffic,
SMP safety, SiFive/JH7110/TH1520 UART operation, physical-board interrupt
delivery, long-duration stability or a production-ready HAL.
