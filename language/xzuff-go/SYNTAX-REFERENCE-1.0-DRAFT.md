# Xzuff Go — Syntax Reference 1.0 Draft

**Status:** consolidated language-design proposal, not a completed compiler. This reference expands the syntax catalog and defines a path toward a frozen language grammar. Xzuff Go is a new language for OpenXzuff UI, not Google's Go. Its syntax deliberately avoids Rust forms such as fn, let, ->, and ::.

## 1. Source files and formatting

- Source extension: .xgo
- Encoding: UTF-8.
- Identifiers are case-sensitive.
- Ordinary statements end with a semicolon.
- Blocks use braces.
- Compiler diagnostics should report file, line and column.
- The final grammar must define line endings, Unicode identifiers and nested-comment behavior.

## 2. Comments

    // Single-line comment

    /*
       Multi-line comment
    */

    /// Documentation comment for a declaration.

Nested block comments are not guaranteed until the grammar specifies them.

## 3. Identifiers and modules

    module Xzuff.Desktop.Settings;

    import System.Console;
    import Xzuff.Graphics as Graphics;

    public constant DEFAULT_WIDTH: u32 = 1280;
    internal function normalize(value: i32) returns i32 {
        return value;
    }

Proposed visibility: public, internal and private. Protected visibility is optional and should be removed if inheritance is not supported.

## 4. Literal forms

    constant decimalValue: i32 = 123;
    constant hexValue: u32 = 0x7B;
    constant binaryValue: u32 = 0b01111011;
    constant groupedValue: u64 = 1_000_000;
    constant scale: f32 = 0.75;
    constant scientific: f64 = 1.25e3;
    constant enabled: bool = true;
    constant letter: char = 'X';
    constant message: string = "Hello\nXzuff";
    constant interpolated: string = "Version ${VERSION}";

The final specification must define escape sequences, Unicode handling, raw strings, multiline strings, literal suffixes and integer overflow.

## 5. Primitive and compound types

    variable active: bool = true;
    variable count: i32 = 12;
    variable total: u64 = 9000;
    variable ratio: f64 = 1.25;
    variable items: array<i32> = [1, 2, 3];
    variable maybeWindow: optional<Window> = none;
    variable result: result<Settings, ConfigError>;
    variable bytes: slice<u8>;

Proposed primitives: void, never, bool, char, string, i8, i16, i32, i64, u8, u16, u32, u64, f32 and f64.

Proposed compound types: pointer<T>, optional<T>, result<T, E>, array<T>, array<T, N>, slice<T>, tuple<A, B>, map<K, V>, set<T>, channel<T> and atomic<T>. Their memory layout, alignment and ABI must be specified per target.

## 6. Constants, variables and inference

    constant OS_NAME: string = "OpenXzuff";
    variable activeWindows: u32 = 0;
    variable inferredCount = 42;

A constant cannot be reassigned. A variable can be reassigned. Type inference is allowed only when the initializer gives an unambiguous type. Definite-initialization rules are required.

## 7. Operators and precedence

Arithmetic: +, -, *, /, %.
Comparison: ==, !=, <, <=, >, >=.
Logical: not, and, or.
Assignment: =, +=, -=, *=, /=, %=.
Access: ., [], function call parentheses, optional access ?. and coalescing ??.

Proposed precedence from high to low:
1. Function call, indexing and field access.
2. Unary not and unary minus.
3. Multiplication, division and remainder.
4. Addition and subtraction.
5. Comparisons.
6. Logical and.
7. Logical or.
8. Null-coalescing.
9. Assignment.

Bitwise operations must use a clearly distinct operator set or named intrinsics. Final associativity and precedence are not frozen yet.

## 8. Casts and conversions

    variable wide: i64 = 500;
    variable narrow: i32 = wide as i32;

The compiler must distinguish safe conversions from conversions that can lose information. String conversion should use explicit conversion functions unless a defined rule applies.

## 9. Functions and defaults

    function add(left: i32, right: i32) returns i32 {
        return left + right;
    }

    function greet(name: string = "User") returns string {
        return "Hello, ${name}";
    }

Named arguments, overloads, variadic parameters and function pointers remain optional design decisions.

## 10. Generic types and constraints

    record Pair<A, B> {
        first: A;
        second: B;
    }

    function firstOf<A, B>(pair: Pair<A, B>) returns A {
        return pair.first;
    }

    contract Printable<T> {
        function print(value: T) returns void;
    }

The compiler must choose monomorphization or runtime generics and specify constraint checking.

## 11. Records, variants and pattern matching

    record Point {
        x: i32;
        y: i32;
    }

    variant NetworkState {
        Offline,
        Connecting(address: string),
        Connected(sessionId: u64),
        Failed(code: i32, message: string)
    }

    choose state {
        case Offline:
            reconnect();
        case Connecting(address):
            Console.write(address);
        case Connected(id):
            Console.writeNumber(id);
        case Failed(code, message):
            Console.write(message);
    }

Pattern exhaustiveness, binding scope, variant layout and default field values require formal rules.

## 12. Conditions and loops

    when score >= 100 {
        Console.write("Excellent");
    } otherwise when score >= 50 {
        Console.write("Good");
    } otherwise {
        Console.write("Try again");
    }

    repeat while running {
        update();
    }

    repeat index from 0 until 10 {
        draw(index);
    }

    repeat 3 times {
        refresh();
    }

leave exits the nearest loop; skip continues to its next iteration. Labels for nested loops are not yet specified.

## 13. Optional values and errors

    function findWindow(id: u32) returns optional<Window> {
        when WindowManager.exists(id) {
            return some(WindowManager.get(id));
        }
        return none;
    }

    choose loadConfig("system.cfg") {
        case success(config):
            Settings.apply(config);
        case failure(problem):
            Console.write(problem.message);
    }

The language must define the representation of optional and result types and whether exceptions are supported.

## 14. Interfaces, contracts and extensions

    interface Drawable {
        function draw() returns void;
    }

    record Icon implements Drawable {
        label: string;

        function draw() returns void {
            Graphics.drawLabel(label);
        }
    }

The language must define interface dispatch, method resolution, visibility and whether class inheritance exists.

## 15. Arrays, maps and slices

    variable values: array<i32> = [10, 20, 30];
    variable first: i32 = values[0];

    variable lookup: map<string, u32>;
    lookup["desktop"] = 1;

Bounds checks, collection allocation, mutability and out-of-memory behavior must be specified.

## 16. Lambdas and callbacks

    variable onPressed = action(button: Button) {
        Console.write(button.label);
    };

    Button.create("Open", onPressed);

Capture semantics, closure layout, escape analysis and lifetimes remain to be designed.

## 17. Resource lifetime

    resource file = Files.open("settings.cfg") {
        variable contents = file.readText();
        Console.write(contents);
    }

A resource scope should define cleanup on normal exit, return and failure. This syntax does not itself implement resource management.

## 18. Async, tasks, channels and actors

    task function refreshIcons() returns void {
        await IconStore.reload();
    }

    actor NotificationService {
        message Show(text: string);

        on Show(text) {
            Desktop.showNotification(text);
        }
    }

A scheduler, cancellation rules, channel behavior, memory ordering and IPC transport are required before these constructs can work.

## 19. Declarative GUI and reactive state

    state darkMode: bool = true;

    view AppearanceSettings {
        column spacing: 12 {
            text "Appearance" size: 24;
            toggle "Dark mode" bind: darkMode;
            button "Apply" onClick: Settings.apply(darkMode);
        }
    }

A widget tree, layout engine, compositor, input routing, accessibility model and UI-thread policy must be implemented.

## 20. System programming

    @kernel
    function handleTimer(context: InterruptContext) returns void {
        Scheduler.onTimerTick(context);
    }

    driver DisplayDriver {
        probe: Display.detect;
        start: Display.initialize;
        stop: Display.shutdown;
    }

Interrupt ABI, register saving, privilege transitions, memory maps and driver lifecycle need a real Gravity Kernel contract.

## 21. Pointers, memory and atomics

    variable pointerValue: pointer<u32>;
    variable ready: atomic<bool> = false;

    ready.store(true, order: release);
    variable state = ready.load(order: acquire);

Pointer safety, alignment, volatile access, aliasing and atomic memory ordering must be formalized. Syntax alone does not make memory access safe.

## 22. FFI and assembly

    external library "display" {
        function display_present(buffer: pointer<u32>, width: u32, height: u32) returns i32;
    }

    @target("riscv64")
    function cpuPause() returns void {
        assembly riscv64 {
            pause;
        }
    }

FFI needs a stable calling convention, symbol resolution and linker support. Inline assembly requires register constraints and clobber rules.

## 23. Annotations and compile-time configuration

    @entry
    @target("riscv64")
    function main() returns i32 {
        return 0;
    }

    compile constant PAGE_SIZE: u32 = 4096;

The compiler must define standard annotations, target-specific compilation and bounded compile-time evaluation.

## 24. Templates and metaprogramming

    template createPair(Name, FirstType, SecondType) {
        record Name {
            first: FirstType;
            second: SecondType;
        }
    }

Expansion order, hygiene, recursion limits and diagnostics must be specified before templates are considered supported.

## 25. Build configuration

    build target "riscv64-openxzuff";
    build output "desktop.xexe";
    build optimization speed;
    build debug symbols on;
    build link Xzuff.Core;

It remains to decide whether build directives belong in source code or in a separate manifest file.

## 26. Tests and assertions

    test "integer addition" {
        verify add(2, 3) equals 5;
    }

    assert PAGE_SIZE == 4096;
    check buffer.length > 0, "Buffer is empty";

A test runner, expected-failure semantics, panic model and source-level diagnostics are required.

## 27. Package metadata and entry point

    package {
        name: "com.openxzuff.settings";
        version: "1.0.0";
        entry: "main";
        permissions: ["display", "settings.read"];
    }

    @entry
    function main() returns i32 {
        return 0;
    }

Application package metadata may ultimately be stored separately from source code.

## 28. Native executable and ABI

The proposed executable suffix is .xexe. A real implementation must define the file header, RV64 architecture identifier, ABI, entry offset, sections, alignment, relocations, startup code, standard-library linking and Gravity Kernel loader contract.

Generating RV64 instructions alone is not enough to make a complete installable OS application.

## 29. Formal grammar checklist

Before claiming syntax 1.0 is frozen, specify a formal grammar for:
1. Tokens and comments.
2. Literals and escapes.
3. Modules and imports.
4. Declarations and visibility.
5. Type expressions and generics.
6. Expression precedence and associativity.
7. Statements and control flow.
8. Pattern matching.
9. Functions, records and interfaces.
10. Async and task declarations.
11. GUI declarations.
12. Memory, FFI and assembly.
13. Annotations and compile-time expressions.
14. Error recovery and diagnostics.

Every grammar production needs valid and invalid parser tests.

## 30. Implementation status

- Draft 0.1 specification: written.
- Draft 0.2 syntax catalog: written.
- Draft 0.3 advanced catalog: written.
- Consolidated syntax reference: proposed in this document.
- Frozen formal grammar: not yet implemented.
- Lexer and parser: not implemented.
- Type checker and IR: not implemented.
- RV64 machine-code generator: not implemented.
- Object/executable format, linker and loader: not implemented.
- Compiler conformance tests: not implemented.

## Design rule

A long list of keywords is not proof of 100% language completeness. Completeness requires a precise grammar, defined semantics, diagnostics, tests and a clear distinction between supported features and proposals.
