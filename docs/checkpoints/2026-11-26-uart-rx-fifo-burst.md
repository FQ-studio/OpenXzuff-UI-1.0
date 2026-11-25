# 2026-11-26 — bounded UART RX FIFO drain and burst verification

## Repository state at start
- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feat/fdt-ranges-root-translation`
- Parent head: `a894b84ec04d3735edcd0309dedd78edd0975331`
- PR #15 remains open and targets `feature/win32-gaming-gravity-port`.
- `main` is unchanged.

## Starting evidence inspected
- Current PR head Actions run #45559961100: RV64 job succeeded.
- The job log showed the existing one-byte UART RX -> PLIC -> S-mode trap -> handler -> claim completion assertion.
- Host suite at the previous code head: 130/130.
- OpenXzuff Core run #45559961205 completed successfully; the detailed job shows OS contracts/Xzuff Go, RV64 NDK, Android ARM64 xgo package, Luau, JavaScript, Lua, OpenXzuff3D and render steps all completed.
- VisionFive 2 run #45559961079 completed host/board-contract checks; this is not a physical-board boot.

## Changes in this checkpoint
1. Added a bounded 16550 RX FIFO drain API. It handles receiver-data, character-timeout and line-status causes, captures LSR error flags, and reports whether data remains after the caller's buffer fills.
2. Added a single-producer/single-consumer byte queue for interrupt-to-task handoff. The intended use is one interrupt producer and one task consumer on a single hart; SMP use is explicitly unsupported without external synchronization.
3. Updated the UART IRQ handler to drain up to 64 bytes per interrupt and enqueue them without allocation or console output in interrupt context.
4. Updated task-context verification to validate the ordered byte sequence `5a, 6b, 7c, 8d`, reject queue drops, reject reported line-status errors and wait until the device reports no pending FIFO data.
5. Updated QEMU's TCP serial harness to inject the four-byte burst and require the new exact serial marker plus the existing final U-mode task-exit marker.
6. Added host tests for FIFO ordering, bounded draining, pending-data reporting, line-status errors and queue wrap/full/zero-capacity behavior.
7. Rewrote the driver/HAL matrix section to remove stale statements that the QEMU UART IRQ path was not yet connected.

## Exact-head verification
Pending at commit creation. The required checks are:
- Rust host suite including the new queue and UART FIFO tests.
- RV64 adapter cross-compile and kernel ELF build.
- OpenSBI/QEMU four-byte UART RX interrupt burst.
- Existing scheduler/context-switch, timer-preemption, IPC, Sv39 and U-mode exit assertions.
- VisionFire 2 host/contract and OpenXzuff Core workflows where triggered.

Do not claim the burst test passes until the exact commit's Actions and serial log prove it.

## Limits
- Single-hart QEMU `virt` only.
- Four-byte injected burst, not continuous-throughput or FIFO-overrun stress.
- No SMP-safe PLIC enable-word lock.
- No physical-board UART/PLIC test.
- No SiFive UART or DesignWare APB UART implementation.
- No long-duration stability claim.
