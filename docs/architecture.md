# Architecture

OpenXzuff UI is the open-source desktop/UI layer for the native OpenXzuff OS
project. It is separate from proprietary Xzuff UI. The target architecture is
RV64 first, with ARM64 later. Linux and Flutter are not target runtime dependencies.

## Layers

1. **Gravity Microkernel (in progress):** RV64 entry and early SBI console
   currently exist. Task scheduling, memory isolation, trap handling, IPC,
   capabilities and drivers remain to be implemented.
2. **Gravity syscall ABI:** names use \`gravity_<group>_<operation>\`. The current
   header reserves stable IDs; the dispatcher fails closed with \`-ENOSYS\`.
3. **Native services:** Rust for system services and SQL storage/retrieval; C++
   for optional native inference backends.
4. **Lua VM and platform host (not integrated):** the Lua UI core is host
   agnostic. A real VM allocator, module loader, syscall/IPC bridge, framebuffer
   and input adapter must be implemented.
5. **OpenXzuff desktop:** widgets, hit-testing, input dispatch and drawing commands.
6. **Compatibility subsystems (planned):**
   - **ART Minimal:** Android APK compatibility for selected daily-use apps and
     games. ART executes DEX bytecode, but useful app compatibility also needs
     appropriate Android framework APIs, Bionic/libcore, graphics/audio/input
     interfaces, package installation, permissions and native-library support.
     It is not enough to copy ART or an APK into the OS.
   - **Win32 Basic:** a deliberately limited Windows API profile for selected
     Win32 desktop applications. It needs PE loading, required DLL/API
     implementations, filesystem/process/thread support and graphics integration.
     Wine may be evaluated as an upstream reference, but its POSIX assumptions
     mean it is not a drop-in microkernel service.
7. **Apps:** Lua app modules are loaded by a future application host. The current
   settings and AI Assistant files are app shells, not installed/runnable OS apps.

## Language responsibilities

- Assembly: architecture entry, trap entry/return and context-switch primitives.
- Rust: memory-safe kernel/services where the chosen ABI and no_std support fit.
- C++: native AI backend integration behind a stable C ABI.
- Lua: desktop shell, widgets and sandboxed user apps.

## AI and retrieval

Gravity SQL AI uses SQLite for persistent document storage and lexical retrieval
in the host-tested Rust library. It is not yet a running Gravity user-space
service or complete RAG system. The C++ boundary for LiteRT is present, but the
native backend is not yet linked. A real host-side Qwen2.5 inference smoke test
exists; it does not prove Qwen runs on Gravity or is compatible with LiteRT-LM.
Verify model format, tokenizer, operators and backend compatibility before
connecting these pieces.

## Current UI host API

The host implements \`fill_rect(x, y, width, height, rgba)\` and
\`draw_text(text, x, y, rgba)\`, plus normalized pointer press/release callbacks.
See \`docs/platform-api.md\`.

## Implementation roadmap

1. Finish RV64 trap entry and privilege-separated syscall dispatch.
2. Implement scheduler, physical/page memory management, capabilities and IPC.
3. Add serial/framebuffer/input drivers and boot protocol for a named board.
4. Port Lua VM as a bounded user task and connect UI drawing/events via IPC.
5. Build the app manager and lifecycle/isolation model.
6. Complete SQL service with query execution, embeddings, vector retrieval and
   source-grounded context.
7. Verify LiteRT backend and Qwen model compatibility before enabling inference.
8. Design ART Minimal as a separate compatibility subsystem after process,
   memory, file, IPC, graphics and audio interfaces exist; validate APKs by
   compatibility tier, including games separately from daily-use apps.
9. Define Win32 Basic API coverage and prototype PE loading/API shims only after
   native process, memory, filesystem and graphics services are usable.
10. Add ARM64 UEFI boot after RV64 interfaces stabilize.

## Acceptance rule

A target is “bootable” only after the image is built reproducibly and the kernel
reaches a visible, testable milestone in QEMU or real hardware. Documentation
and stubs do not count as boot support. Compatibility is claimed per tested app
and API profile, not by the mere presence of a runtime folder.
