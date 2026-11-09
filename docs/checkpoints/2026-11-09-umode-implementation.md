# RV64 U-mode trap implementation checkpoint — 2026-11-09

## Scope implemented

This continuation adds the first code path for a real U-mode ECALL round-trip while preserving the existing S-mode trap vector and timer scheduler.

### Source changes

- Added `kernel/src/umode_trap.S`.
  - Uses `sscratch` as the per-entry supervisor trap-stack top while in U-mode.
  - Exchanges `sp` and `sscratch` as the first trap instruction.
  - If the exchanged supervisor stack pointer is zero, restores the original S-mode stack and tail-calls the established S-mode vector.
  - For U-origin traps, allocates the 288-byte frame on the supervisor stack, saves all integer GPRs plus `sepc`, `sstatus`, `scause`, and `stval`, and saves the user stack pointer from `sscratch`.
  - Calls the Rust U-mode trap handler, rearms `sscratch` with the supervisor stack top, restores the saved frame, and executes `sret`.
  - Adds a small U-mode probe that executes `ecall` then `ebreak`.

- Updated `kernel/src/trap.rs`.
  - Installs the privilege-aware vector and initializes `sscratch=0` for S-mode operation.
  - Adds dedicated static U-mode and supervisor trap stacks.
  - Builds an initial frame with SPP=0 and enters the probe through the existing `sret` restore path.
  - Implements a Rust-native U-mode ECALL test ABI: syscall ID in `a7`, argument in `a0`, return in `a0`.
  - Requires U-origin privilege state and expected test values; advances `sepc` by four for the returning ECALL.
  - Emits a QEMU serial PASS marker only after the U-mode probe reaches its post-ECALL breakpoint.

- Updated `kernel/src/runtime.rs` to launch the U-mode probe after the existing S-mode timer-preemption smoke path.
- Updated `.github/workflows/rv64-s-mode-kernel.yml` to require the U-mode ECALL/supervisor-stack/`sret` serial marker.

## Important validation boundary

These are source changes, not a test result. The GitHub commit-status query currently returned an empty status list for the latest source commit. No new host build, RV64 cross-build, QEMU boot, or GitHub Actions PASS is claimed in this checkpoint.

## Risks and required CI review

- Confirm the Rust toolchain accepts the inline/global assembly and that the linker includes the new assembly object.
- Confirm QEMU/OpenSBI starts U-mode with the configured memory protection behavior.
- Confirm the U-mode probe can access its mapped stack/code under the current Bare translation setup.
- Confirm the U-origin frame uses the supervisor stack and preserves the user `sp`.
- Confirm the S-origin fallback still passes the existing S-mode breakpoint round-trip and timer-preemption tests.
- If CI fails, inspect the exact compiler or QEMU log before changing code; do not infer the failure cause.

## Next actions

1. Obtain the GitHub Actions run for the latest push and inspect the complete build/QEMU log.
2. Fix any assembler, linker, privilege-return, or QEMU failure.
3. Require both legacy S-mode markers and the new U-mode ECALL marker.
4. Only after successful logs, update the status to validated and record run/artifact IDs.
