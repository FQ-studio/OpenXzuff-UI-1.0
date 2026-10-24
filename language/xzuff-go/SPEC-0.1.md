# Xzuff Go Language Specification — Draft 0.1

Xzuff Go is a proposed native systems and application language for OpenXzuff UI. It is a new language; it is not Google's Go, Rust, C++, Lua, or Luau.

This document defines the initial syntax direction only. It does not claim that a compiler or RV64 code generator has been implemented.

## Design rules

- Readable keywords; no Rust-style `fn`, `let`, `->`, or `::` syntax.
- Braces delimit blocks; each declaration or statement ends with a semicolon.
- Static types are explicit in declarations and function results.
- The first compiler target is RV64 machine code.
- The compiler implementation may use C++; selected low-level components may use Rust, but Xzuff Go source syntax is independent of either implementation language.
- The initial executable format is tentatively `.xexe`; it must be specified alongside the loader and ABI before programs can run on Gravity Kernel.

## Example program

```xgo
module Hello;

import System.Console;

function main() returns i32 {
    constant version: i32 = 1;
    constant title: string = "OpenXzuff UI";

    Console.write("Hello from Xzuff Go!");
    Console.write(title);
    Console.writeNumber(version);

    return 0;
}
```

The example describes the proposed language syntax. `System.Console` and its functions are standard-library API proposals, not implemented APIs yet.

## Declarations

```xgo
constant maxItems: u32 = 64;
variable currentItem: u32 = 0;

function add(left: i32, right: i32) returns i32 {
    return left + right;
}
```

- `constant` declares a binding that cannot be reassigned.
- `variable` declares a mutable binding.
- Function parameters use `name: type`.
- `returns type` specifies the return type.
- The first draft supports `void`, `bool`, `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, and `string`.

## Conditions and loops

```xgo
if currentItem < maxItems {
    currentItem = currentItem + 1;
} else {
    currentItem = 0;
}

while currentItem < 10 {
    currentItem = currentItem + 1;
}
```

The grammar and type checker must define boolean conditions, operator precedence, scope, overflow behavior, and loop control before this example can be compiler-tested.

## Modules and imports

```xgo
module Desktop;

import Xzuff.GUI;
import System.Console;
```

Use dotted module names rather than Rust-style path separators. Module resolution rules will be documented with the package and linker design.

## Native compilation pipeline

`source (.xgo) -> lexer -> parser -> type checker -> intermediate representation -> optimizer -> RV64 machine-code generator -> linker -> executable`

The compiler must emit real RV64 instructions and test them with a disassembler or emulator. Producing assembly text alone, or merely writing an example source file, is not proof of a working native compiler.

## Planned implementation milestones

1. Freeze lexical rules and a formal grammar.
2. Implement lexer, parser, diagnostics, and syntax tests.
3. Add type checking and a minimal intermediate representation.
4. Generate a small RV64 subset and test instruction encodings.
5. Define the Xzuff ABI, executable format, linker, and kernel loader contract.
6. Add system APIs and later GUI APIs for OpenXzuff.

## Status

Draft language design only. Compiler, linker, executable loader, standard library, and RV64 execution support are not implemented by this specification.
