# Architecture

OpenXzuff UI is the open-source desktop/UI layer for the native OpenXzuff OS
project. It is separate from proprietary Xzuff UI. The target architecture is
RV64 first, with ARM64 later. Linux and Flutter are not runtime dependencies.

## Layers

1. **Gravity Kernel (in progress):** RV64 entry point and early SBI console
   currently exist. Task scheduling, memory isolation, trap handling, IPC,
   capabilities and drivers remain to be implemented.
2. **Gravity syscall ABI:** names use `gravity_<group>_<operation>`. The current
   header reserves stable IDs; the dispatcher fails closed with `-ENOSYS`.
3. **Native services:** Rust for system services and Gravity SQL AI retrieval;
   C++ for optional native inference backends.
4. **Lua VM and platform host (not integrated):** the Lua UI core is host
   agnostic. A real VM allocator, module loader, syscall/IPC bridge, framebuffer
   and input adapter must be implemented.
5. **OpenXzuff UI:** widgets, hit-testing, input dispatch and drawing commands.
6. **Apps:** Lua app modules are loaded by a future application host. The current
   settings and AI Assistant files are app shells, not installed/runnable OS apps.

## Language responsibilities

- Assembly: architecture entry, trap entry/return and context-switch primitives.
- Rust: memory-safe kernel/services where the chosen ABI and no_std support fit.
- C++: native AI backend integration behind a stable C ABI.
- Lua: desktop shell, widgets and sandboxed user apps.

## AI and retrieval

Gravity SQL AI starts as a Rust retrieval library. Its current lexical search
provides candidate passages for RAG; it is not yet a SQL database or vector
database. A separate C++ service boundary is reserved for LiteRT integration.
Qwen2.5-0.5B is not assumed to run on LiteRT until its model format, tokenizer,
operators and backend compatibility have been verified.

## Current UI host API

The host implements `fill_rect(x, y, width, height, rgba)` and
`draw_text(text, x, y, rgba)`, plus normalized pointer press/release callbacks.
See `docs/platform-api.md`.

## Implementation roadmap

1. Finish RV64 trap entry and privilege-separated syscall dispatch.
2. Implement scheduler, physical/page memory management, capabilities and IPC.
3. Add serial/framebuffer/input drivers and boot protocol for a named board.
4. Port Lua VM as a bounded user task and connect UI drawing/events via IPC.
5. Build the app manager and lifecycle/isolation model.
6. Grow Gravity SQL AI from lexical retrieval to persistence, SQL queries,
   embeddings, vector retrieval and source-grounded context.
7. Verify LiteRT backend and Qwen model compatibility before enabling inference.
8. Add ARM64 UEFI boot only after RV64 interfaces stabilize.
