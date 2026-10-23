# OpenXzuff native kernel

This directory begins the native-kernel implementation. The first supported development architecture is RV64; ARM64/AArch64 is a separate port. No Linux kernel or Flutter dependency is used.

## Current implementation status

- `kernel.c`: freestanding kernel entry and architecture-independent halt loop.
- `arch/riscv/entry.S`: minimal RV64 entry for an SBI/OpenSBI-style environment.
- `arch/arm64/entry.S`: minimal AArch64 entry stub for a future firmware-specific boot path.
- `linker-riscv.ld`: RV64 linker layout for a controlled test environment; the load address must match the chosen firmware/board setup.

These are bring-up foundations, not yet a complete bootable desktop OS. There is no memory manager, scheduler, interrupt controller, framebuffer driver, device tree parser, or Lua VM integration yet. Board boot must be verified with a real cross-toolchain and emulator/hardware before calling it bootable.

## Build tools

RV64: `riscv64-unknown-elf-gcc` and `riscv64-unknown-elf-ld`.
AArch64: `aarch64-none-elf-gcc` and a firmware/UEFI-compatible loader path.

The two architectures use separate entry code and must not share machine-code binaries.