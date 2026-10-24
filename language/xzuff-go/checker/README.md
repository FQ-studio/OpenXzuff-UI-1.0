# Xzuff Go Coder Checker

`xzuff-check` is a standalone static checker for the currently implemented Xzuff Go source subset. It is intentionally honest about the current grammar: it does not pretend to validate language features that the compiler does not yet support.

## Checks performed

- Required `function main() returns i32 { return INTEGER; }` structure.
- Missing, duplicated, reordered, or trailing tokens.
- Signed i32 literal range.
- Invalid characters and malformed integer literals.
- Source file read errors.
- Optional machine-readable JSON output for editors and CI.

## Build and use

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel 2
./build/xzuff-check language/xzuff-go/tests/return_42.xgo
./build/xzuff-check --json language/xzuff-go/tests/return_42.xgo
```

Exit status is 0 when the source passes, 1 when diagnostics are reported, and 2 for command-line or I/O errors.

## CLI integration

The user-facing command is `xgo check` from a project directory containing `xgo.project` and `src/main.xgo`. The CLI calls the sibling `xzuff-check` executable. `xgo build` runs the checker before compiling; `xgo run` is reserved for test runs and does not build.

## Current limitation

This checker currently shares the compiler's narrow parser rules, not a full language grammar. It does **not** yet implement ownership, borrow-conflict, lifetime, or general type analysis. Do not describe it as Rust-equivalent until those analyses and regression tests are implemented. The next step is to expand a shared frontend/AST and add semantic checks without making the compiler and checker grammar drift apart.
