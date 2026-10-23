# Implementation status

This document distinguishes code that exists from planned OS features.

## Present and testable in this branch

- Host-independent Lua widget core and desktop shell example.
- RV64 direct-payload entry and early SBI console message.
- Draft Gravity syscall IDs and C dispatch boundary. All calls currently return
  `-GRAVITY_ENOSYS`; there is no connected user-mode trap path yet.
- Rust SQL AI library backed by the real SQLite engine through rusqlite: schema
  creation, parameterized insert/upsert/delete, persistent database opening, and
  title-weighted lexical retrieval. Unit tests exercise its database operations.
- C++ AI C-ABI boundary that explicitly reports the inference backend is not
  configured.
- Lua settings model and AI Assistant app shell.

## Not implemented yet

- Full microkernel: privilege transitions, trap/interrupt handling, scheduler,
  address spaces, allocator, capability enforcement, IPC endpoints and drivers.
- Bootable desktop, real framebuffer mapping, font renderer, keyboard/touch
  drivers, app lifecycle manager or compositor; Lua VM is not embedded in kernel.
- SQL AI as a running Gravity user-space service, embeddings, vector search, or
  complete retrieval-augmented generation.
- Actual Google LiteRT backend, verified Qwen2.5-0.5B compatible model export,
  tokenizer/chat template integration, and real inference test.
- Aurora Store running on OpenXzuff. It is an Android Kotlin app and needs a
  tested Android compatibility runtime or a separate Android-based edition.
- ARM64 UEFI boot path.

## Verification commands

```sh
cargo test --manifest-path services/sql-ai/Cargo.toml
make check-ai-cpp
make check-lua
make kernel-rv64 RISCV_PREFIX=riscv64-unknown-elf-
```

## Acceptance rule

Do not label this repository as a complete or hardware-bootable OS, or claim an
AI/app integration works, until the real dependency is built and exercised by a
reproducible test on the declared target. Keep proprietary Xzuff UI code and
assets out of this open-source repository.
