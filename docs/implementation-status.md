# Implementation status

This document distinguishes code that exists from planned OS features.

## Present and testable in this branch

- Host-independent Lua widget core and desktop shell example.
- RV64 direct-payload entry and early SBI console message.
- Draft Gravity syscall IDs and C dispatch boundary. All calls currently return
  \`-GRAVITY_ENOSYS\`; there is no connected user-mode trap path yet.
- Rust SQL AI library backed by the real SQLite engine through rusqlite: schema
  creation, parameterized insert/upsert/delete, persistent database opening, and
  title-weighted lexical retrieval. Unit tests exercise its database operations.
- C++ AI C-ABI boundary that explicitly reports the inference backend is not
  configured.
- Host-side Qwen2.5-0.5B inference smoke-test script using Transformers. It is
  not evidence of Qwen running on Gravity Kernel or through LiteRT.
- LiteRT-LM CLI runner that invokes the real \`litert lm run\` command when given
  a verified model bundle. No native Gravity LiteRT backend is integrated.

## Not implemented yet

- Full microkernel: privilege transitions, trap/interrupt handling, scheduler,
  address spaces, allocator, capability enforcement, IPC endpoints and drivers.
- Bootable desktop, real framebuffer mapping, font renderer, keyboard/touch
  drivers, app lifecycle manager or compositor; Lua VM is not embedded as a
  Gravity user process.
- SQL AI as a running Gravity user-space service, embeddings, vector search, or
  complete retrieval-augmented generation.
- Native Google LiteRT integration and a verified Qwen2.5-compatible LiteRT-LM
  model bundle.
- **ART Minimal:** no ART, libcore/Bionic, Android framework compatibility
  services, APK installer or Android graphics/audio/input bridge is integrated.
  Aurora Store cannot run on the native OS until sufficient Android compatibility
  exists; an APK alone is not enough.
- **Win32 Basic:** no PE loader, Win32 API/DLL layer or Windows app runtime exists.
- ARM64 UEFI boot path.

## Verification commands

\`\`\`sh
cargo test --manifest-path services/sql-ai/Cargo.toml
make check-ai-cpp
make check-lua
make kernel-rv64 RISCV_PREFIX=riscv64-unknown-elf-
\`\`\`

## Acceptance rule

Do not label this repository as a complete or hardware-bootable OS, or claim an
AI/app integration works, until the real dependency is built and exercised by
a reproducible test on the declared target. Compatibility claims must name the
tested API profile and apps. Keep proprietary Xzuff UI code and assets out of
this open-source repository.
