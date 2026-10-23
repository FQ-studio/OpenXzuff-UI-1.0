# OpenXzuff UI 1.0

OpenXzuff UI is the open-source native UI and OS foundation from FQ Solo Studio.
It is distinct from the proprietary Xzuff UI project. Target RV64 first; ARM64 is
planned later. The design uses Assembly, Rust, C++, and Lua without depending on
Linux or Flutter as the target runtime.

## Current state

This repository is a development foundation, **not a complete or bootable OS**.
The current RV64 payload only reaches an early kernel entry and prints through
SBI. Gravity syscall IDs are reserved but not yet wired to a user-mode trap path.
The Lua UI core runs against a host adapter in tests; it is not yet embedded in a
bare-metal Lua VM.

The Rust Gravity SQL AI crate currently provides dependency-free lexical
retrieval as a starting point for RAG. The C++ AI boundary is a stub and does
not load LiteRT or Qwen2.5-0.5B yet.

## Components

- `kernel/`: RV64 early entry and draft Gravity syscall ABI.
- `src/openxzuff/`: host-agnostic Lua UI and desktop shell.
- `runtime/lua/apps/`: early Settings and AI Assistant app modules.
- `services/sql-ai/`: Rust lexical retrieval library with unit tests.
- `ai/runtime/`: C++ ABI boundary for future inference backend.
- `docs/implementation-status.md`: implemented versus planned features.

## Checks

```sh
make check-lua
make check-rust
make check-ai-cpp
make kernel-rv64 RISCV_PREFIX=riscv64-unknown-elf-
```

Keep proprietary Xzuff UI code and assets out of this open-source repository.
