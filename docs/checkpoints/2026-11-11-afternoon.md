# Work-session checkpoint — 2026-11-11 afternoon

## Repository and revision

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- Kernel implementation commit validated: `4eae95c5366314927959f0fe1c60eff8f1f8ad25`
- Core renderer cleanup commit validated: `3e8f1ed6c8b3725085391a0e39f4e9cbb4e1250a`

## Changes made

1. `graphics3d/src/renderer.cpp`: removed the unused internal `clamp01` helper. It had no call sites and generated a non-fatal unused-function warning.
2. `kernel/src/trap.rs`: replaced the single static supervisor trap stack with 16 page-aligned slots indexed by scheduler `TaskId`. Initial U-mode trap-frame construction selects the task's slot; trap validation obtains the currently scheduled task ID and checks the frame belongs to that slot before dereferencing it.
3. `kernel/src/runtime.rs`: passes the U-mode task ID when creating its initial frame and exposes the current task ID for trap ownership validation.
4. `kernel/src/main.rs`: checks stack-slot alignment/contiguity and verifies all 16 stack pages remain supervisor-only in the user page table.
5. `.github/workflows/rv64-s-mode-kernel.yml`: requires the new per-task trap-stack serial marker.

## RV64 build and QEMU evidence

[RV64 S-Mode Kernel run #145](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971914049) completed **successfully** on 2026-11-11 for the kernel implementation commit above.

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
  - `RV64 callee-saved register + private stack preservation: PASS`
  - `RV64 IPC empty receive -EAGAIN: PASS`
  - `RV64 IPC mailbox send/receive round-trip: PASS`
  - `U-mode Yield syscall context switch + Sv39 isolation: PASS`
  - `U-mode Exit syscall + scheduler task exit: PASS`

The workflow has an individual `grep -q` assertion for the new stack marker and the existing runtime markers. [Run #145 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971914049/artifacts/13833448650) contains the ELF, raw binary and serial log.

## OpenXzuff UI Core host evidence

[OpenXzuff UI Runtime, OS Contracts and 3D run #153](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971393352) completed **successfully** on 2026-11-11 for the renderer cleanup commit.

- Root CTest: **14/14 passed**.
- Xzuff Go compiler and `.xexe` fixture validation: passed.
- Xzuff Go / xgo Android ARM64 cross-compile and packaging with NDK 27.2.12479018: passed.
- Luau source and `.lap` package smoke tests: passed.
- JavaScript host tests: **2/2**; standard Lua host tests: **2/2**; OpenXzuff3D tests: **1/1**; rendered-scene artifact verification: passed.
- The unused renderer helper warning is gone. Two non-fatal unused-parameter warnings remain in vendored Luau `Ast.h`; GitHub Actions also reports Node.js 20 action deprecation notices.
- [Run #153 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971393352/artifacts/13833493328).

## VisionFive 2 host contract baseline

[VisionFive 2 Kernel Readiness run #36](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42925793943) passed CMake build, strict-warning compilation, CTest **14/14**, and board boot-contract source checks. This is host/contract validation only. It is not a physical VisionFive 2 boot or LAVA result.

The board-readiness workflow previously watched only `kernel/src/fdt.rs` from the lowercase Rust kernel tree. The workflow path filter is being broadened to `kernel/**` so future Rust kernel changes trigger its host contract checks. The Core host workflow is likewise being broadened to include `kernel/**`.

## Regression history

- RV64 run #140 failed during cross-build with Rust error E0603 because `supervisor_trap_stack_bounds` was private while `main.rs` called it. The follow-up visibility correction was validated by later successful runs; the new task-indexed implementation in #145 also builds and boots.
- The renderer warning was a genuine unused internal function, not a test failure. It was removed and the full Core workflow #153 passed afterward.
- No physical-board result is inferred from QEMU or host contract tests.

## Remaining work

- Only one U-mode task is currently exercised; the other 15 supervisor stack slots are allocated and checked but not used by a second U-mode task.
- Add per-task user stacks and a two-U-task trap/resume test.
- Add per-hart stack ownership and SMP-safe scheduling/IPC.
- Expand U-mode GPR/CSR sentinels and negative trap/syscall/user-address tests.
- Evolve the bootstrap Sv39 mapping into per-process address spaces and safe user-memory copy-in/copy-out.
- Integrate the Rust-native kernel syscall path with the host-tested C++ ABI only after the contract is stabilized.
- Validate physical SiFive Unmatched, VisionFive 2 and Lichee Pi 4A boot/timer behavior, including LAVA where available.
