# 2026-11-28 — Xzuff Go, Gravity SDK and NDK checkpoint

## Repository and branch

- Repository: FQ-studio/OpenXzuff-UI-1.0
- Work branch: feat/xgo-cross-platform-host-cli
- Pull request: #16 — Make Xzuff Go CLI portable across POSIX and Windows hosts
- PR #16 remains a draft and is not merged into main.
- Compiler/ABI implementation commit: fd8a693ca783d1f90e03c8be78961cb83eb56e66
- Compiler parser build-fix commit: 9505fc6b71424791792ac671f9b5f89edb03b804

## Changes completed

### Xzuff Go compiler and Coder Checker

- Extended the parser from a single integer literal to a restricted integer constant-expression grammar.
- Added unary plus/minus, parentheses, multiplication, division, remainder, addition and subtraction.
- Added operator precedence and left associativity.
- Added checked signed 64-bit intermediate arithmetic, division/remainder-by-zero rejection, signed overflow rejection and final signed-i32 range checking.
- Kept expression evaluation as compile-time constant folding. The compiler does not yet emit dynamic arithmetic for variables or runtime values.
- Added examples/rv64_constant_expression.xgo, which evaluates to 33.
- Expanded the compiler self-test and Coder Checker CTest cases for precedence, unary operators, division by zero, i32 overflow, i64 overflow and malformed expressions.

### Gravity ABI and XEXE

- Centralized .xexe v1 magic, version, header size, RV64 architecture, ABI version, field offsets and reserved flags in sdk/gravity/sysroot/usr/include/gravity/abi.h.
- Updated the compiler emitter, kernel image validator and validator tests to use the shared constants.
- Extended the image test utility to compare compiler-produced RV64 Exit-stub bytes against an expected signed status, including the new expression fixture.

### SDK and NDK

- Added C11 static assertions in sdk/gravity/examples/abi_smoke.c for Gravity ABI v1, .xexe v1 header geometry and stable Exit/Yield syscall IDs.
- Kept the SDK wrapper boundary explicit: Exit, SendMessage, ReceiveMessage and Yield are the current general C wrappers in this branch.
- Kept the NDK as a freestanding RV64 C object toolchain. It is not yet a linkable application NDK.

## Verification evidence

- Windows host: [Xzuff Go Windows Host Smoke run #46081788865](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/46081788865) — SUCCESS on commit 9505fc6. CTest selected suite: 15/15 passed. This is a Windows x64 hosted runner, not a native Windows ARM64 runner.
- Linux / SDK / NDK / Android: [OpenXzuff UI Runtime run #46081788858](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/46081788858) — the following Xzuff-relevant steps completed successfully on commit 9505fc6: configure/build, OS contracts + Xzuff Go/Coder Checker tests, compiler-produced .xexe fixture, RV64 bare-metal C toolchain installation, Gravity RV64 NDK configure/build, Android SDK/NDK setup, Android ARM64 cross-compilation of xgo/xzuff-go/xzuff-check, and packaging of the Android ARM64 toolchain and Gravity RV64 SDK/NDK. The overall workflow was still running later unrelated Luau/JavaScript/Lua/3D jobs when this checkpoint was recorded; do not label the entire workflow PASS.
- VisionFive 2: [Readiness run #46081788860](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/46081788860) — SUCCESS for hosted build, strict-warning, CTest and board-contract checks. This is not a physical VisionFive 2 boot.
- Separate kernel/QEMU evidence: [RV64 S-Mode Kernel run #45969995127](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45969995127) — SUCCESS on the separate Gravity/kernel branch. It built the RV64 kernel ELF and completed the OpenSBI/QEMU boot with UART RX interrupt injection. This is not the same branch/commit as PR #16 and does not execute the compiler-produced .xexe application.

## Current implementation boundary

| Component | Verified state | Remaining boundary |
|---|---|---|
| Xzuff Go compiler | One main function returning an i32 constant expression; emits RV64 Exit stub | No variables, runtime arithmetic, functions, modules, IR or optimizer |
| Coder Checker | Shares compiler lexer/parser; expression and rejection tests pass | Not a full language type checker/linter |
| xgo CLI | create/check/build and source-smoke run; POSIX and Windows process launch tested | run does not execute compiled applications |
| Gravity ABI | Shared v1 syscall and .xexe constants | No general per-process memory/startup contract |
| .xexe v1 | 32-byte code-only image; compiler output and validator integration tested | No data/BSS, relocations, symbol table, linker or process loader |
| Gravity SDK | Sysroot headers and Exit/Send/Receive/Yield wrappers | No general application memory or file-service wrappers in this branch |
| Gravity NDK | RV64IMAC/LP64 freestanding C object build | No C++ runtime, libc, CRT, linker or application launcher |
| Android ARM64 host tools | xgo/compiler/checker cross-built and packaged for arm64-v8a | No physical Termux device test in this run |
| Windows host tools | Windows x64 compiler/CLI/validator tests passed | No native Windows ARM64 runner test |

## Next technical gates

1. Expand Xzuff Go beyond constant expressions: variables, runtime arithmetic, functions, typed AST/IR and diagnostics.
2. Freeze a versioned .xexe segment/data/BSS/relocation/startup contract before implementing a production linker.
3. Implement general per-process address spaces, executable loading, stack/heap setup, startup and teardown in Gravity Kernel.
4. Add C/C++ CRT and library support to the NDK only after the loader and executable contract are stable.
5. Add a QEMU test that loads and executes an actual compiler-produced .xexe; the current QEMU kernel smoke is separate.
6. Validate the Android ARM64 toolchain on Termux and add native Windows ARM64 CI when a suitable runner is available.

No 100% completion claim is made. The verified scope is the compiler's constant-expression subset, shared .xexe v1 validation, Windows/Linux host tests, RV64 C object compilation and Android ARM64 host-tool cross-build.
