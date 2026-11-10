# RV64 U-mode trap implementation checkpoint — 2026-11-09

> Historical implementation checkpoint. Its initial CI state was pending when first written. The implementation has since advanced; the latest validated state is recorded in [2026-11-10 afternoon checkpoint](2026-11-10-afternoon.md).

## Initial implementation scope

The initial change introduced a privilege-aware U-origin/S-origin trap vector, a 288-byte trap frame, a supervisor-owned stack, and a Rust-native U-mode ECALL probe. It preserved the established S-mode trap path and connected the probe to the runtime.

The live implementation has since expanded beyond the original one-ECALL probe to include Sv39 isolation, non-blocking IPC, mailbox request/reply, Yield and Exit syscalls, and an explicit supervisor trap-stack/user-SP assertion. Refer to the current source and the 2026-11-10 checkpoint rather than treating the initial probe description as the complete current behavior.

## Initial source changes

- Added `kernel/src/umode_trap.S` with the `sscratch` stack exchange, S-origin fallback, U-origin register/CSR save, and Rust dispatch.
- Updated `kernel/src/trap.rs` to install the privilege-aware vector and create the initial U-mode task frame.
- Updated `kernel/src/runtime.rs` to schedule the U-mode test task alongside two S-mode tasks.
- Updated the RV64 workflow to require serial evidence rather than relying on process exit alone.

## Superseding validation — 2026-11-10

- Latest source commit: `cbc196338e124005ea95958a10c430d3f3ade3a0`.
- GitHub Actions run [#134](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42768638698): **completed / success**.
- Host tests: **21 passed, 0 failed**.
- RV64 cross-build: **success**, target `riscv64imac-unknown-none-elf`.
- QEMU OpenSBI boot: **success**; all required serial assertions passed, including explicit supervisor trap-stack frame placement and saved user-SP preservation, Sv39 isolation, Yield, Exit, IPC empty-queue and IPC request/reply checks.
- Artifact: [ELF, raw binary and QEMU serial log](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42768638698/artifacts/13741116624).

This validates the recorded QEMU test path only. It does not establish physical-board boot, SMP safety, a general multi-process address-space manager, or production OS stability.
