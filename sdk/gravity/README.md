# Gravity SDK — RV64 ABI v1 starter

Status: **early SDK headers and compile-only example**. This is not yet a complete application SDK or NDK.

## Sysroot layout

The project-specific sysroot headers are under `sysroot/usr/include/gravity/`:

- `abi.h` — Gravity ABI v1 constants, syscall IDs, and negative errno values.
- `syscall.h` — freestanding RV64 inline wrappers for the currently specified syscall calling convention.

The header currently exposes Exit (0), SendMessage (1), ReceiveMessage (2), and Yield (4). MapMemory (3) is intentionally not wrapped as a working API because the live kernel reports it as `-ENOSYS`. ABI v1 reserves payload bit patterns that are interpreted as signed -1 through -4095, because those values collide with the syscall error channel in `a0`; the SDK rejects those values in `gravity_send_message`.

## Compile smoke example

`examples/abi_smoke.c` exercises the syscall wrappers. CI compiles it to an RV64 object with `riscv64-unknown-elf-gcc` and checks that the object contains `ecall`. This verifies header compatibility and RV64 code generation only; it does not link or execute an application.

## Toolchain roles

- The Android NDK workflow builds the `xgo`, `xzuff-go`, and `xzuff-check` host tools for Android ARM64 so they can run in Termux.
- `xzuff-go` emits RV64 Gravity `.xexe` machine-code images. Android ARM64 is the host-tool architecture, not the generated application's target architecture.
- The Gravity SDK sysroot is the target-facing header set for RV64 application code.

## Deliberate boundaries

A complete libc, CRT/startup objects, allocator, package manager, relocatable object linker, and production linker script are not present yet. No linker script is supplied because the current kernel has no general user-process loader or dynamic user address-space setup. The 32-byte `.xexe` v1 format describes a single code payload and does not yet describe data, BSS, relocations, or memory size.

The C++ `.xexe` validator in `Kernel/src/xexe_loader.cpp` is host-testable validation only; it does not load or execute an application. The separate `kernel/src/app.rs` GAPP metadata validator is not included by `kernel/src/main.rs` and is not the `.xexe` loader. These formats must be reconciled before a real application linker/loader is claimed.

The current RV64 runtime exercises a built-in U-mode probe task. The syscall wrappers are ABI-facing SDK groundwork, not proof that arbitrary SDK-built programs can already be launched by Gravity Kernel.
