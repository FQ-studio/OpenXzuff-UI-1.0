# 2026-11-21 — Xzuff Go, Gravity SDK and NDK checkpoint

## Scope

This checkpoint records the Xzuff Go host-tool, compiler fixture, Gravity SDK/NDK build path, and validation work in draft PR [#16](https://github.com/FQ-studio/OpenXzuff-UI-1.0/pull/16).

The code-test point for this checkpoint is commit `5c73a47bf2f8d44b9bfa02e5f6e4ae5d03b44abf` on branch `feat/xgo-cross-platform-host-cli`. PR #16 is not merged. Its base is the repository default `main` branch; the newer kernel/ABI work described below remains on the separate stacked FDT/kernel branch.

## Changes recorded

- Replaced shell-string execution in `xgo` with direct process launch: POSIX uses `fork/exec`; Windows uses `CreateProcessW` and Windows-compatible argument quoting.
- Windows tool lookup now uses the sibling `.exe` filenames.
- The CLI smoke test creates its project under a path containing spaces and checks that child Coder Checker output is forwarded.
- Added a Windows-host CI workflow.
- Added `language/xzuff-go/examples/rv64_return_42.xgo` as a source file accepted by the current compiler subset. The existing `examples/hello.xgo` remains a forward-looking syntax sketch and is not currently compilable.
- CMake and the Linux packaging workflow compile/package the supported RV64 example.

## CI evidence

- **Windows host:** run [44835674756](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44835674756) completed successfully. The compiler, CLI, Coder Checker and `.xexe` tests passed **9/9**, including the spaced-path CLI test and child-output forwarding. This runner is Windows x64, not a native Windows ARM64 runner.
- **Linux / Android / RV64 toolchain:** run [44835674586](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44835674586) completed the Xzuff-relevant steps successfully at the time this checkpoint was written: host build, CTest **16/16**, compiler-produced `.xexe` fixture and validation, RV64 freestanding C NDK build plus `ecall` disassembly check, Android NDK cross-build of `xgo`, `xzuff-go`, and `xzuff-check` for `arm64-v8a`, and packaging of both toolchain archives. The overall workflow had not yet finished its later, unrelated Luau/runtime build step when recorded; do not treat that as a completed whole-workflow PASS.
- **VisionFive 2 readiness contracts:** run [44835674404](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44835674404) completed successfully. This is a hosted CI contract/smoke check, not a physical-board boot.
- **RV64 QEMU evidence:** run [44743766492](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/44743766492) completed successfully on branch `feat/fdt-ranges-root-translation`, commit `77c074db409501096094ff90effb18a2876399db`. Its serial log records S-mode entry, Sv39 isolation/W^X, timer preemption and context switching, U-mode Yield/Exit, IPC, and the built-in probe's MapMemory/ProtectMemory/UnmapMemory checks. This QEMU run is evidence for that kernel branch, not for launching arbitrary `.xexe` applications from the compiler.

## Current implementation boundary

### Xzuff Go compiler and CLI

- The compiler accepts one `main` function returning a signed i32 integer literal.
- It emits RV64 instructions for the return value, Gravity ABI v1 Exit syscall setup in `a7`, and `ecall`, then wraps the code in a 32-byte `.xexe` v1 header.
- `xgo create/check/build` are implemented for the current project scaffold. `xgo run` currently checks `.xgo` test files; it does **not** execute a program or provide a runtime.
- Android ARM64 binaries are host tools intended for Termux. Their generated `.xexe` output targets RV64; it is not an Android executable. No on-device Termux execution was performed in this CI run.

### Gravity SDK and NDK

- Sysroot headers are under `sdk/gravity/sysroot/usr/include/gravity`.
- The current SDK wrappers cover Exit, SendMessage, ReceiveMessage and Yield. The compile-only `abi_smoke.c` exercises the calling convention.
- `ndk/gravity-rv64` is a CMake cross-toolchain for freestanding RV64 C object files using RV64IMAC/LP64. The object and its `ecall` instruction are checked in CI.
- This is not a complete NDK: C++ support, libc, CRT/startup, allocator integration, relocatable object processing, application linker, linker script, and application runner are absent.

### Gravity ABI and executable loading

- The 32-byte `.xexe` v1 format contains a single code payload. It does not yet describe data/BSS, relocations, memory size, argv/environment, or stack/heap startup.
- `Kernel/src/xexe_loader.cpp` validates the file structure; it is not a process loader and does not map or execute the image.
- The latest separate FDT/kernel branch extends the ABI contract with MapMemory (3), UnmapMemory (7), and ProtectMemory (8) for its built-in U-mode probe and bounded heap. Those operations are not yet a general per-process application-memory API, and the SDK does not expose them as general application wrappers.
- Therefore a production Gravity linker and runnable SDK-built application are intentionally gated on a general user-process loader, a defined segment/memory contract, and process startup/lifecycle support.

## Next technical gates

1. Define a versioned `.xexe` segment/data/BSS/relocation and startup contract that matches the kernel loader design.
2. Implement general per-process address spaces, user-memory ownership, process startup and teardown.
3. Add a linker/relocation pipeline and CRT/startup only after those contracts are stable.
4. Expand Xzuff Go grammar/code generation and add executable integration tests under QEMU before claiming `xgo run` or arbitrary SDK applications are runnable.
5. Add native Windows ARM64 and on-device Termux validation when those runners/devices are available.
