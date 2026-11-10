# Work-session checkpoint — 2026-11-10 afternoon

## Repository and source revision

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- Validated source commit: `cdd0f46b19ac4ca5b04040e3eedb0f264b4d82f5`
- Workflow: [RV64 S-Mode Kernel run #132](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725952037)
- Result: **completed / success** on 2026-11-10.

## Work verified in the live repository

1. `kernel/src/umode_trap.S`
   - Privilege-aware trap vector uses `sscratch` to switch U-origin traps to the supervisor-owned trap stack before allocating the 288-byte frame.
   - S-origin traps restore the original S stack and enter the established S-mode trap vector.
   - Saves the integer register frame and `sepc/sstatus/scause/stval`.

2. `kernel/src/trap.S`
   - The shared restore path sets `sscratch=0` for S-mode returns.
   - For U-mode returns, it restores the supervisor trap-stack top to `sscratch` before `sret`.

3. `kernel/src/trap.rs`
   - Verifies U-origin privilege state.
   - Handles U-mode instruction page fault used by the isolation probe and Rust-native ECALL dispatch.
   - Uses compile-time assertions for the 288-byte frame offsets and 16-byte alignment.

4. `kernel/src/vm.rs` and `kernel/linker.ld`
   - Set up a single Sv39 bootstrap identity window.
   - Expose the probe's code and dedicated stack pages as U-accessible; other pages in the bootstrap mapping remain supervisor-only.
   - This is not a general per-process address-space manager.

5. `kernel/src/runtime.rs`, `kernel/src/scheduler.rs`, and `kernel/src/ipc.rs`
   - Two S-mode tasks plus one U-mode test task.
   - SBI TIME timer preemption and trap-frame context switching.
   - Rust-native syscall IDs: Exit=0, SendMessage=1, ReceiveMessage=2, Yield=4.
   - Fixed-capacity scalar mailboxes; ReceiveMessage is non-blocking and returns `-EAGAIN` when empty.

## Actual build and test evidence

The #132 job completed all steps successfully:

| Stage | Observed result |
|---|---|
| Install Rust RV64 target and QEMU | Success |
| `cargo test --manifest-path kernel/allocator-tests/Cargo.toml` | **21 passed, 0 failed** |
| `bash scripts/build-kernel-rv64-s-mode.sh` | Success |
| RV64 output | Statically linked ELF64 RISC-V, RVC, soft-float ABI; entry `0x80200000` |
| OpenSBI/QEMU boot | Success |
| Artifact upload | Success |

The artifact ZIP was downloaded and inspected. It contains:

- `openxzuff-kernel.elf` — 63,648 bytes.
- `openxzuff-kernel.bin` — 28,688 bytes.
- `qemu-serial.log` — 3,280 bytes.

The captured QEMU serial log (OpenSBI v1.3, `qemu-system-riscv64 -machine virt -m 256M -smp 1`) reports:

- `RV64 S-mode entry: OK`
- `FDT timebase-frequency: parsed; 10 ms quantum ticks=186a0`
- `Sv39 bootstrap page tables: PASS; U-mode mappings isolated`
- `PPA alloc/free smoke test: PASS`
- `Heap alloc/free smoke test: PASS`
- `S-mode trap GPR save/restore + sret: PASS`
- `Cooperative scheduler state test: PASS`
- `RV64 timer preemption + stack context switch: PASS; interrupts=2`
- `RV64 callee-saved register + private stack preservation: PASS`
- `RV64 IPC empty receive -EAGAIN: PASS`
- `RV64 IPC mailbox send/receive round-trip: PASS`
- `U-mode Yield syscall context switch + Sv39 isolation: PASS`
- `U-mode Exit syscall + scheduler task exit: PASS`

The workflow contains an individual `grep -q` assertion for every required runtime marker above, so missing output fails the job.

Artifact: [run #132 build outputs and serial log](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725952037/artifacts/13723271109).

## Regression diagnosis during integration

- Run #129 (commit `d69b5b9...`) and run #130 (commit `da80b3c...`) built the kernel but failed at the QEMU workflow assertion stage.
- Their serial logs showed the allocator, S-mode trap, timer, IPC request/reply, U-mode Yield and Exit markers. The missing assertion was the explicit `RV64 IPC empty receive -EAGAIN: PASS` marker even though the empty-queue path had been observed internally.
- Run #131 added the missing serial assertion marker; it passed.
- Run #132 passed again after the SIE mask was scoped to the RV64 target. Its host-test step emitted no Rust warnings in the captured key log lines.

The failures were real CI failures caused by incomplete test evidence, not evidence that those failed revisions had passed. The corrected workflow now verifies the EAGAIN behavior explicitly.

## Boundaries and remaining engineering work

The verified runtime is a single-hart QEMU bring-up test, not a production scheduler or complete OS.

- Only one U-mode task is exercised, using one static supervisor trap stack. Multiple U-mode tasks need per-task/per-hart supervisor trap stacks.
- A dedicated test must force a timer interrupt while the U-mode task is executing; the current timer marker proves timer preemption/context switching in the mixed runtime but does not isolate U-mode timer preemption.
- Add exhaustive U-mode GPR preservation sentinels and negative trap/syscall tests.
- IPC is scalar-only and non-blocking; no user pointers, copy-in/copy-out, or blocking receive.
- Sv39 is one bootstrap identity window, not per-process address spaces or memory reclamation.
- The active syscall path is Rust-native; no stable Rust/C++ dispatcher bridge is wired into the kernel.
- No physical-board boot, physical timer validation or LAVA result is established by this run.
- No SMP/multi-hart safety, complete driver stack, GUI/compositor integration or production stability claim is made.

## Next implementation sequence

1. Add explicit U-mode timer-preemption and full user-register preservation tests.
2. Introduce per-task/per-hart supervisor trap-stack ownership before adding another U-mode process.
3. Add negative syscall/trap and user-memory range tests.
4. Evolve the bootstrap Sv39 mapping into per-process address spaces with safe allocation/reclamation.
5. Re-run QEMU CI after each change, then arrange physical-board/LAVA validation separately.

