# Work-session checkpoint — 2026-11-10 afternoon

## Repository and source revision

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- Latest validated source commit: `cbc196338e124005ea95958a10c430d3f3ade3a0`
- Workflow: [RV64 S-Mode Kernel run #134](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42768638698)
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
   - Validates the U trap frame pointer is at the expected top-of-supervisor-stack frame location before dereferencing it.
   - Verifies the saved user `sp` equals the probe's initial user stack top.
   - Verifies U-origin privilege state and handles the U-mode instruction page fault and Rust-native ECALL dispatch.
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

Run #134 completed all steps successfully:

| Stage | Observed result |
|---|---|
| Install Rust RV64 target and QEMU | Success |
| `cargo test --manifest-path kernel/allocator-tests/Cargo.toml` | **21 passed, 0 failed** |
| `bash scripts/build-kernel-rv64-s-mode.sh` | Success |
| RV64 output | Statically linked ELF64 RISC-V, RVC, soft-float ABI; entry `0x80200000` |
| OpenSBI/QEMU boot | Success |
| Artifact upload | Success |

The artifact ZIP was downloaded and inspected. It contains:

- `openxzuff-kernel.elf` — 63,936 bytes.
- `openxzuff-kernel.bin` — 28,688 bytes.
- `qemu-serial.log` — 3,347 bytes.

The captured QEMU serial log (OpenSBI v1.3, `qemu-system-riscv64 -machine virt -m 256M -smp 1`) reports:

- `RV64 S-mode entry: OK`
- `FDT timebase-frequency: parsed; 10 ms quantum ticks=186a0`
- `Sv39 bootstrap page tables: PASS; U-mode mappings isolated`
- `PPA alloc/free smoke test: PASS`
- `Heap alloc/free smoke test: PASS`
- `S-mode trap GPR save/restore + sret: PASS`
- `Cooperative scheduler state test: PASS`
- `U-mode supervisor trap-stack switch + user SP preservation: PASS`
- `RV64 timer preemption + stack context switch: PASS; interrupts=2`
- `RV64 callee-saved register + private stack preservation: PASS`
- `RV64 IPC empty receive -EAGAIN: PASS`
- `RV64 IPC mailbox send/receive round-trip: PASS`
- `U-mode Yield syscall context switch + Sv39 isolation: PASS`
- `U-mode Exit syscall + scheduler task exit: PASS`

The workflow contains an individual `grep -q` assertion for every required runtime marker above, so missing output fails the job.

Artifact: [run #134 build outputs and serial log](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42768638698/artifacts/13741116624).

## Regression diagnosis and follow-up

- Runs #129 and #130 built the kernel but failed at the QEMU assertion stage because the explicit `RV64 IPC empty receive -EAGAIN: PASS` serial marker was missing, even though the empty-queue path had been observed internally. Run #131 added the marker; #131 and #132 then passed.
- Run #133 added an explicit runtime check that the U-origin trap frame occupies the supervisor-owned stack and that the saved user `sp` is preserved.
- Run #134 moved this validation ahead of the frame dereference and corrected the diagnostic line ending. Host tests, RV64 build, QEMU boot and the new serial assertion all passed again.

These earlier failures remain real failures for their respective revisions; the corrected revisions were re-tested rather than retroactively relabelled.

## Related OpenXzuff UI Core host baselines from the same morning

These workflows ran earlier on 2026-11-10 against commit `17c54b5a23`; they were not re-run after the later kernel-only trap changes because their path filters do not include `kernel/src/trap.rs`.

- [OpenXzuff UI Runtime run #150](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725681832): CMake host build and root CTest **14/14 passed**; Xzuff Go compiler/.xexe fixture checks; Android NDK 27.2.12479018 ARM64 toolchain build/package; Luau source and .lap package smoke tests; JavaScript tests **2/2**; standard Lua tests **2/2**; OpenXzuff3D tests **1/1** and non-empty rendered scene artifact. The 3D build emitted one non-fatal unused-function warning in `renderer.cpp`.
- [VisionFive 2 Kernel Readiness run #34](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725681937): CMake host build, CTest **14/14 passed**, and VisionFive 2 source/boot-contract checks. This is host/contract validation only, not a physical-board boot or LAVA result.

## Boundaries and remaining engineering work

The verified runtime is a single-hart QEMU bring-up test, not a production scheduler or complete OS.

- Only one U-mode task is exercised, using one static supervisor trap stack. Multiple U-mode tasks need per-task/per-hart supervisor trap stacks.
- A dedicated test must force a timer interrupt while the U-mode task is executing; the current timer marker does not isolate U-mode timer preemption.
- Add exhaustive U-mode GPR/CSR preservation sentinels and negative trap/syscall tests.
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
