# OpenXzuff UI 1.0

OpenXzuff UI is the open-source graphical user-interface layer planned for the Gravity Kernel ecosystem. It is maintained separately from the proprietary Xzuff UI.

## Project direction

- **UI language:** Lua.
- **Runtime:** a Lua VM embedded by the platform host; the VM is not bundled in this first foundation commit.
- **Graphics boundary:** a small host-provided graphics interface that can be implemented over a framebuffer.
- **Kernel target:** Gravity Kernel RV64, without requiring Linux or Flutter.
- **Initial scope:** a small retained-mode UI core, pointer/touch hit-testing, a desktop shell foundation, and automated Lua tests.

## Current status

This repository currently contains the first **Lua UI foundation**, not a bootable operating system. A bootable system still requires a real Lua VM port/build, a kernel-to-userspace or in-kernel host adapter, framebuffer discovery/mapping, input drivers, memory management, and a boot/test environment.

## Repository layout

- `src/openxzuff/ui.lua` — small UI core and button input dispatch.
- `src/openxzuff/desktop.lua` — desktop background, top panel, and application-menu example.
- `docs/architecture.md` — component boundaries and implementation sequence.
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

## Platform integration

The UI core does not make kernel syscalls directly. The host supplies the documented graphics functions and forwards normalized pointer/touch events. This keeps the UI independent of an unverified Gravity Kernel ABI. See [Platform API](docs/platform-api.md).

## License

MIT. See [LICENSE](LICENSE).
