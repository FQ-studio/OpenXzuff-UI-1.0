# RV64 U-mode trap implementation checkpoint — 2026-11-09

> Historical implementation checkpoint. Its original CI state was pending when first written. The implementation has since advanced; the validated follow-up is recorded in [2026-11-10 afternoon checkpoint](2026-11-10-afternoon.md).

## Initial implementation scope

The initial change introduced a privilege-aware U-origin/S-origin trap vector, a 288-byte trap frame, a supervisor-owned stack, and a Rust-native U-mode ECALL probe. It preserved the established S-mode trap path and connected the probe to the runtime.

The live implementation has since expanded beyond the original one-ECALL probe to include Sv39 isolation, non-blocking IPC, mailbox request/reply, Yield and Exit syscalls. Refer to the current source and the 2026-11-10 checkpoint rather than treating the initial probe description below as the complete current behavior.

## Initial source changes

- Added `kernel/src/umode_trap.S` with the `sscratch` stack exchange, S-origin fallback, U-origin register/CSR save, and Rust dispatch.
- Updated `kernel/src/trap.rs` to install the privilege-aware vector and create the initial U-mode task frame.
- Updated `kernel/src/runtime.rs` to schedule the U-mode test task alongside two S-mode tasks.
- Updated the RV64 workflow to require serial evidence rather than relying on process exit alone.

## Superseding validation — 2026-11-10

- Source commit: `cdd0f46b19ac4ca5b04040e3eedb0f264b4d82f5`.
- GitHub Actions run [#132](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725952037): **completed / success**.
- Host tests: **21 passed, 0 failed**.
- RV64 cross-build: **success**, target `riscv64imac-unknown-none-elf`.
- QEMU OpenSBI boot: **success**; all required serial assertions passed, including U-mode Sv39 isolation, Yield, Exit, IPC empty-queue and IPC request/reply checks.
- Artifact: [ELF, raw binary and QEMU serial log](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42725952037/artifacts/13723271109).

This validates the recorded QEMU test path only. It does not establish physical-board boot, SMP safety, a general multi-process address-space manager, or production OS stability.
