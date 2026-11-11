# RV64 U-mode trap entry — implementation and validation contract

Status: **QEMU-validated for one U-mode integration task, with 16 task-indexed supervisor trap-stack slots and an explicit U-mode timer-preemption/resume test**. Multiple U-mode tasks, per-hart stacks and SMP remain out of scope.

## Current implementation

- `kernel/src/umode_trap.S` installs the privilege-aware vector.
  - In U-mode, `sscratch` holds the supervisor trap-stack top; in S-mode it is zero.
  - The first `csrrw sp, sscratch, sp` switches U-origin traps to supervisor-owned storage before allocating the 288-byte frame.
  - S-origin traps restore the interrupted S stack and enter `__gravity_s_trap_vector`.
  - U-origin traps save x0–x31 plus `sepc`, `sstatus`, `scause` and `stval`, then call the Rust dispatcher.
- `kernel/src/trap.S` selects the next frame and sets `sscratch` from the selected frame's SPP state before `sret`: zero for S-mode, supervisor-stack top for U-mode.
- `kernel/src/trap.rs` allocates 16 page-aligned supervisor trap-stack slots indexed by scheduler `TaskId`. It validates a U-origin frame against the current task's exact stack top before dereferencing it, verifies the saved user `sp`, checks SPP=0, and handles U-mode instruction page fault and ECALL dispatch. QEMU checks every slot is supervisor-only in the user page table.
- `kernel/src/vm.rs` creates one Sv39 bootstrap identity window. The probe code and its dedicated stack are the only U=1 mappings in the split 4 KiB page tables; kernel pages remain supervisor-only.
- `kernel/src/runtime.rs` runs two S-mode tasks and one U-mode test task. The timer path uses SBI TIME; a U-mode busy-loop is preempted by the timer, its saved frame is resumed, and test-only syscall ID 5 asserts that event. The U-mode test also exercises Yield, Exit and scalar IPC.
- The active syscall implementation is Rust-native. The host-tested C++ dispatcher is not yet linked into this kernel path.

## Current syscall smoke contract

| ID in a7 | Operation | Current behavior |
|---:|---|---|
| 0 | Exit | Records status and removes the U-mode test task from scheduling |
| 1 | SendMessage | Sends one scalar word to a task mailbox |
| 2 | ReceiveMessage | Non-blocking receive; empty queue returns `-EAGAIN` |
| 4 | Yield | Saves the current U frame and rotates to another runnable task |
| 5 | VerifyUserTimerPreemption (test-only) | Returns success only after the scheduler recorded a timer interrupt while task 2 was in U-mode; emits a QEMU PASS marker |

Arguments/results use `a0..a5`. Returning ECALLs advance `sepc` by four bytes. The test does not accept or dereference user-provided pointers.

## Verified evidence

GitHub Actions [RV64 S-Mode Kernel run #145](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971914049), source commit `4eae95c5366314927959f0fe1c60eff8f1f8ad25`, completed successfully on 2026-11-11.

- Host test step: **23 passed, 0 failed**.
- RV64 cross-build step: success for `riscv64imac-unknown-none-elf`.
- OpenSBI v1.3 / QEMU `virt` boot: success.
- Captured serial log contains:
  - `Sv39 bootstrap page tables: PASS; U-mode mappings isolated`
  - `Per-task supervisor trap stacks: PASS; slots=16; supervisor-only`
  - `S-mode trap GPR save/restore + sret: PASS`
  - `U-mode supervisor trap-stack switch + user SP preservation: PASS`
  - `U-mode timer preemption + saved-frame resume: PASS`
  - `RV64 timer preemption + stack context switch: PASS; interrupts=2`
  - `RV64 callee-saved register + private stack preservation: PASS`
  - `RV64 IPC empty receive -EAGAIN: PASS`
  - `RV64 IPC mailbox send/receive round-trip: PASS`
  - `U-mode Yield syscall context switch + Sv39 isolation: PASS`
  - `U-mode Exit syscall + scheduler task exit: PASS`

The workflow requires each marker with an individual `grep -q` assertion. The [run #145 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971914049/artifacts/13833448650) includes `openxzuff-kernel.elf`, `openxzuff-kernel.bin` and `qemu-serial.log`.

## Explicit limits and next tests

This is a bring-up test, not a production process subsystem:

1. Sixteen task-indexed supervisor trap-stack slots are allocated and checked for S-only mappings, but only task 2 currently runs in U-mode. Add a second U-mode task with a distinct user stack and prove both task IDs trap and resume on their own stack slots.
2. Add per-hart trap-stack ownership and SMP-safe scheduler/IPC synchronization before multi-hart execution.
3. Add exhaustive U-mode GPR/CSR sentinels across ECALL, timer preemption, Yield and task exit; current tests do not prove every U-mode register value.
4. Add negative tests for S-mode ECALL routing, unsupported U-mode exceptions, invalid syscall arguments and invalid user addresses.
5. Introduce user-memory copy-in/copy-out with range and permission checks before any pointer-bearing syscall.
6. Replace the bootstrap-only page tables with per-process address spaces and reclamation.
7. Define and test the Rust/C++ ABI bridge only after syscall error, argument, return and task-exit semantics are stable.
8. Repeat the relevant tests on physical boards and LAVA before claiming board support.

Do not use a successful QEMU run as evidence of physical-board boot, SMP safety, or a complete OS.
