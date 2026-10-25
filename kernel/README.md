# Gravity Kernel — Rust / RV64

Gravity Kernel is the native kernel foundation for OpenXzuff UI. The current build target is **RV64 only**. AArch64 and x86_64 are not supported targets in this project milestone. No Linux kernel or Flutter dependency is used.

## Language boundary

- Kernel logic: Rust `no_std`
- Firmware entry and CPU trap entry: RV64 Assembly
- Lua Native OS: planned user-space runtime, not yet integrated into the kernel build

## Rust migration status

- `src/main.rs`: freestanding kernel entry, SBI console output, trap dispatch boundary and panic halt.
- `src/memory.rs`: bounded boot allocator and physical memory-region bookkeeping.
- `src/fdt.rs`: FDT RAM discovery, FDT memreserve handling and reserved-memory child handling.
- `src/syscall.rs`: draft Gravity syscall ABI, deliberately fail-closed until implementations and capability checks exist.
- `src/app.rs`: GAPP v1 metadata validation.
- `arch/riscv/entry.S` and `arch/riscv/trap.S`: assembly entry/trap boundary.

## Build and test

Host-side Rust module tests:

```sh
cargo test --manifest-path kernel/Cargo.toml
```

RV64 payload build (requires Rust stable with `riscv64gc-unknown-none-elf` and `riscv64-unknown-elf-gcc`):

```sh
rustup target add riscv64gc-unknown-none-elf
make clean kernel-rv64
```

## Important limitations

This is still an early bring-up payload, **not a complete microkernel or bootable desktop OS**. Task creation, scheduler/context switching, Sv39 address-space isolation, validated user-memory access, functional IPC/capabilities, real driver services, framebuffer/input services, and Lua VM integration are not implemented yet. The current syscall IDs are reserved ABI values and return `-ENOSYS`.

The trap assembly still requires a dedicated, trusted kernel-stack transition before enabling user tasks. Do not treat installation of `stvec` as proof that U-mode traps are safe. QEMU smoke tests validate only the checks they explicitly execute.
