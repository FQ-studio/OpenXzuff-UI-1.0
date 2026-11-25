# 2026-11-25 — UART RX through PLIC external interrupt

## Branch / PR
- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feat/fdt-ranges-root-translation`
- PR: #15, still open; base `feature/win32-gaming-gravity-port`
- Checkpoint code head: `02d8b96586f238dbf7dacf3da4e191a36fde0c2f`

## Starting checkpoint verified
The previous UART routing metadata head had exact-head evidence:
- RV64 PR run #45514095179: SUCCESS, 123/123 host tests, RV64 adapter cross-compile, ELF64 RISC-V kernel build and OpenSBI/QEMU smoke.
- QEMU serial included UART PLIC route `controller=3; source=a`, 16550 internal loopback, timer preemption/context switching, IPC and Sv39 checks.
- VisionFive 2 run #45514095021: SUCCESS for host/board-contract checks only, not physical board boot.
- Win32 bridge run #45514094934: SUCCESS.

## New implementation
1. Added 16550 RX interrupt enable and IIR-based RX service to `kernel/src/uart16550.rs`.
2. Connected the FDT-resolved UART source to the selected boot-hart S-mode PLIC context:
   - PLIC priority 1 and source enable through `irq::register_handler`.
   - UART receiver-data-available interrupt enable after MMIO mapping.
   - The handler reads IIR and RBR; the PLIC dispatcher completes the claimed source.
3. Added atomic result reporting from task context, avoiding console output inside the interrupt handler.
4. Extended the RV64 QEMU workflow to use a TCP serial backend, wait for the ready marker, inject byte `0x5a`, and require both the interrupt result and final U-mode task-exit marker.
5. Added host integration coverage for ProcessTable, TaskManager, per-process handles, RAM VFS and IPC mailbox FIFO/capacity, plus UART RX register behavior.

## Exact-head test evidence before workflow harness correction
For head `ff488c93dd70f71538917fd4b134ed5364c99243`, RV64 run #45559210690:
- Host allocator/integration suite: **130/130 passed**.
- RV64 ecall adapter cross-compile: success.
- Kernel ELF: built as statically linked ELF64 RISC-V.
- QEMU serial explicitly printed:
  `UART PLIC RX IRQ ready; source=a`
  `UART PLIC RX IRQ round-trip: PASS; source=a; byte=5a`

The overall RV64 job nevertheless **FAILED**. Diagnosis from its log: the Python QEMU harness terminated the emulator as soon as the new UART IRQ marker appeared, before the existing timer-preemption and U-mode exit assertions had time to print. This was a test orchestration early-stop, not a reported UART RX/PLIC handler failure.

## Correction pushed
Commit `02d8b96586f238dbf7dacf3da4e191a36fde0c2f` changes the QEMU harness to wait for both:
- `UART PLIC RX IRQ round-trip: PASS; source=a; byte=5a`
- `U-mode Exit syscall + scheduler task exit: PASS`

The rerun for this corrected head is not yet confirmed at checkpoint creation. Do not claim the full QEMU workflow PASS until that exact-head run succeeds.

## Other current-head workflow observations
For the earlier head `ff488c9`:
- Win32 Gravity bridge #45559210727: both RV64 adapter and host bridge jobs succeeded.
- VisionFive 2 readiness #45559210744: host/board-contract job succeeded; this is not a physical board boot.
- OpenXzuff UI Core #45559210689: OS contract/Xzuff Go/.xexe/Gravity RV64 NDK steps succeeded; Android SDK/NDK installation was still in progress. Full Core result not yet established.

## Boundaries
- The QEMU interrupt path is one byte, one hart and the QEMU 16550 source only.
- FIFO burst draining, UART timeout handling, other UART IPs, SMP locking, physical-board interrupt delivery and long-duration stability remain unverified.
- Existing dynamic process metadata, handles, RAM VFS and IPC structures are now host-integration-tested, but this does not make them a complete process loader, independent per-process Sv39 VM, blocking IPC or persistent storage system.
- PR #15 is not merged into `main`.

## Next
1. Verify exact-head Actions for `02d8b96`; inspect the QEMU serial log and require both the UART IRQ marker and final runtime marker.
2. If green, preserve the new host test coverage and proceed to UART FIFO draining/line-status handling, then a non-QEMU board UART contract.
3. Continue independent per-process page-table roots and process-owned resources; do not claim the current shared kernel/probe-user Sv39 roots are a general process VM.


## Exact-head verification completed after correction

Code head: `55bd3784500553a453748b1fb4edd34edf5971f7`.

- RV64 S-Mode Kernel PR run #45559688659: **SUCCESS**.
  - Host suite: **130/130 passed**.
  - RV64 ecall adapter cross-compile: success.
  - Kernel: statically linked ELF64 RISC-V produced.
  - OpenSBI/QEMU smoke: success, including all required serial assertions.
  - New end-to-end marker: `UART PLIC RX IRQ round-trip: PASS; source=a; byte=5a`.
  - Existing PLIC priority/threshold readback, timer preemption/context switch, IPC, Sv39, U-mode syscall and final task-exit assertions also passed.
  - [Kernel/QEMU artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45559688659/artifacts/14996488588).

The two preceding QEMU workflow failures were diagnosed and corrected:
1. Run #45559210690 stopped QEMU at the first UART IRQ marker, before the rest of the existing runtime smoke assertions. The harness was changed to wait for the final U-mode task-exit marker as well.
2. Run #45559392042 then showed the UART interrupt and final runtime markers, but failed the old `PLIC MMIO access` assertion. Inspection found the earlier main.rs block replacement had inadvertently removed the priority/threshold readback smoke. Commit `55bd378` restored that block before dispatcher installation. The next exact-head run passed.

Other workflows on code head `55bd378`:
- Win32 Gravity Backend Bridge #45559688598: **SUCCESS**, host mock and RV64 adapter jobs.
- VisionFive 2 Kernel Readiness #45559688607: **SUCCESS**, host/board-contract checks only; not a physical board boot.
- OpenXzuff UI Runtime/Core #45559689062: still **IN PROGRESS** at the latest check. OS contract/Xzuff Go/Coder Checker, `.xexe`, Gravity RV64 NDK, Android ARM64 xgo cross-compile/package and SDK/NDK package steps have completed; Luau runtime build is still running. No overall Core PASS is claimed.
