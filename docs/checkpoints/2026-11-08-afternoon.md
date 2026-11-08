# Work-session checkpoint — 2026-11-08 afternoon

## Repository state
- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `main`
- Validated code commit: `f204e521d00784418eff3f4fd7d0b1d879b2135d`
- GitHub connection: read/write; branch update succeeded.
- This checkpoint records evidence observed during the afternoon session. A CI run still in progress is explicitly not counted as passing.

## Changes committed in this session
Commit: `f204e521d00784418eff3f4fd7d0b1d879b2135d`
Message: `test(kernel): cover FDT parsing and trap ABI preservation`

1. `kernel/allocator-tests/src/lib.rs`
   - Added the production FDT parser module to the host test crate.
   - Added synthetic DTB tests for RAM `reg` parsing and the FDT memory-reservation map.
   - Added negative tests for bad magic, out-of-bounds structure offsets, and a truncated reservation map.
   - Existing physical-page and heap allocator tests remain in the same crate.
2. `tests/rv64_trap_dispatch_test.cpp`
   - Expanded the U-mode `ecall` test to verify that `a1..a5` remain unchanged across returning calls.
   - Added an unknown-syscall case verifying `-ENOSYS` and a four-byte saved-PC advance.
   - Retained checks for Yield, non-returning Exit, and rejecting an S-mode ecall in the U-mode dispatch boundary.
3. `Kernel/README.md`
   - Corrected stale scaffold-only wording.
   - Documents the implemented host-testable syscall/trap policy boundary and SBI wrapper.
   - Explicitly distinguishes the C++ contract layer from the separate Rust S-mode boot image and lists the missing hardware trap path, scheduler, page tables, and process execution.

## New GitHub Actions evidence for commit f204e521
- VisionFive 2 Kernel Readiness run #18: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42454318861
  - Result: `success`.
  - Strict-warning host build completed.
  - CTest: 14/14 passed, 0 failed.
  - This is host/contract readiness only; it is not a physical VisionFive 2 boot.
- RV64 S-Mode Kernel run #53: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42454318803
  - Result: `success`.
  - Host allocator/FDT test crate: 12 passed, 0 failed.
  - RV64 target build produced `openxzuff-kernel.elf`; the workflow's build step passed.
  - OpenSBI/QEMU boot passed the required serial checks:
    - `RV64 S-mode entry: OK`
    - `FDT parser: OK`
    - `FDT memory map: parsed`
    - `Physical page allocator: OK; managed pages: 4000`
    - `PPA alloc/free smoke test: PASS`
    - `Kernel heap allocator: OK; arena bytes: 65536`
    - `Heap alloc/free smoke test: PASS`
  - Artifact: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42454318803/artifacts/13607306127
- OpenXzuff UI Runtime run #136: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42454318927
  - At checkpoint time: `in_progress`, paused at the `Build Luau runtime` step according to the Actions job-step API.
  - Root configure/build, root OS/compiler/Coder Checker test step, RV64 .xexe fixture step, Android SDK/NDK setup, Android ARM64 cross-compile, and toolchain packaging were marked completed successfully.
  - Luau source/package smoke tests and JavaScript/Lua/3D prototype tests had not yet completed in this run. Do not treat run #136 as passing until GitHub reports a final conclusion.
  - The job log endpoint was not yet available while the job remained in progress; no failure diagnosis is claimed.

## Additional verified regression context
- Runtime workflow run #135 on commit `35b4d5f59eb9e37a6ac83bbb263d5697d8c5da54` completed successfully: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42437351605
- Its Luau package smoke test passed after the earlier `.lap` magic-length fix, and the JavaScript host, standard Lua host, OpenXzuff3D geometry/render tests and artifact upload completed successfully.
- The morning `.lap` regression is therefore verified by a post-fix package smoke test in run #135; run #136 is a further regression run, still pending at this checkpoint.

## Scope and limitations
- RV64-only target remains in force.
- The Rust image under `kernel/` boots in QEMU S-mode and initializes FDT memory, the physical page allocator, and the 64 KiB heap.
- The C++ U-mode trap dispatcher remains a host-testable policy boundary. It is not wired to a real `stvec` entry or `sret` path.
- No user-mode process execution, real trap register-save/restore assembly, context switching, scheduler, page-table isolation, hardware interrupt path, physical VisionFive 2 boot, or device-driver validation is claimed.
- OpenSBI firmware workflows were not rerun in this session because no OpenSBI/board firmware source path was changed. The latest observed successful OpenSBI workflow remains run #4 from 2026-10-29: https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/40870619576

## Next actions
1. Recheck run #136. If it fails, inspect the completed job log and fix only the evidenced failure; if it succeeds, record the final result.
2. Keep the next kernel milestone separate from the host contract: design and test a real RV64 trap entry/return path before claiming U-mode execution.
3. Continue scheduler/context-switch and page-table work only after the trap-frame and privilege-transition boundary is specified and tested.
