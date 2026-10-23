# Implementation status

This document distinguishes code that exists from planned OS features.

## Present in this foundation

- Host-independent Lua widget core and desktop shell example.
- RV64 direct-payload entry and early SBI console message.
- Draft Gravity syscall IDs and C dispatch boundary. All calls currently return
  `-GRAVITY_ENOSYS`; there is no connected user-mode trap path yet.
- Rust lexical retrieval library with unit tests. It is not a SQL engine, durable
  database, embedding pipeline, or complete RAG service.
- C++ AI C-ABI boundary that explicitly reports the inference backend is not
  configured.
- Lua settings model and AI Assistant app shell.

## Not implemented yet

- Full microkernel: privilege transitions, trap/interrupt handling, scheduler,
  address spaces, allocator, capability enforcement, IPC endpoints and drivers.
- Bootable desktop, real framebuffer mapping, font renderer, keyboard/touch
  drivers, app lifecycle manager or compositor.
- SQL parser/storage engine, durable index, embedding model and vector search.
- LiteRT integration, compatible Qwen2.5-0.5B model packaging/tokenizer, and
  actual inference.
- ARM64 UEFI boot path.

## Safety and correctness rule

Do not label this repository as a complete or hardware-bootable OS until each
subsystem is implemented, tested, and its target hardware/boot protocol is
documented. Keep proprietary Xzuff UI code out of this open-source repository.
