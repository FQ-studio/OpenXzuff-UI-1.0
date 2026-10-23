# OpenXzuff UI 1.0

OpenXzuff UI is the open-source graphical user-interface layer planned for the Gravity Kernel ecosystem. It is maintained separately from the proprietary Xzuff UI.

## Project direction

- **UI language:** Lua.
- **Runtime:** a Lua VM embedded by the platform host; the VM is not bundled in this first foundation commit.
- **Graphics boundary:** a small host-provided graphics interface that can be implemented over a framebuffer.
- **OS direction:** a native kernel with RV64 first and a separate ARM64/AArch64 port; no Linux or Flutter dependency.
- **Initial scope:** a small retained-mode UI core, pointer/touch hit-testing, a desktop shell foundation, and automated Lua tests.

## Current status

This repository now contains a Lua UI foundation plus initial freestanding kernel entry stubs and architecture/board plans. It is **not yet a bootable operating system**. The RV64 kernel skeleton and AArch64 entry stub have not yet been verified on hardware or QEMU; a real Lua VM port, framebuffer/input drivers, memory management, device discovery, and firmware-specific boot integration are still required.

## Repository layout

- `src/openxzuff/ui.lua` — small UI core and button input dispatch.
- `src/openxzuff/desktop.lua` — desktop background, top panel, and application-menu example.
- `kernel/` — minimal freestanding kernel entry foundations for RV64 and AArch64.
- `boards/` — board-specific bring-up requirements.
- `boot/` — documented firmware boot integration status.
- `runtime/` and `compat/` — planned Lua, minimal ART research and Win32 Basic subset.
- `docs/architecture.md` and `docs/roadmap.md` — component boundaries and phased implementation.
- `docs/platform-api.md` — contract expected from the graphics/input host.
- `tests/ui_spec.lua` — dependency-free Lua tests.
- `.github/workflows/lua.yml` — syntax and unit-test checks.

## Run tests locally

With Lua 5.4 installed:

```sh
lua tests/ui_spec.lua
```

Syntax-check all Lua sources:

```sh
find src tests -name '*.lua' -print0 | xargs -0 -n1 luac -p
```

## Native kernel build foundation\n\nThe `kernel-rv64` Make target requires `riscv64-unknown-elf-gcc`. It produces a minimal ELF payload for a controlled SBI-style bring-up environment; it does not provide a desktop, drivers, or a general-purpose boot image. ARM64 PC support is planned around UEFI and requires a separate EFI loader.\n\n## Platform integration

The UI core does not make kernel syscalls directly. The host supplies the documented graphics functions and forwards normalized pointer/touch events. This keeps the UI independent of an unverified Gravity Kernel ABI. See [Platform API](docs/platform-api.md).

## License

MIT. See [LICENSE](LICENSE).
