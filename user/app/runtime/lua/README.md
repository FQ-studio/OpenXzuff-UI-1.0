# OpenXzuff standard Lua Runtime

Status: host proof-of-concept source added; CI build/test result is pending. This runtime is separate from the Luau runtime in `runtime/`.

## Runtime contract

- Standard Lua 5.5.1, pinned to the official source archive and SHA-256 checksum.
- `.luap` and `.lua` are treated as standard Lua source files in this first host prototype.
- `.lap` remains owned by the Luau runtime; this runtime rejects it.
- Exposes a restricted set of standard libraries: base, table, string, math, UTF-8 and coroutine.
- Does not open the I/O, OS, package or debug libraries.
- Removes `dofile`, `loadfile` and `load` from the base environment.
- Enforces a 16 MiB Lua allocator budget, 4 MiB source limit and 2-million-instruction hook budget.

## Build and test

```sh
cmake -S user/app/runtime/lua -B build-lua -DCMAKE_BUILD_TYPE=Release
cmake --build build-lua --parallel 2
ctest --test-dir build-lua --output-on-failure
```

## Boundaries

This is host-only proof-of-concept code. It is not yet integrated with Gravity Kernel, the OpenXzuff process ABI, app permissions, filesystem services, IPC, networking or GUI. The instruction hook is a cooperative VM instruction-count guard, not an OS-level real-time deadline or security boundary. No binary `.luap` package container is defined yet; the current `.luap` extension denotes Lua source text until a separate package format is specified.

Upstream Lua is MIT-licensed. The build downloads the official Lua 5.5.1 source archive and verifies its published SHA-256 hash:
- https://www.lua.org/download.html
- https://www.lua.org/manual/5.5/
