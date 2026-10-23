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

Gravity SQL AI now uses the real SQLite engine through Rust's rusqlite binding
for persistent document storage and lexical retrieval. It is a working library
baseline, but it is not yet running as a service on Gravity Kernel and does not
yet include embeddings or vector search.

Google LiteRT, Qwen2.5-0.5B inference, and Aurora Store are **not yet integrated**.
LiteRT needs a compatible runtime/build target and Qwen needs a verified model
format, tokenizer, and inference smoke test. Aurora Store is an Android Kotlin
app and requires an Android compatibility runtime; it cannot run as a native Lua
app merely by copying its APK. See
[Runtime integration gates](docs/runtime-integration-gates.md).

## Components

- `kernel/`: RV64 early entry and draft Gravity syscall ABI.
- `src/openxzuff/`: host-agnostic Lua UI and desktop shell.
- `runtime/lua/apps/`: early Settings and AI Assistant app modules.
- `services/sql-ai/`: SQLite-backed Rust document store and lexical retrieval.
- `ai/runtime/`: C++ ABI boundary; no inference backend is linked yet.
- `docs/implementation-status.md`: implemented versus planned features.
- `docs/runtime-integration-gates.md`: explicit dependency and acceptance gates.

## Checks

```sh
make check-lua
make check-rust
make check-ai-cpp
make kernel-rv64 RISCV_PREFIX=riscv64-unknown-elf-
```

Keep proprietary Xzuff UI code and assets out of this open-source repository.
