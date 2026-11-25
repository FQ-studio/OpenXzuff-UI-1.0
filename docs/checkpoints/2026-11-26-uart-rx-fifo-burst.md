# 2026-11-26 — bounded UART RX FIFO drain and burst verification

## Repository state at start
- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feat/fdt-ranges-root-translation`
- Parent head: `a894b84ec04d3735edcd0309dedd78edd0975331`
- Code checkpoint head: `79c004ff3af532091dac8842b197090b8306b196`
- PR #15 remains open and targets `feature/win32-gaming-gravity-port`.
- `main` is unchanged.

## Starting evidence inspected
- Previous PR head Actions run #45559961100: RV64 job succeeded.
- Its log showed the existing one-byte UART RX -> PLIC -> S-mode trap -> handler -> claim completion assertion.
- Host suite at the previous code head: 130/130.
- OpenXzuff Core run #45559961205 completed successfully; its job log shows OS contracts/Xzuff Go, RV64 NDK, Android ARM64 xgo package, Luau, JavaScript, Lua, OpenXzuff3D and render steps completed.
- VisionFive 2 run #45559961079 completed host/board-contract checks; this is not a physical-board boot.

## Changes in this checkpoint
1. Added a bounded 16550 RX FIFO drain API. It handles receiver-data, character-timeout and line-status causes, captures LSR error flags, and reports whether data remains after the caller's buffer fills.
2. Added a single-producer/single-consumer byte queue for interrupt-to-task handoff. The intended use is one interrupt producer and one task consumer on a single hart; SMP use is explicitly unsupported without external synchronization.
3. Updated the UART IRQ handler to drain up to 64 bytes per interrupt and enqueue them without allocation or console output in interrupt context.
4. Updated task-context verification to validate the ordered byte sequence `5a, 6b, 7c, 8d`, reject queue drops, reject reported line-status errors and wait until the device reports no pending FIFO data.
5. Updated QEMU's TCP serial harness to inject the four-byte burst and require the new exact serial marker plus the existing final U-mode task-exit marker.
6. Added host tests for FIFO ordering, bounded draining, pending-data reporting, line-status errors and queue wrap/full/zero-capacity behavior.
7. Rewrote the driver/HAL matrix section to remove stale statements that the QEMU UART IRQ path was not yet connected.

## Exact-head verification — code head 79c004f
- [RV64 S-Mode Kernel PR run #45720938345](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938345): **SUCCESS**.
  - Host tests: **136 passed, 0 failed**.
  - RV64 adapter cross-compile and kernel ELF build succeeded.
  - OpenSBI/QEMU serial log contains:
    `UART PLIC RX IRQ burst: PASS; source=a; count=4; bytes=5a,6b,7c,8d`.
  - Existing timer preemption/context switch (3 interrupts), IPC mailbox, Sv39, U-mode syscall and final task-exit assertions also passed.
  - [Kernel ELF and QEMU serial artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938345/artifacts/15070889449).
- [RV64 S-Mode Kernel push run #45720931248](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720931248): **SUCCESS**, also 136/136 host tests and the same QEMU four-byte burst marker.
  - [Push artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720931248/artifacts/15072307283).
- [VisionFive 2 PR run #45720938335](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938335): **SUCCESS**, CTest **16/16** and host/board-contract checks. This is not a physical-board boot.
- [VisionFive 2 push run #45720931257](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720931257): **SUCCESS**, CTest **16/16** and host/board-contract checks only.
- [Win32 Gravity Backend Bridge PR run #45720938413](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938413): **SUCCESS**, host bridge and RV64 adapter jobs.
- OpenXzuff UI Core PR run [#45720938336](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938336) and push run [#45720931254](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720931254) were still running at the last inspection, at **Build Luau runtime**. Earlier steps including OS contracts, Xzuff Go, RV64 NDK, Android ARM64 xgo and SDK/NDK packaging had completed. No overall Core PASS is claimed for this head.

The code head was tested before this documentation-only follow-up commit. The follow-up does not change kernel code or workflow logic.

## Limits
- Single-hart QEMU `virt` only.
- Four-byte injected burst, not continuous-throughput or FIFO-overrun stress.
- No SMP-safe PLIC enable-word lock.
- No physical-board UART/PLIC test.
- No SiFive UART or DesignWare APB UART implementation.
- No long-duration stability claim.
