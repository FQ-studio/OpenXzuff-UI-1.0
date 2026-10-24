# OpenXzuff UI 1.0 — Architecture Status

## Current source layout

- `Kernel/`: kernel contracts and future architecture-specific kernel implementation.
- `sys/`: system service registry and future HAL/driver layers.
- `gui/`: software framebuffer and future window manager/desktop shell.
- `user/app/`: application manifest/example and planned Lua/Luau runtimes and installer.
- `runtime/`: current runtime-related source; review before moving or duplicating code.
- `graphics3d/`: existing 3D renderer and tests.
- `app-hub/`: planned App Hub server, uploader, and SQL schema.
- `boot/`, `boards/`: planned boot and board-support scaffolding.

## Truthful implementation status

A directory and README do not constitute a working subsystem. The current scaffold does not by itself provide a bootable RV64 OS, scheduler, full memory manager, hardware driver, desktop compositor, graphical package installer, or App Hub server. Existing host-side C++ tests must not be described as proof of physical-hardware boot.

## Package naming

- `.lap`: Luau App Package.
- `.luap`: Lua App Package.

Keep the two formats distinct and document their manifests and validation rules before implementing installation.
