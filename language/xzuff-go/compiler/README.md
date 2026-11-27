# Xzuff Go compiler — initial Gravity Kernel native-code milestone

This is a real but deliberately limited C++20 compiler implementation. It is not a complete Xzuff Go compiler yet.

## Supported source subset

~~~xgo
function main() returns i32 {
    return (6 + 3) * 4 - 6 / 2;
}
~~~

The compiler accepts one main function returning a signed i32 constant expression. Supported operators are unary plus and minus, multiplication, division, remainder, addition, subtraction, and parentheses. Multiplication, division, and remainder bind more tightly than addition and subtraction; operators at the same precedence are left-associative.

The compiler evaluates this restricted expression at compile time using checked signed 64-bit intermediates. Division or remainder by zero, signed 64-bit intermediate overflow, malformed expressions, and final results outside signed i32 range are rejected with source-position diagnostics. The example above evaluates to 33.

This is constant folding for the current compiler milestone. It does not yet generate dynamic arithmetic instructions for variables, function parameters, or runtime values.

## Current compilable examples

- examples/rv64_return_42.xgo — literal return value 42.
- examples/rv64_constant_expression.xgo — precedence, parentheses, division and subtraction; returns 33.

The separate examples/hello.xgo file is a forward-looking syntax sketch (modules, imports, strings, constants, and console calls); it is not accepted by the current compiler yet and must not be used as a build fixture.

## Output: OpenXzuff .xexe

The compiler emits a 32-byte .xexe v1 header followed by RV64 machine code. It does not target Linux or emit ELF. Header values and field offsets are shared with the kernel validator through sdk/gravity/sysroot/usr/include/gravity/abi.h.

The generated entry stub does three things:

1. Sets the computed signed i32 exit status in a0 using addi, or lui plus addiw for larger values.
2. Sets a7 to Gravity Kernel Exit (syscall ID 0).
3. Executes ecall.

The kernel-side .xexe validator checks the shared magic, format version, header size, architecture, ABI version, reserved flags, code length and entry bounds. It validates an image only; it does not map memory or start a process. The general user-mode loader, process startup and runtime integration are not yet implemented. A .xexe file must therefore not yet be described as runnable on the current kernel scaffold.

## Build and test

With CMake 3.20+ and a C++20 compiler:

~~~sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel 2
ctest --test-dir build --output-on-failure
./build/xzuff-go language/xzuff-go/examples/rv64_constant_expression.xgo build/constant_expression.xexe
./build/xexe-loader-test build/constant_expression.xexe 33
~~~

The compiler self-test checks RV64 instruction encodings for 0, 42, 65536, INT32_MIN and INT32_MAX; expression precedence, parentheses, unary signs, division, remainder and left associativity; and rejection of division by zero, malformed expressions, i32 result overflow and i64 intermediate overflow. The integration test validates the compiler-produced .xexe and compares its RV64 exit-stub bytes with the expected status.

These checks validate the current constant-expression subset and image format. They do not prove general language conformance, process loading, QEMU execution of compiler-produced .xexe, or physical-board boot.

## Not implemented yet

- Variables, runtime arithmetic, assignments, branches and loops.
- Multiple functions, parameters, calls, modules and imports.
- Full AST/IR, general type checking and optimization.
- Strings, records, generics, ownership and GUI syntax.
- Relocations, symbol tables, production linker and process-start integration.
- Standard library, memory model, stack/heap setup and application runtime.

Do not advertise this compiler as 100% complete until the full language spec, code generation, .xexe loader, ABI implementation and tests are complete.

## Coder Checker

A separate xzuff-check executable is built from the same lexer and parser as the compiler, so syntax validation uses the same grammar instead of a separately maintained parser.

~~~sh
./build/xzuff-check language/xzuff-go/examples/rv64_constant_expression.xgo
./build/xzuff-check --json language/xzuff-go/examples/rv64_constant_expression.xgo
~~~

The checker returns exit code 0 for accepted source, 1 for source or file-read errors, and 2 for command-line usage errors. JSON mode emits an ok flag, the folded return_value, and diagnostics. The supported grammar remains one main function with an i32 constant expression.

## User-facing xgo CLI

The terminal-facing command dispatcher is built as xgo. Build all targets with CMake, then keep xgo, xzuff-check, and xzuff-go in the same output directory.

~~~sh
./build/xgo create HelloWorld
cd HelloWorld
../build/xgo check
../build/xgo test
../build/xgo build
../build/xgo run
~~~

- xgo create creates xgo.project, src/main.xgo, and tests/smoke_test.xgo.
- xgo check runs Coder Checker against src/main.xgo.
- xgo test checks each .xgo test file under tests.
- xgo run returns an explicit not-supported diagnostic until the kernel has a general .xexe process loader and process startup path.
- xgo build runs Coder Checker first, then invokes the compiler to write build/project-name.xexe.
- xgo delete only targets a project directly under the current directory and requires typing DELETE before recursive removal.

The CLI launches sibling tools directly (POSIX fork/exec; Windows CreateProcessW with Windows command-line argument quoting) instead of constructing a shell command. This avoids shell-quoting differences and is covered by a smoke test whose project/build path contains spaces. The Windows CI job runs on a Windows x64 hosted runner; that validates the Windows process-launch path, but is not evidence of a native Windows ARM64 build or test.
