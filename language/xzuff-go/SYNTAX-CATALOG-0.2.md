# Xzuff Go — Katalog Sintaks Draft 0.2

> Status: cadangan reka bentuk bahasa. Contoh dalam dokumen ini belum semuanya disokong oleh compiler. Jangan anggap ia boleh dikompil atau dijalankan sehingga lexer, parser, type checker, runtime dan code generator melaksanakannya.

Xzuff Go ialah bahasa baharu untuk aplikasi dan perisian sistem OpenXzuff UI. Namanya tidak bermaksud ia bahasa Go daripada Google. Sintaksnya menggunakan kata kunci deskriptif seperti `function`, `constant`, `variable`, `returns`, `when`, dan `choose`. Reka bentuk ini sengaja mengelakkan bentuk Rust seperti `fn`, `let`, `->`, dan `::`.

## 1. Fail, modul dan import

Fail sumber menggunakan sambungan `.xgo`.

```xgo
module Demo.Greeting;

import System.Console;
import System.Time;

function main() returns i32 {
    Console.write("Hai daripada Xzuff Go!");
    return 0;
}
```

Alias import:

```xgo
import Graphics.Drawing as Draw;
import Storage.Files as Files;
```

## 2. Pemalar dan pemboleh ubah

```xgo
constant productName: string = "OpenXzuff UI";
constant releaseNumber: u32 = 1;

variable score: i32 = 0;
variable isReady: bool = false;

score = score + 10;
isReady = true;
```

Nilai yang jenisnya boleh disimpulkan:

```xgo
variable retries = 3;
constant welcome = "Selamat datang";
```

## 3. Jenis data asas

```xgo
variable enabled: bool = true;
variable smallNumber: i8 = -8;
variable count: i32 = 120;
variable largeCount: u64 = 9000;
variable ratio: f32 = 0.5;
variable precision: f64 = 3.14159;
variable letterCode: char = 'A';
variable message: string = "Hello";
```

Jenis awal yang dicadangkan: `void`, `bool`, `char`, `string`, `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, dan `f64`.

## 4. Operator

```xgo
variable total: i32 = 8 + 4;
variable difference: i32 = 8 - 4;
variable product: i32 = 8 * 4;
variable quotient: i32 = 8 / 4;
variable remainder: i32 = 9 % 4;

variable equal: bool = total == 12;
variable different: bool = total != 9;
variable validRange: bool = total >= 10 and total <= 20;
variable eitherFlag: bool = enabled or isReady;
variable inverted: bool = not enabled;
```

## 5. Pilihan bersyarat

```xgo
when score >= 100 {
    Console.write("Hebat!");
} otherwise when score >= 50 {
    Console.write("Bagus!");
} otherwise {
    Console.write("Terus cuba.");
}
```

## 6. Pilihan nilai

```xgo
variable accessLevel: i32 = 2;

choose accessLevel {
    case 0:
        Console.write("Tetamu");
    case 1:
        Console.write("Pengguna");
    case 2:
        Console.write("Pentadbir");
    otherwise:
        Console.write("Tidak diketahui");
}
```

## 7. Gelung

Gelung selagi syarat benar:

```xgo
variable attempt: i32 = 0;

repeat while attempt < 3 {
    Console.writeNumber(attempt);
    attempt = attempt + 1;
}
```

Gelung julat:

```xgo
repeat index from 0 until 5 {
    Console.writeNumber(index);
}
```

Ulang bilangan kali:

```xgo
repeat 4 times {
    Console.write("Xzuff");
}
```

Keluar atau langkau satu pusingan:

```xgo
repeat while true {
    when shouldStop() {
        leave;
    }

    when shouldSkip() {
        skip;
    }

    doWork();
}
```

## 8. Fungsi

```xgo
function add(left: i32, right: i32) returns i32 {
    return left + right;
}

function showWelcome(name: string) returns void {
    Console.write("Selamat datang");
    Console.write(name);
}

function main() returns i32 {
    constant answer: i32 = add(20, 22);
    Console.writeNumber(answer);
    return 0;
}
```

Fungsi tanpa nilai pulangan boleh ditulis dengan `returns void`. Parameter dan nilai pulangan mempunyai jenis yang dinyatakan.

## 9. Rekod data

```xgo
record Point {
    x: i32;
    y: i32;
}

record WindowInfo {
    title: string;
    width: u32;
    height: u32;
    visible: bool;
}

function main() returns i32 {
    variable origin: Point = Point { x: 0, y: 0 };
    Console.writeNumber(origin.x);
    return 0;
}
```

## 10. Enum

```xgo
enum BootState {
    Starting,
    LoadingDrivers,
    StartingDesktop,
    Ready,
    Failed
}

variable boot: BootState = BootState.Starting;
```

## 11. Array dan senarai

```xgo
variable scores: array<i32> = [10, 20, 30, 40];
variable firstScore: i32 = scores[0];

scores[1] = 25;
Console.writeNumber(firstScore);
```

## 12. Rentetan dan interpolasi

```xgo
variable username: string = "Faqih";
variable greeting: string = "Hai, ${username}!";
Console.write(greeting);
```

Interpolasi `${...}` ialah cadangan sintaks; compiler perlu menentukan peraturan penukaran jenis dan escape.

## 13. Jenis pilihan dan nilai kosong

```xgo
function findUser(id: u32) returns optional<string> {
    when id == 1 {
        return some("Pengguna");
    }

    return none;
}
```

`optional<T>`, `some(...)`, dan `none` dicadangkan untuk menyatakan nilai yang mungkin tiada tanpa bergantung pada penunjuk null.

## 14. Pengendalian ralat

```xgo
result<Config, ConfigError> loadConfig(string path);

function main() returns i32 {
    variable loaded = loadConfig("system.cfg");

    on success config = loaded {
        Console.write(config.name);
    } on failure problem = loaded {
        Console.write(problem.message);
        return 1;
    }

    return 0;
}
```

Blok `on success` dan `on failure` ialah cadangan sintaks. Perincian jenis `result<T, E>` dan aliran kawalan mesti ditetapkan dalam spesifikasi penuh.

## 15. Antara muka dan pelaksanaan

```xgo
interface Drawable {
    function draw() returns void;
}

record Panel implements Drawable {
    width: u32;
    height: u32;

    function draw() returns void {
        Graphics.fillRectangle(0, 0, width, height);
    }
}
```

## 16. Ruang nama

```xgo
namespace Desktop.Settings {
    constant defaultScale: u32 = 100;

    function reset() returns void {
        Console.write("Tetapan dipulihkan");
    }
}
```

## 17. Penggunaan sistem

Nama API berikut ialah contoh API yang dicadangkan, bukan bukti bahawa pemacu atau servis kernel telah tersedia.

```xgo
import Xzuff.Display;
import Xzuff.Input;

function main() returns i32 {
    Display.clear(0x101820);
    Display.writeText(24, 24, "OpenXzuff Desktop");

    when Input.keyPressed("ESCAPE") {
        return 0;
    }

    return 0;
}
```

## 18. Tugas dan kerja serentak

```xgo
task function refreshDesktop() returns void {
    Desktop.refreshIcons();
}

function main() returns i32 {
    Scheduler.start(refreshDesktop);
    return 0;
}
```

`task` dan `Scheduler.start` hanya boleh digunakan apabila runtime dan penjadual OS menyokongnya.

## 19. Pengurusan sumber

```xgo
using file = Files.open("settings.cfg") {
    variable contents: string = file.readText();
    Console.write(contents);
}
```

Blok `using` dicadangkan untuk memastikan sumber ditutup secara automatik apabila blok tamat. Semantik tepatnya belum ditetapkan.

## 20. Ciri aras rendah untuk sistem

```xgo
memory region Framebuffer size 4096 aligned 64;

function readStatus() returns u64 {
    return Hardware.readRegister(0x1000);
}
```

Sintaks aras rendah seperti pemetaan memori, daftar perkakasan dan operasi atomik mesti dibatasi oleh target, ABI dan model keselamatan. Contoh ini belum boleh digunakan untuk mengakses perkakasan sebenar.

## 21. Ujian

```xgo
test "add returns the sum" {
    verify add(2, 3) equals 5;
}

test "boot starts in starting state" {
    verify BootState.Starting is valid;
}
```

`test`, `verify`, dan `equals` ialah cadangan untuk sistem ujian masa hadapan.

## 22. Titik masuk program

```xgo
function main() returns i32 {
    Console.write("Xzuff Go berjalan");
    return 0;
}
```

Draft awal menetapkan `main` sebagai titik masuk konvensional. Format executable, calling convention, pengendalian ralat, layout memori dan ABI RV64 perlu ditetapkan sebelum binari boleh dijalankan pada Gravity Kernel.

## Status pelaksanaan

| Komponen | Status |
|---|---|
| Dokumen sintaks Draft 0.1 | Ditulis |
| Katalog sintaks Draft 0.2 | Dokumen reka bentuk ini |
| Lexer | Belum dilaksanakan |
| Parser | Belum dilaksanakan |
| Pemeriksaan jenis | Belum dilaksanakan |
| IR dan pengoptimum | Belum dilaksanakan |
| Penjana kod mesin RV64 | Belum dilaksanakan |
| Linker dan format executable | Belum dilaksanakan |
| Runtime dan standard library | Belum lengkap |

**Penting:** Katalog ini memberi bahasa identiti sintaks yang lebih luas, tetapi ciri-cirinya masih cadangan. Implementasi compiler perlu bermula dengan subset kecil yang boleh diuji dan berkembang secara berperingkat.
