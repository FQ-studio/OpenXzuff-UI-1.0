# Xzuff Go — Advanced Syntax Catalog Draft 0.3

Status: cadangan reka bentuk lanjutan. Contoh tidak semestinya boleh dikompil. Ciri hanya dianggap disokong selepas grammar, type checker, runtime atau ABI berkaitan dilaksanakan dan diuji.

Xzuff Go ialah bahasa tersendiri untuk OpenXzuff UI. Ia bukan bahasa Go Google dan tidak menggunakan bentuk sintaks Rust seperti `fn`, `let`, `->`, atau `::`.

## 1. Generic types

```xgo
record Pair<Left, Right> {
    first: Left;
    second: Right;
}

function makePair<A, B>(first: A, second: B) returns Pair<A, B> {
    return Pair<A, B> { first: first, second: second };
}

function main() returns i32 {
    constant point = makePair<i32, i32>(12, 24);
    return point.first;
}
```

Generic parameters are written inside angle brackets. Type inference rules still need to be specified.

## 2. Constraints on generic types

```xgo
contract Comparable<T> {
    function compare(left: T, right: T) returns i32;
}

function maximum<T: Comparable<T>>(left: T, right: T) returns T {
    when left.compare(right) >= 0 {
        return left;
    }

    return right;
}
```

The `contract` syntax describes a required set of operations. The compiler must define static dispatch, method resolution and constraint checking.

## 3. Pattern matching

```xgo
choose response {
    case Success(value):
        Console.write(value);
    case Failure(code, message):
        Console.writeNumber(code);
        Console.write(message);
    case Pending:
        Console.write("Sedang diproses");
}
```

Patterns with guards:

```xgo
choose number {
    case value when value < 0:
        Console.write("Negatif");
    case 0:
        Console.write("Sifar");
    case value when value > 0:
        Console.write("Positif");
}
```

## 4. Result and error propagation

```xgo
function readSettings(path: string) returns result<Settings, FileError> {
    variable file = Files.open(path) or return failure(FileError.OpenFailed);
    variable text = file.readText() or return failure(FileError.ReadFailed);
    return success(Settings.parse(text));
}

function main() returns i32 {
    choose readSettings("system.cfg") {
        case success(settings):
            Settings.apply(settings);
        case failure(problem):
            Console.write(problem.message);
            return 1;
    }

    return 0;
}
```

The `result<T, E>`, `success`, `failure`, and error-propagation rules need a formal definition. This example does not imply a filesystem API already exists.

## 5. Optional values and safe access

```xgo
variable selectedWindow: optional<Window> = none;

when selectedWindow has window {
    Console.write(window.title);
}

variable title: string = selectedWindow?.title or "Tiada tetingkap";
```

The `has` pattern and `?.` safe-access operator are proposed features.

## 6. Resource scopes

```xgo
resource connection = Network.connect("app-hub.local") {
    connection.send("PING");
    variable answer = connection.receive();
    Console.write(answer);
}
```

A `resource` scope would require deterministic cleanup when leaving the block, including when an error occurs. The exact cleanup model is not yet defined.

## 7. Ownership and borrowing without Rust-style declarations

```xgo
function inspectText(borrow text: string) returns u32 {
    return text.length;
}

function updateBuffer(share buffer: Buffer) returns void {
    buffer.clear();
}

function takeBuffer(consume buffer: Buffer) returns Buffer {
    return buffer;
}
```

The proposed parameter modes are `borrow`, `share`, and `consume`. They are descriptive language keywords, not yet implemented memory-safety guarantees. The compiler must specify aliasing, lifetimes, destruction and whether each mode is enforced statically or dynamically.

## 8. Fixed-size arrays and slices

```xgo
variable pixels: array<u32, 256>;
variable visiblePixels: slice<u32> = pixels.slice(0, 64);

function fillPixels(target: slice<u32>, color: u32) returns void {
    repeat index from 0 until target.length {
        target[index] = color;
    }
}
```

The compiler must define bounds checking and slice lifetime rules.

## 9. Bit operations and hardware fields

```xgo
constant FLAG_ENABLED: u32 = 0x01;
constant FLAG_VISIBLE: u32 = 0x02;

variable flags: u32 = FLAG_ENABLED or FLAG_VISIBLE;
variable enabled: bool = (flags and FLAG_ENABLED) != 0;
variable shifted: u32 = flags << 4;
```

Bitwise operators must be distinct from logical `and` and `or` in the final grammar; this example uses them illustratively and requires a final token policy.

## 10. Enumerated data with payloads

```xgo
variant AppMessage {
    Launch(appId: u32),
    Close(appId: u32),
    Rename(appId: u32, newName: string),
    Shutdown
}

function handleMessage(message: AppMessage) returns void {
    choose message {
        case Launch(id):
            AppManager.launch(id);
        case Close(id):
            AppManager.close(id);
        case Rename(id, name):
            AppManager.rename(id, name);
        case Shutdown:
            System.requestShutdown();
    }
}
```

Payload variants need layout, ABI and exhaustiveness rules.

## 11. Interfaces and default methods

```xgo
interface Drawable {
    function draw() returns void;

    function isVisible() returns bool {
        return true;
    }
}

record DesktopIcon implements Drawable {
    label: string;

    function draw() returns void {
        Graphics.drawLabel(label);
    }
}
```

The compiler must specify interface dispatch and object layout.

## 12. Extension methods

```xgo
extend string {
    function isEmpty() returns bool {
        return self.length == 0;
    }
}

when "OpenXzuff".isEmpty() {
    Console.write("Rentetan kosong");
}
```

The meaning and resolution of `self` in extension blocks must be specified. Extension conflicts need deterministic resolution rules.

## 13. Operator declarations

```xgo
operator add(left: Vector2, right: Vector2) returns Vector2 {
    return Vector2 {
        x: left.x + right.x,
        y: left.y + right.y
    };
}
```

Operator overloading should be limited to a documented operator set to preserve readable code.

## 14. Lambdas and callbacks

```xgo
variable onClick = action(button: Button) {
    Console.write(button.label);
};

Button.create("Open App Hub", onClick);
```

A lambda uses `action(parameters) { ... }`. Capture rules, inferred types and closure memory layout remain to be specified.

## 15. Tasks and asynchronous work

```xgo
task function loadIcons() returns result<IconSet, LoadError> {
    return await IconStore.loadAll();
}

function main() returns i32 {
    variable icons = await loadIcons();
    Desktop.setIcons(icons);
    return 0;
}
```

The language needs a defined executor or scheduler, task state model, cancellation behavior and ABI. `await` cannot work without runtime or compiler support.

## 16. Channels and message passing

```xgo
variable messages: channel<AppMessage> = Channel.create(32);

task function desktopService() returns void {
    repeat while true {
        variable message = await messages.receive();
        handleMessage(message);
    }
}
```

Channel capacity, blocking behavior, memory ordering and cross-process transport need to be defined separately.

## 17. Actors and isolated services

```xgo
actor WindowManager {
    state windows: array<Window>;

    message OpenWindow(title: string);
    message CloseWindow(id: u32);

    on OpenWindow(title) {
        windows.add(Window.create(title));
    }

    on CloseWindow(id) {
        windows.removeById(id);
    }
}
```

This proposes an actor declaration with private state and message handlers. It requires a runtime and inter-service IPC implementation.

## 18. Declarative GUI

```xgo
view SettingsPage {
    column spacing: 12 {
        text "Tetapan OpenXzuff" size: 24;
        toggle "Mod gelap" bind: settings.darkMode;
        slider "Skala UI" bind: settings.scale range: 75..150;
        button "Simpan" onClick: saveSettings();
    }
}
```

The GUI grammar, layout engine, event system, accessibility and framebuffer integration must be implemented before this can produce an actual window.

## 19. Reactive state

```xgo
state counter: i32 = 0;
derive doubled: i32 = counter * 2;

view CounterPanel {
    text "Nilai: ${counter}";
    text "Dua kali: ${doubled}";
    button "Tambah" onClick: counter = counter + 1;
}
```

The compiler or UI runtime must define dependency tracking, update batching and thread affinity.

## 20. Annotations

```xgo
@entry
@target("riscv64")
function main() returns i32 {
    return 0;
}

@deprecated("Gunakan openWindow")
function createOldWindow() returns void {
    WindowManager.openWindow("Lama");
}
```

Annotations can describe entry points, target constraints and diagnostics. The compiler must define which annotations affect code generation and which are informational.

## 21. Compile-time constants and evaluation

```xgo
compile constant PAGE_SIZE: u32 = 4096;
compile constant ICON_COUNT: u32 = 16 * 16;

when compile target("riscv64") {
    import Hardware.RiscV64;
} otherwise {
    import Hardware.Host;
}
```

Compile-time evaluation must be bounded and deterministic. Conditional imports need target-specific build rules.

## 22. Compile-time templates

```xgo
template makePairType(Name, TypeA, TypeB) {
    record Name {
        first: TypeA;
        second: TypeB;
    }
}

makePairType(Coordinate, i32, i32);
```

This is an illustrative macro/template syntax, not an implemented macro system. Expansion order, hygiene and diagnostics must be designed before adoption.

## 23. Inline assembly boundary

```xgo
@target("riscv64")
function cpuPause() returns void {
    assembly riscv64 {
        pause;
    }
}
```

Inline assembly must be target-gated and checked against the selected assembler syntax. The compiler must define clobbers, register constraints and privilege restrictions. User applications must not gain unrestricted kernel privileges.

## 24. Interrupt and driver declarations

```xgo
@kernel
@interrupt(11)
function timerInterrupt(context: InterruptContext) returns void {
    Scheduler.onTimerTick(context);
}

driver DisplayDriver {
    probe: Display.detect;
    start: Display.initialize;
    stop: Display.shutdown;
}
```

These declarations are proposals only. Interrupt entry, register saving, privilege transitions and driver lifecycle require a real kernel ABI and hardware implementation.

## 25. Atomic operations

```xgo
variable ready: atomic<bool> = false;

function publishReady() returns void {
    ready.store(true, order: release);
}

function isReady() returns bool {
    return ready.load(order: acquire);
}
```

Atomic widths, alignment and memory-order semantics must match the RV64 target and compiler backend.

## 26. FFI and C ABI

```xgo
external library "display" {
    function display_present(buffer: pointer<u32>, width: u32, height: u32) returns i32;
}
```

FFI declarations need ABI compatibility, symbol naming, pointer safety and linking rules. This does not imply a library already exists.

## 27. Module visibility

```xgo
module Xzuff.Graphics;

public record Color {
    red: u8;
    green: u8;
    blue: u8;
    alpha: u8;
}

internal function normalizeChannel(value: i32) returns u8 {
    when value < 0 {
        return 0;
    }
    when value > 255 {
        return 255;
    }
    return value as u8;
}
```

The final language must define visibility defaults, module boundaries and cast behavior.

## 28. Compile-time and runtime assertions

```xgo
assert PAGE_SIZE == 4096;
check buffer.length > 0, "Buffer tidak boleh kosong";
```

The language must distinguish compile-time assertions from runtime checks and define behavior in release builds.

## 29. Tests and expected failures

```xgo
test "addition" {
    verify add(2, 3) equals 5;
}

test "bad index is rejected" expects BoundsError {
    variable values: array<i32, 2> = [4, 8];
    Console.writeNumber(values[9]);
}
```

A test runner, assertion library and panic/error model are prerequisites.

## 30. Native build configuration

```xgo
build target "riscv64-openxzuff";
build output "desktop.xexe";
build optimization speed;
build debug symbols on;
build link Xzuff.Core;
```

This is a proposed build configuration syntax. It needs a grammar or separate manifest format and a functioning linker.

## 31. Memory-mapped device declarations

```xgo
device register DisplayControl at 0x10000000 {
    field enabled: u1 at bit 0;
    field mode: u3 at bits 1..3;
}
```

This must eventually validate address width, alignment, access permissions, volatile semantics and target board memory maps. It is not safe to use arbitrary addresses on real hardware.

## 32. Native entry and return conventions

```xgo
@entry
@callingConvention("openxzuff-rv64")
function main() returns i32 {
    return 0;
}
```

The actual calling convention, stack alignment, register use, executable header, relocations and startup code must be defined alongside the compiler and OS loader.

## 33. Proposed keyword inventory

Declarations: `module`, `import`, `as`, `public`, `internal`, `constant`, `variable`, `function`, `returns`, `record`, `enum`, `variant`, `interface`, `contract`, `extend`, `namespace`, `actor`, `driver`, `view`, `state`.

Control flow: `when`, `otherwise`, `choose`, `case`, `repeat`, `from`, `until`, `times`, `while`, `leave`, `skip`, `return`.

Advanced: `task`, `await`, `resource`, `borrow`, `share`, `consume`, `template`, `compile`, `assembly`, `external`, `library`, `memory`, `region`, `device`, `register`, `field`, `atomic`, `order`, `assert`, `check`, `test`, `verify`, `expects`, `build`, `target`, `output`, `optimization`, `debug`, `symbols`, `link`, `on`, `success`, `failure`.

This inventory is provisional. Before the parser is implemented, keywords and operator precedence must be frozen in a formal grammar.

## Implementation warning

This catalogue describes possible language features, not a working compiler. Initial compiler milestones should implement and test a small subset first: lexer, parser, type checking for integer functions, RV64 code generation, then object/executable format and a test harness. Advanced syntax should not be advertised as supported until each feature passes compiler tests.
