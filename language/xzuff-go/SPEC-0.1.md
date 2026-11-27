# Xzuff Go Language Specification — Draft 0.1

Xzuff Go is a proposed native systems and application language for OpenXzuff UI. It is a new language; it is not Google's Go, Rust, C++, Lua, or Luau.

This document defines the initial syntax direction only. It does not claim that the complete language compiler or runtime has been implemented.

## Design rules

- Readable keywords; no Rust-style fn, let, arrow-return, or double-colon syntax.
- Braces delimit blocks; each declaration or statement ends with a semicolon.
- Static types are explicit in declarations and function results.
- The first compiler target is RV64 machine code.
- The compiler implementation may use C++; selected low-level components may use Rust, but Xzuff Go source syntax is independent of either implementation language.
- The initial executable format is .xexe; it is specified alongside the initial Gravity ABI and validator, but general process loading is not complete.

## Example program

~~~xgo
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
~~~

The example describes the proposed language syntax. System.Console and its functions are standard-library API proposals, not implemented APIs yet.

## Declarations

~~~xgo
constant maxItems: u32 = 64;
variable currentItem: u32 = 0;

function add(left: i32, right: i32) returns i32 {
    return left + right;
}
~~~

- constant declares a binding that cannot be reassigned.
- variable declares a mutable binding.
- Function parameters use name: type.
- returns type specifies the return type.
- The first draft proposes void, bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, and string.

## Conditions and loops

~~~xgo
if currentItem < maxItems {
    currentItem = currentItem + 1;
} else {
    currentItem = 0;
}

while currentItem < 10 {
    currentItem = currentItem + 1;
}
~~~

The full grammar and type checker must define boolean conditions, scope, overflow behavior, and loop control before this example can be compiler-tested.

## Modules and imports

~~~xgo
module Desktop;

import Xzuff.GUI;
import System.Console;
~~~

Use dotted module names rather than Rust-style path separators. Module resolution rules will be documented with the package and linker design.

## Implemented compiler subset

The current compiler is intentionally much narrower than this draft. It accepts one function main() returns i32 body containing one return statement whose value is an integer constant expression. The expression subset supports unary plus/minus, multiplication, division, remainder, addition, subtraction, and parentheses. The compiler folds the expression at compile time using checked signed 64-bit intermediates and requires the final value to fit signed i32.

The RV64 backend emits an exit stub using Gravity ABI v1 Exit (syscall ID 0), then writes a .xexe v1 image. It does not yet generate runtime arithmetic for variables or parameters, and the kernel validator does not yet load or execute arbitrary .xexe images.

## Native compilation pipeline

source (.xgo) -> lexer -> parser -> semantic/type checker -> intermediate representation -> optimizer -> RV64 machine-code generator -> linker -> executable

The implemented milestone currently covers lexing, parsing and constant-expression evaluation for the restricted subset, a small RV64 exit-stub generator, and .xexe image emission/validation. The general type checker, IR, optimizer, linker and process loader remain planned.

## Planned implementation milestones

1. Freeze lexical rules and a formal grammar.
2. Implement lexer, parser, diagnostics, and syntax tests.
3. Add type checking and a minimal intermediate representation.
4. Generate a small RV64 subset and test instruction encodings.
5. Define the Xzuff ABI, executable format, linker, and kernel loader contract.
6. Add system APIs and later GUI APIs for OpenXzuff.

## Status

Draft language design and a narrow, executable compiler implementation subset. This is not a complete language compiler, linker, standard library, or RV64 application runtime.
