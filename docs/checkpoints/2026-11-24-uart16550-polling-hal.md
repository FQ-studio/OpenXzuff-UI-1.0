# 2026-11-24 — QEMU 16550 polling UART HAL

## Repository and branch state

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feat/fdt-ranges-root-translation`
- Latest documentation head at checkpoint creation: `febea4b7d294032a97a68926dcede229efb17e49`
- UART code revision: `180ddc116c197dd35f568f2890eff632f6a34efa`
- PR #15 remains open and targets `feature/win32-gaming-gravity-port`; this work is not merged into `main`.

## Verified baseline before this UART change

The immediately preceding code head `0f92f6e5f9dd9e62ca021ec5e3127ed143501692` completed:
- RV64 host suite: **116/116 passed**.
- RV64 release build: emitted a statically linked ELF64 RISC-V kernel.
- OpenSBI + QEMU smoke: passed all existing serial assertions, including PLIC dispatcher installation, timer preemption, context switching, IPC and Sv39 isolation.
- OpenXzuff UI Core workflow #45293500446: **SUCCESS**; Lua 2/2, JavaScript 2/2, OpenXzuff3D 1/1 and render artifact checks passed; root CTest and the xgo/NDK/Luau build stages completed.
- VisionFive 2 and Win32 bridge workflows completed successfully; VisionFive checks are host/contract checks, not physical board boot.

## Implemented in this checkpoint

- Added `kernel/src/uart16550.rs`, a no_std, byte-wide 16550 driver.
- Added the narrow `UartConsole` contract: bounded `write_byte`, non-blocking `read_byte`, and `flush`.
- Register addresses are validated against the translated `MmioRegion`; reads/writes use volatile byte accesses and RISC-V I/O fences.
- Baud divisor is caller-supplied; the driver does not guess a board clock.
- Added a QEMU-only internal loopback probe. It preserves the existing line-control, divisor, modem-control and interrupt-enable configuration; it refuses to run if DLAB is already set.
- Integrated the probe after the UART page has been mapped into both Sv39 roots. It sends `0x5a` and requires the received byte to match.
- Added host tests for the full eight-register window, byte offset bounds, zero divisor rejection and safe rejection while DLAB is active.
- Added a mandatory QEMU serial assertion:
  `UART 16550 loopback: PASS; tx=5a; rx=5a`.
- Updated the driver/HAL matrix and serial driver README to distinguish the QEMU kernel implementation from the existing C++ host-side UART classes.

## Current-head verification

The new RV64 and related PR workflows have been created but were still **queued** at checkpoint time:
- RV64 S-Mode Kernel PR run #45340678146.
- VisionFive 2 Kernel Readiness PR run #45340678165.
- Win32 Gravity Backend Bridge PR run #45340678150.
- OpenXzuff UI Runtime/Core PR run #45340678313.

No host test count, RV64 build, QEMU UART loopback result, or Core result is claimed for the new UART code until these exact-head workflows finish. Earlier successful runs are baseline evidence only.

## Boundaries

- The new driver is currently enabled only for the QEMU `ns16550a` compatible match.
- The SiFive UART and JH7110/TH1520 DesignWare variants are not covered by this implementation.
- FDT clock-frequency, reg-shift, reg-io-width, clock and reset handling are not yet connected to this driver.
- The UART is not yet registered as a PLIC interrupt source; no UART IRQ handler, QEMU input injection or end-to-end external interrupt round-trip is claimed.
- No physical-board UART operation or board boot is established.

## Next actions

1. Inspect exact-head host tests, RV64 ELF build and QEMU UART loopback assertion; repair any failure before proceeding.
2. Keep the QEMU 16550 path isolated from unsupported UART IPs.
3. Parse the UART interrupt-parent/specifier from FDT and add a QEMU external-interrupt round-trip only after the polling driver is verified.
4. Then extend the HAL to SiFive and DesignWare UARTs with their clock/reset and register-width/shift contracts.
