#!/usr/bin/env bash
set -euo pipefail
clang="$1"; converter="$2"; tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/probe.cpp" <<'CPP'
extern "C" [[noreturn]] void _start() {
    volatile unsigned long answer = 40UL + 2UL;
    (void)answer;
    for (;;) {}
}
CPP
"$clang" --target=riscv64-unknown-elf -march=rv64gc -mabi=lp64d -std=c++20 \
  -ffreestanding -fno-exceptions -fno-rtti -fno-stack-protector -nostdlib \
  -fuse-ld=lld -Wl,-static -Wl,-Ttext=0x80200000 -Wl,-e,_start \
  "$tmp/probe.cpp" -o "$tmp/probe.elf"
"$converter" "$tmp/probe.elf" "$tmp/probe.gnb"
"$converter" --verify "$tmp/probe.gnb"
