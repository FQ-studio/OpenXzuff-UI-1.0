# Xzuff Go compiler — initial Gravity Kernel native-code milestone

This is a real but deliberately limited C++20 compiler implementation. It is not a complete Xzuff Go compiler yet.

## Supported source subset

```xgo
function main() returns i32 {
    return 42;
}
```

Only one `main` function returning a signed integer literal is supported. The literal must fit signed 32-bit range (-2147483648 through 2147483647). The compiler splits larger immediates into RV64 `lui` and `addi` instructions. The lexer handles identifiers, decimal integer literals, punctuation, whitespace and `//` comments. Unsupported statements and trailing tokens are rejected.

## Output: OpenXzuff `.xexe`

The compiler emits a 32-byte `.xexe` v1 header followed by RV64 machine code. It does **not** target Linux or emit ELF. Header details and syscall rules are documented in [`Kernel/ABI/GRAVITY-RV64-USER-ABI-v1.md`](../../../Kernel/ABI/GRAVITY-RV64-USER-ABI-v1.md).

The generated entry stub does three things:

1. `addi` sets small exit values directly; `lui` plus `addi` builds larger signed 32-bit exit values in `a0`.
2. `addi a7, zero, 0` selects Gravity Kernel `Exit` (syscall ID 0).
3. `ecall` requests the kernel service.

The syscall ID matches `SyscallId::Exit` in `Kernel/include/openxzuff/kernel/contracts.hpp`. The kernel now has a host-testable `.xexe` image validator that checks magic, format version, header size, architecture, ABI version, reserved flags, code length and entry bounds. This validator does not map memory or start a process. The actual user-mode loader, trap handler and syscall dispatcher still need implementation and integration tests. A `.xexe` file therefore must not yet be described as runnable on the current kernel scaffold.

## Build and test

With CMake 3.20+ and a C++20 compiler:

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel 2
ctest --test-dir build --output-on-failure
./build/xzuff-go language/xzuff-go/tests/return_42.xgo build/return_42.xexe
```

The built-in self-test compares RV64 instruction encodings for return values 0, 42, 65536 and INT32_MIN. CI checks that output is larger than the 32-byte header and begins with the `.xexe` magic. These tests validate the compiler's current narrow subset, not a physical boot or execution on Gravity Kernel.

## Not implemented yet

- General expressions, variables, function calls, multiple functions and modules.
- Full grammar, AST, semantic/type checking and complete diagnostics.
- Branches, loops, strings, records, generics, ownership and GUI syntax.
- Relocations, symbol tables, linker and process-start integration.
- Actual Gravity Kernel trap entry, syscall dispatch and process setup.
- Standard library, memory model, stack/heap setup and broad conformance tests.

Do not advertise this compiler as 100% complete until the language spec, code generation, `.xexe` loader, ABI implementation and tests are complete.
