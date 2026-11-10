# RV64 U-mode trap entry — implementation and validation contract

Status: **implemented and QEMU-validated for the single U-mode integration test task**. General multi-process and SMP support remain out of scope.

## Current implementation

- `kernel/src/umode_trap.S` installs the privilege-aware vector.
  - In U-mode, `sscratch` holds the supervisor trap-stack top; in S-mode it is zero.
  - The first `csrrw sp, sscratch, sp` switches U-origin traps to supervisor-owned storage before allocating the 288-byte frame.
  - S-origin traps restore the interrupted S stack and tail into `__gravity_s_trap_vector`.
  - U-origin traps save x0–x31 plus `sepc`, `sstatus`, `scause` and `stval`, then call the Rust dispatcher.
- `kernel/src/trap.S` selects the next frame and sets `sscratch` from the selected frame's SPP state before `sret`: zero for S-mode, supervisor-stack top for U-mode.
- `kernel/src/trap.rs` checks that user-origin traps have SPP=0 and handles instruction page fault, U-mode ECALL and unexpected U-mode exceptions.
- `kernel/src/vm.rs` creates one Sv39 bootstrap identity window. The probe code and its dedicated stack are the only U=1 mappings in the split 4 KiB page tables; kernel pages remain supervisor-only.
- `kernel/src/runtime.rs` runs two S-mode tasks and one U-mode test task. The timer path uses SBI TIME; the U-mode test exercises Yield, Exit and scalar IPC.
- The active syscall implementation is Rust-native. The host-tested C++ dispatcher is not yet linked into this kernel path.

## Current syscall smoke contract

| ID in a7 | Operation | Current behavior |
|---:|---|---|
| 0 | Exit | Records status and removes the U-mode test task from scheduling |
| 1 | SendMessage | Sends one scalar word to a task mailbox |
| 2 | ReceiveMessage | Non-blocking receive; empty queue returns `-EAGAIN` |
| 4 | Yield | Saves the current U frame and rotates to another runnable task |

Arguments/results use `a0..a5`. Returning ECALLs advance `sepc` by four bytes. The test does not accept or dereference user-provided pointers.

## Verified evidence

GitHub Actions [RV64 S-Mode Kernel run #132](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725952037), source commit `cdd0f46b19ac4ca5b04040e3eedb0f264b4d82f5`, completed successfully on 2026-11-10.

- Host test step: **21 passed, 0 failed**.
- RV64 cross-build step: success for `riscv64imac-unknown-none-elf`.
- OpenSBI v1.3 / QEMU `virt` boot: success.
- Captured serial log contains:
  - `Sv39 bootstrap page tables: PASS; U-mode mappings isolated`
  - `S-mode trap GPR save/restore + sret: PASS`
  - `RV64 timer preemption + stack context switch: PASS; interrupts=2`
  - `RV64 callee-saved register + private stack preservation: PASS`
  - `RV64 IPC empty receive -EAGAIN: PASS`
  - `RV64 IPC mailbox send/receive round-trip: PASS`
  - `U-mode Yield syscall context switch + Sv39 isolation: PASS`
  - `U-mode Exit syscall + scheduler task exit: PASS`

The workflow requires these markers with individual `grep -q` assertions. The uploaded artifact includes `openxzuff-kernel.elf`, `openxzuff-kernel.bin` and `qemu-serial.log`.

## Explicit limits and next tests

This is a bring-up test, not a production process subsystem:

1. The current implementation has one U-mode task and one static supervisor trap stack. Allocate a distinct supervisor trap stack per U-mode task (and per hart) before supporting concurrent or multiple U-mode processes.
2. Add an explicit test proving a timer interrupt can preempt a U-mode task and later resume that exact U frame.
3. Add exhaustive U-mode register-preservation sentinels, including all caller/callee-saved integer registers and stack alignment across ECALL, Yield and timer paths.
4. Add negative tests for S-mode ECALL routing, unsupported U-mode exceptions and invalid syscall arguments.
5. Introduce user-memory copy-in/copy-out with range and permission checks before any pointer-bearing syscall.
6. Replace the bootstrap-only page tables with per-process address spaces and reclamation.
7. Define and test the Rust/C++ ABI bridge only after syscall error, argument, return and task-exit semantics are stable.
8. Repeat the relevant tests on physical boards and LAVA before claiming board support.

Do not use a successful QEMU run as evidence of physical-board boot, SMP safety, or a complete OS.
