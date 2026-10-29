#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="riscv64imac-unknown-none-elf"
KERNEL_DIR="$ROOT/kernel"
OUT_DIR="$ROOT/build/kernel/rv64-s-mode"

command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo is required" >&2; exit 2; }
command -v rustup >/dev/null 2>&1 || { echo "ERROR: rustup is required" >&2; exit 2; }

rustup target add "$TARGET"
mkdir -p "$OUT_DIR"
cargo build --manifest-path "$KERNEL_DIR/Cargo.toml" --target "$TARGET" --release
install -m 0644 "$KERNEL_DIR/target/$TARGET/release/openxzuff-kernel" "$OUT_DIR/openxzuff-kernel.elf"

if command -v llvm-objcopy >/dev/null 2>&1; then
  llvm-objcopy -O binary "$OUT_DIR/openxzuff-kernel.elf" "$OUT_DIR/openxzuff-kernel.bin"
elif command -v riscv64-unknown-elf-objcopy >/dev/null 2>&1; then
  riscv64-unknown-elf-objcopy -O binary "$OUT_DIR/openxzuff-kernel.elf" "$OUT_DIR/openxzuff-kernel.bin"
else
  echo "NOTE: objcopy unavailable; ELF produced, raw .bin omitted."
fi

file "$OUT_DIR/openxzuff-kernel.elf"
test -s "$OUT_DIR/openxzuff-kernel.elf"
echo "RV64 S-mode kernel build completed."
