# Xzuff Luau Runtime

C++20 runtime host embedding the official upstream Luau VM/compiler. This milestone adds a versioned .luap game-package container and CLI pack/run flow.

## Build

Requirements: CMake 3.24+, C++20 compiler, Git, and internet access on the first build (CMake downloads upstream Luau).

```sh
cmake -S runtime -B runtime/build -DCMAKE_BUILD_TYPE=Release
cmake --build runtime/build --parallel 2
```

## Run a script

```sh
runtime/build/xzuff-luau runtime/examples/hello.luau
```

## Create and run a game package

```sh
runtime/build/xzuff-luau --pack runtime/examples/hello.luau hello.luap "Hello Game" 1.0.0
runtime/build/xzuff-luau hello.luap
```

On Windows, use the executable at the corresponding build output path, commonly runtime/build/Release/xzuff-luau.exe.

## .luap v1 binary format

The current .luap format is a compact, single-entry binary container (not ZIP yet):

1. 8-byte magic: XZLUAP1 followed by a newline byte
2. 4-byte little-endian unsigned manifest byte length
3. 8-byte little-endian unsigned source byte length
4. UTF-8 JSON manifest
5. UTF-8 Luau source for the required entry main.luau

The manifest includes format, numeric format version, package name, packageVersion, and entry. Manifest size is capped at 64 KiB; source is capped at 16 MiB. The loader rejects wrong magic, unsupported entry declaration, truncated content, oversize fields and trailing bytes. It is a defined v1 container format; it does not yet package multiple assets.

## Current scope

- C++20 host embedding the official Luau VM/compiler
- Run .luau / .lua source
- Pack source into .luap and run that package
- Versioned manifest and size/truncation validation
- Instruction-count budget and script error reporting
- GitHub Actions configure/build/smoke test

## Still not implemented

- Multi-file/ZIP .luap packages, signature verification and package capability permissions
- Xzuff 2D/3D renderer, input, audio, and graphics/game APIs
- App installer integration and mobile/Gravity Kernel integration

**Security:** This is an early developer runtime, not a security boundary. Do not run untrusted packages on a sensitive system. Before distributing games, verify Luau API compatibility, test the build, restrict exposed libraries deliberately, and add OS-level isolation. The current package format has no signature or checksum.


## Repository placement

This runtime and its GitHub Actions workflow live in `FQ-studio/OpenXzuff-UI-1.0`, alongside the OpenXzuff UI OS project. The App Hub repository is not the home of the OS runtime.
