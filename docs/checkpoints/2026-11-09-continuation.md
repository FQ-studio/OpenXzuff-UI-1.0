# Continuation checkpoint — 2026-11-09

## Repository inspected

Repository: `FQ-studio/OpenXzuff-UI-1.0`, branch `main`.

Inspection was performed against live GitHub file contents, not inferred from the previous chat summary.

## Verified current source state

- `kernel/src/trap.S`: existing 288-byte trap frame is allocated from the current `sp`; this supports the current S-mode-only paths and is not yet safe for U-mode traps.
- `kernel/src/trap.rs`: installs `stvec`; handles supervisor timer interrupt cause 5 and S-mode breakpoint cause 3; no U-mode ECALL cause 8 handler.
- `kernel/src/runtime.rs`: fixed two-task S-mode timer preemption runtime, not a U-mode process runtime.
- `Kernel/arch/riscv64/trap_dispatch.cpp`: host-tested U-mode ECALL policy, not connected to the Rust `no_std` kernel.
- `Kernel/include/openxzuff/kernel/riscv64/trap_frame.hpp` and Rust `TrapFrame` use the same 288-byte layout, but there is no shared cross-language ABI bridge.
- `.github/workflows/rv64-s-mode-kernel.yml`: builds RV64 and boots QEMU; current marker checks cover S-mode paths only.

## Work committed in this continuation

Added `docs/design/rv64-umode-trap-entry.md` in commit:

`7cbca86b4576e676949de00fc7872e0707466994`

The design specifies privilege-aware entry/return, per-hart supervisor trap stack, `sscratch` convention, 288-byte frame preservation, U-mode ECALL dispatch semantics, integration boundary, and required QEMU regression tests.

## Build/test status for this continuation

No source code or workflow was changed in this continuation. Therefore no new host build, RV64 cross-build, QEMU run, or GitHub Actions result is claimed. The previous checkpoint's run results are historical evidence for its recorded commits only, not evidence for this documentation commit.

## Next implementation step

Implement the privilege-aware U-origin/S-origin split in `kernel/src/trap.S` and corresponding `trap.rs` handler, with a dedicated supervisor trap stack and a QEMU U-mode ECALL probe. Preserve the existing S-mode breakpoint and two-task timer-preemption paths. Trigger the RV64 workflow and require all old and new serial markers before calling the integration validated.
