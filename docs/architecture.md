# Architecture

## Scope and non-goals

OpenXzuff UI is the open-source GUI layer, not the kernel and not the proprietary Xzuff UI. The first implementation keeps the UI core small and host-agnostic. It does not claim to boot hardware or to have a working Gravity Kernel integration.

## Layers

1. **Gravity Kernel (external dependency):** boot, memory management, scheduling, device discovery, input drivers, and framebuffer mapping.
2. **Platform host adapter (to implement):** exposes a narrow graphics/input API to the Lua VM. It translates kernel-specific interfaces into stable UI calls.
3. **Lua VM (to port/package):** executes UI scripts. VM allocator, panic handling, module loading, and security policy belong to the platform integration.
4. **OpenXzuff UI core:** widgets, hit-testing, input dispatch, layout, and drawing commands.
5. **Desktop shell:** panel, application menu, status, and later window management.

## Current foundation

- `src/openxzuff/ui.lua`: button model, draw dispatch, reverse-order hit testing, pointer press/release and callback dispatch.
- `src/openxzuff/desktop.lua`: simple desktop shell sample using a host-supplied drawing API.
- `tests/ui_spec.lua`: tests against a fake host; no hardware is needed to test the core.

## Required platform API

The host must implement:

- `fill_rect(x, y, width, height, rgba)`
- `draw_text(text, x, y, rgba)`

The host must also call the shell's normalized pointer methods for press/release events. Coordinates are physical framebuffer pixels; the host is responsible for converting touch coordinates, display rotation, and device-specific events.

This is deliberately not a kernel syscall specification. The actual OpenXzuff kernel ABI and framebuffer/input APIs must be implemented and verified before writing a real adapter. See `kernel/README.md` and `docs/roadmap.md`.

## Roadmap

1. **Foundation (this branch):** Lua UI core, example shell, tests, CI, API documentation.
2. **Runtime choice:** pin a Lua version, build configuration, allocator strategy, and target-specific VM port.
3. **Gravity Kernel adapter:** use verified framebuffer, input, memory, and scheduling APIs from the kernel repository.
4. **Raster/text backend:** pixel format handling, clipping, bitmap or glyph font rendering, damage regions, and frame pacing.
5. **Desktop services:** windows, focus, keyboard events, app lifecycle, settings, and accessibility hooks.
6. **Hardware test:** boot on a documented RV64 target or emulator only after the required kernel facilities exist.

## Correctness constraints

- UI callbacks run synchronously in the host event context for now. The platform must avoid re-entrant input dispatch.
- Widget identifiers must be unique within a UI instance.
- Rectangles use non-negative width/height and finite coordinates.
- UI modules must not import Linux, Flutter, Android, or unverified kernel syscall bindings.
