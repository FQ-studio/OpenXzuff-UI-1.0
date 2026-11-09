# Work-session checkpoint — 2026-11-09 afternoon

## Repository state

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- Validated kernel code commit: `f4e2d70cc78e2a8cc4ee7e920ceea348175cea24`
- RV64 validation: [RV64 S-Mode Kernel run #77](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42571213556) — success.
- Host/OS/runtime validation: [OpenXzuff UI Runtime run #138](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42571060146) — success.
- VisionFive 2 contract/host validation: [VisionFive 2 Kernel Readiness run #21](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42571060135) — success; this is not a physical-board boot test.

## Afternoon changes committed

1. `.github/workflows/rv64-s-mode-kernel.yml`
   - Replaced an accidental literal backslash-n between two shell commands with separate YAML block lines.
   - Made the QEMU job independently require the S-mode trap-vector installation, GPR save/restore + `sret`, cooperative scheduler state, and timer-preemption/context-switch markers.
   - Runs #74, #75, and #77 show these grep checks executed as separate commands and succeeded.

2. `kernel/src/scheduler.rs`
   - Corrected the module contract to describe the round-robin task-state scheduler used by the RV64 timer-trap runtime.

3. `Kernel/README.md`
   - Documented the fixed two-task S-mode SBI TIME preemption smoke runtime and explicitly bounded what the QEMU result proves.

4. `kernel/src/main.rs`
   - Added boot-time physical-page allocator accounting checks: managed and available counts must match the initial page count, and alloc/free must restore the free count.

5. `kernel/src/heap_alloc.rs`
   - Removed the unused `HeapError::Capacity` variant. Heap allocation failure is currently reported by the existing `Option` return type.

## Verified RV64 build and QEMU evidence

Run #77, commit `f4e2d70...`:

- Host Rust allocator/scheduler/FDT tests: **17 passed, 0 failed**.
- RV64 cross-build produced a statically linked 64-bit RISC-V ELF:
  `openxzuff-kernel.elf: ELF 64-bit LSB executable, UCB RISC-V, RVC, soft-float ABI`.
- OpenSBI entered Gravity Kernel in S-mode under QEMU `virt`.
- Serial log markers verified by individual workflow grep commands:
  - `RV64 S-mode entry: OK`
  - `FDT parser: OK` and `FDT memory map: parsed`
  - `Physical page allocator: OK; managed pages: 4000`
  - `PPA alloc/free smoke test: PASS`
  - `Kernel heap allocator: OK; arena bytes: 65536`
  - `Heap alloc/free smoke test: PASS`
  - `S-mode trap GPR save/restore + sret: PASS`
  - `Cooperative scheduler state test: PASS`
  - `RV64 timer preemption + stack context switch: PASS; interrupts=2`
- The final RV64 run reported no Rust dead-code warnings after the accounting checks and unused heap error variant removal. GitHub action-runtime deprecation notices remain external to the kernel build.
- Artifact: [openxzuff-gravity-kernel-rv64-s-mode](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42571213556/artifacts/13652469491).

## Verified host and runtime evidence

OpenXzuff UI Runtime run #138:

- Root CMake host build completed.
- Root CTest: **14/14 passed**, including OS smoke, Xzuff Go compiler/CLI, Coder Checker, .xexe validation, SBI wrapper, syscall dispatch, RV64 trap-frame layout, and host-testable U-mode trap dispatch.
- Xzuff Go generated a 44-byte Gravity Kernel RV64 .xexe; machine-code/format validation passed.
- Android NDK 27.2.12479018 cross-build for arm64-v8a completed and the toolchain package was uploaded.
- Luau runtime built; source execution and .lap pack/run smoke tests passed.
- JavaScript host tests: **2/2 passed**.
- Standard Lua host tests: **2/2 passed**.
- OpenXzuff3D tests: **1/1 passed**; render smoke test produced a non-empty scene.ppm.
- Artifact: [openxzuff-ui-1.0-smoke-artifacts](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42571060146/artifacts/13653521891).

VisionFive 2 Kernel Readiness run #21 separately built the host CMake targets and passed **14/14** CTest tests plus the source/documentation contract checks. Its runner is ubuntu-24.04; it does not boot a physical VisionFive 2.

## CI failures diagnosed during the morning integration

- Runs #68–#72 failed while the new runtime module was being integrated. The decisive compiler error showed a literal backslash-n in kernel/src/main.rs (`mod scheduler; backslash-n mod runtime;`), not a Rust module or ABI defect. The source was corrected; subsequent RV64 runs #73–#77 built and booted successfully.
- The RV64 workflow also contained a literal backslash-n between grep commands. It caused confusing grep file-name errors and meant the timer marker was not independently enforced. The YAML was corrected; runs #74, #75, and #77 show separate marker checks passing.

## Scope and unresolved work

The verified timer-preemption path is a fixed **two-task S-mode QEMU smoke runtime**. It demonstrates SBI TIME timer interrupts, saved trap-frame switching between two independent S-mode task stacks, and return through sret. It is not a complete production scheduler.

Still not implemented or not verified:

- U-mode trap entry and sscratch switch to a supervisor-owned trap stack.
- U-mode ecall wired from the Rust trap handler into the C++ syscall dispatcher.
- User-process execution, process lifecycle, and general task creation/exit/block/wake integration.
- Page tables, address-space isolation, page faults, and IPC transport.
- Timer operation on a physical VisionFive 2 and physical-board boot.
- LAVA hardware test on an actual board.

## Next checkpoint target

Implement and test the U-mode trap-entry contract, including a supervisor-owned stack switch, before connecting the existing host-tested syscall dispatcher. Require an independent RV64 QEMU ecall/return test and keep user-mode claims separate from the existing S-mode timer-preemption result.
