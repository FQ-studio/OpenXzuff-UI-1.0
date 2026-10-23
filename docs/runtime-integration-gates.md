# Runtime integration gates: Aurora Store, LiteRT, Qwen, and SQL

This document is an implementation contract, not a claim that these integrations
already run on Gravity Kernel.

## 1. Gravity SQL AI — implemented baseline

The Rust crate now uses SQLite through rusqlite with the bundled SQLite library.
It creates a documents table, persists records, performs parameterized SQL
operations, and ranks matching passages with a deterministic lexical score.
Run its tests with:

```sh
cargo test --manifest-path services/sql-ai/Cargo.toml
```

Current limits: this is not yet a service process on Gravity Kernel, a vector
database, an embedding pipeline, or complete RAG. The service needs a defined
user-space ABI and storage-device/file-system support before it can run on the
native OS.

## 2. Google LiteRT

Official LiteRT packages and APIs target supported host platforms such as Android,
Linux, desktop systems, and web. A freestanding Gravity RV64 kernel is not itself
a supported drop-in target. Do not link host LiteRT binaries into the bare-metal
kernel or claim they work on RV64 without a compatible build.

Integration gates:
1. Choose a supported development host for inference tests.
2. Pin a LiteRT/LiteRT-LM version and build the C++ adapter against its real API.
3. Test model loading and at least one known input/output pair.
4. Separately evaluate whether the runtime can be ported to the eventual Gravity
   user-space ABI; this requires OS facilities and architecture support.

## 3. Qwen2.5-0.5B

The upstream Qwen2.5-0.5B weights are not automatically a LiteRT model. A real
integration must select a compatible export/conversion path, preserve the
matching tokenizer and chat template, verify supported operators, and run a
known-answer smoke test. Model files are large binary artifacts and should be
downloaded from the official model repository at build/deployment time rather
than committed into Git.

The AI C ABI must return a clear error until the actual runtime and compatible
model are loaded. Never return success merely because a model path exists.

## 4. Aurora Store

Aurora Store is an Android application (Kotlin) and an unofficial FOSS client
for Google Play; it does not own or redistribute the apps it displays. It cannot
run as a native Lua app just by copying its APK into this repository.

To provide Aurora Store on OpenXzuff OS, one of these must be implemented and
tested:
- a maintained Android application compatibility runtime, including Android
  framework APIs, package installation, permissions, and required services; or
- an Android-based edition/companion environment, which is a different product
  architecture from the current native-only target.

Until that exists, list Aurora Store as a planned compatibility target, not an
installed or working native application. Respect Aurora Store's own GPLv3
license and distribution instructions if its code is ever modified or bundled.

## Acceptance rule

A component is called integrated only after CI builds the real dependency and a
test executes its real API. A component is called usable on Gravity Kernel only
after the native OS supplies the required runtime, memory, filesystem, and
process facilities and a reproducible boot test demonstrates it.
