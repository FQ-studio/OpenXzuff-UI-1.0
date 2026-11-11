# Work-session checkpoint — 2026-11-11 afternoon

## Repository and revision

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- Current main head: `4cb674aedd98571a3b9c5e9a4d5bc04162eba6e8`.
- Latest validated kernel source commit: `4cb674aedd98571a3b9c5e9a4d5bc04162eba6e8`.
- Core renderer cleanup commit: `3e8f1ed6c8b3725085391a0e39f4e9cbb4e1250a`.

## Changes made

1. `graphics3d/src/renderer.cpp`: removed the unused internal `clamp01` helper. It had no call sites and generated a non-fatal unused-function warning.
2. `kernel/src/trap.rs`: replaced the single static supervisor trap stack with 16 page-aligned slots indexed by scheduler `TaskId`. Initial U-mode trap-frame construction selects the task's slot; trap validation obtains the current task ID and checks the frame belongs to that slot before dereferencing it.
3. `kernel/src/runtime.rs`: passes the U-mode task ID when creating its initial frame and exposes the current task ID for trap ownership validation.
4. `kernel/src/main.rs`: checks stack-slot alignment/contiguity and verifies all 16 stack pages remain supervisor-only in the user page table.
5. `kernel/src/trap.rs` and `kernel/src/umode_trap.S`: add a negative U-mode syscall probe. Unknown syscall ID 99 must return `-ENOSYS` (-38), and the runtime emits a dedicated marker.
6. The unknown-syscall marker now follows a two-stage check: ID 99 returns `-ENOSYS`; the assembly probe compares `a0` with `-38`; only then does test-only ID 6 ask the handler to emit PASS. This prevents a false PASS marker before the return value is verified.
7. `.github/workflows/rv64-s-mode-kernel.yml`: requires the per-task stack and unknown-syscall serial markers.
7. CI path filters: Core runtime and VisionFive 2 readiness workflows now include lowercase `kernel/**`; the Core workflow includes it for push and pull requests, and VisionFive 2 readiness now includes a pull-request trigger.

## RV64 build and QEMU evidence

[RV64 S-Mode Kernel run #148](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42973125672) completed **successfully** on 2026-11-11 for the latest kernel source commit `4cb674aedd98571a3b9c5e9a4d5bc04162eba6e8`.

- Host Rust tests: **23 passed, 0 failed**.
- RV64 cross-build: `riscv64imac-unknown-none-elf` release build succeeded; output is a statically linked ELF64 RISC-V executable with entry point `0x80200000`.
- OpenSBI v1.3 + QEMU `virt`, 256 MiB, one hart: boot and all required serial assertions succeeded.
- Relevant serial evidence:
  - `Sv39 bootstrap page tables: PASS; U-mode mappings isolated`
  - `Sv39 kernel/user address spaces: PASS; distinct roots and U-only probe mappings`
  - `Per-task supervisor trap stacks: PASS; slots=16; supervisor-only`
  - `U-mode supervisor trap-stack switch + user SP preservation: PASS`
  - `RV64 timer preemption + stack context switch: PASS; interrupts=3`
  - `U-mode timer preemption + saved-frame resume: PASS`
  - `U-mode unknown syscall -ENOSYS: PASS` (printed only after the U-mode probe verified the `-38` return value)
  - `RV64 callee-saved register + private stack preservation: PASS`
  - `RV64 IPC empty receive -EAGAIN: PASS`
  - `RV64 IPC mailbox send/receive round-trip: PASS`
  - `U-mode Yield syscall context switch + Sv39 isolation: PASS`
  - `U-mode Exit syscall + scheduler task exit: PASS`

The workflow has an individual `grep -q` assertion for the new markers. [Run #148 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42973125672/artifacts/13833867960) contains the ELF, raw binary and QEMU serial log.

## OpenXzuff UI Core host evidence

[OpenXzuff UI Runtime, OS Contracts and 3D run #153](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971393352) completed **successfully** for the renderer cleanup commit.

- Root CTest: **14/14 passed**.
- Xzuff Go compiler and `.xexe` fixture validation: passed.
- Xzuff Go / xgo Android ARM64 cross-compile and packaging with NDK 27.2.12479018: passed.
- Luau source and `.lap` package smoke tests: passed.
- JavaScript host tests: **2/2**; standard Lua host tests: **2/2**; OpenXzuff3D tests: **1/1**; rendered-scene artifact verification: passed.
- The unused renderer helper warning is gone. Two non-fatal unused-parameter warnings remain in vendored Luau `Ast.h`; GitHub Actions also reports Node.js 20 action deprecation notices.
- [Run #153 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971393352/artifacts/13833493328).

Core run [#157](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42972631482) completed successfully after the Core workflow path filter was expanded to include `kernel/**` on push and pull requests. It passed root CTest **14/14**, Xzuff Go `.xexe` fixture validation, Android ARM64 toolchain cross-build/package, Luau source/package smoke tests, JavaScript **2/2**, Lua **2/2**, OpenXzuff3D **1/1** and rendered-scene verification. Its artifact is [here](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42972631482/artifacts/13834056868). A further Core run [#158](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42973125749) is validating the latest ENOSYS assertion change; it was still in progress at checkpoint time. Runs #154–#156 were intermediate duplicate runs and are not used as the latest Core baseline.

## VisionFive 2 host contract evidence

[VisionFive 2 Kernel Readiness run #41](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42973125726) passed CMake build, strict-warning compilation, CTest **14/14**, and board boot-contract source checks on the latest validated kernel source commit. This is host/contract validation only, not a physical VisionFive 2 boot or LAVA result.

[Run #40](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42972803734) completed successfully after the VisionFive 2 workflow gained a pull-request trigger; run #41 also passed against the latest kernel source commit with CTest **14/14** and board-contract source checks.

## Regression diagnosis

- RV64 run #140 failed during cross-build with Rust error E0603 because `supervisor_trap_stack_bounds` was private while `main.rs` called it. The visibility correction was validated by later successful builds, including #145 and #147.
- The first unknown-syscall test commit emitted a literal escaped newline in its diagnostic marker. The marker was corrected, then tightened further so PASS is emitted only after the U-mode probe checks the returned `-ENOSYS` value. Run #148 rebuilt and re-ran QEMU successfully with this two-stage assertion.
- The renderer warning was a genuine unused internal function, not a test failure. It was removed and the full Core workflow #153 passed afterward.
- No physical-board result is inferred from QEMU or host contract tests.

## Remaining work

- Only one U-mode task is currently exercised; the other 15 supervisor stack slots are allocated and checked but are not yet used by a second U-mode task.
- Add per-task user stacks and a two-U-task trap/resume test.
- Add per-hart stack ownership and SMP-safe scheduling/IPC.
- Expand U-mode GPR/CSR sentinels and negative S-mode trap, invalid-argument and invalid-user-address tests.
- Evolve the bootstrap Sv39 mapping into per-process address spaces and safe user-memory copy-in/copy-out.
- Integrate the Rust-native kernel syscall path with the host-tested C++ ABI only after the contract is stabilized.
- Validate physical SiFive Unmatched, VisionFive 2 and Lichee Pi 4A boot/timer behavior, including LAVA where available.
