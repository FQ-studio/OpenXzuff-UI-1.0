#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="v1.9"
SOURCE_DIR="${ROOT}/build/third_party/opensbi"
OUTPUT_DIR="${ROOT}/build/firmware/opensbi"
JOBS="${JOBS:-2}"

if ! command -v git >/dev/null 2>&1; then
  echo "ERROR: git is required to fetch OpenSBI." >&2
  exit 2
fi
if ! command -v make >/dev/null 2>&1; then
  echo "ERROR: make is required." >&2
  exit 2
fi
if ! command -v riscv64-linux-gnu-gcc >/dev/null 2>&1; then
  echo "ERROR: install a RISC-V 64-bit Linux cross toolchain (riscv64-linux-gnu-gcc)." >&2
  exit 2
fi
if ! command -v dtc >/dev/null 2>&1; then
  echo "ERROR: device-tree-compiler (dtc) is required by OpenSBI." >&2
  exit 2
fi

mkdir -p "$(dirname "$SOURCE_DIR")" "$OUTPUT_DIR"
if [[ ! -d "$SOURCE_DIR/.git" ]]; then
  git clone --depth 1 --branch "$VERSION" https://github.com/riscv-software-src/opensbi.git "$SOURCE_DIR"
else
  git -C "$SOURCE_DIR" fetch --depth 1 origin "refs/tags/$VERSION:refs/tags/$VERSION"
  git -C "$SOURCE_DIR" checkout --detach "$VERSION"
fi

make -C "$SOURCE_DIR" -j"$JOBS" CROSS_COMPILE=riscv64-linux-gnu- PLATFORM=generic O="$SOURCE_DIR/build"
install -m 0644 "$SOURCE_DIR/build/platform/generic/firmware/fw_dynamic.bin" "$OUTPUT_DIR/fw_dynamic.bin"
install -m 0644 "$SOURCE_DIR/build/platform/generic/firmware/fw_dynamic.elf" "$OUTPUT_DIR/fw_dynamic.elf"
cp "$SOURCE_DIR/COPYING.BSD" "$OUTPUT_DIR/COPYING.BSD"
git -C "$SOURCE_DIR" rev-parse HEAD > "$OUTPUT_DIR/opensbi-source-commit.txt"
printf 'Built OpenSBI %s generic fw_dynamic firmware. This artifact is not certified for every OpenXzuff target board.\n' "$VERSION" | tee "$OUTPUT_DIR/BUILD-INFO.txt"
