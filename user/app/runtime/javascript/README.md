# OpenXzuff JavaScript Runtime

Status: **host proof-of-concept added; not an OpenXzuff OS runtime yet**.

## Current host prototype

- Embeds QuickJS-NG v0.18.0 from a pinned upstream tag.
- Runs a JavaScript file in a fresh QuickJS runtime/context.
- Exposes only a small `print(...)` host function.
- Applies a per-runtime memory limit (default 16 MiB), stack limit (256 KiB), source-size limit (4 MiB), and execution deadline (default 1 second).
- Includes a normal execution smoke test and an infinite-loop timeout test.

Build and test:

```sh
cmake -S user/app/runtime/javascript -B build-js -DCMAKE_BUILD_TYPE=Release
cmake --build build-js --parallel 2
ctest --test-dir build-js --output-on-failure
```

Run:

```sh
build-js/xzuff-js path/to/script.js
build-js/xzuff-js --timeout-ms 500 --memory-mib 16 path/to/script.js
```

## Important boundaries

This is a **host-side proof-of-concept only**. It is not yet cross-compiled or tested for RV64, Gravity Kernel, or the OpenXzuff user-space ABI. It does not provide `fetch`, networking, TLS, timers, filesystem access, DOM/browser APIs, or YouTube.js compatibility. It must not be described as a runnable OpenXzuff application runtime.

Before adoption, the next gates are:
1. Verify host build and both CTest cases in GitHub Actions.
2. Audit QuickJS-NG platform dependencies and build it for the actual RV64 target/toolchain.
3. Run it in the intended RV64/QEMU environment only after the user-space process and memory ABI exists.
4. Add capability-checked service bindings only after the network, storage, timer and IPC service contracts are implemented.
5. Evaluate YouTube.js APIs independently; basic JavaScript execution is not evidence of YouTube.js compatibility.

QuickJS-NG is MIT-licensed. Preserve its upstream license and copyright notices when redistributing the engine. This repository fetches the pinned source during configuration rather than copying the engine source into this repository.

Upstream:
- https://github.com/quickjs-ng/quickjs/tree/v0.18.0
- https://quickjs-ng.github.io/quickjs/developer-guide/intro/
