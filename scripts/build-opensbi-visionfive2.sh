#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="v1.9"
SOURCE_DIR="${ROOT}/build/third_party/opensbi-visionfive2"
OUTPUT_DIR="${ROOT}/build/firmware/visionfive2"
JOBS="${JOBS:-2}"

for tool in git make riscv64-linux-gnu-gcc dtc; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "ERROR: required build tool '$tool' is missing." >&2
    exit 2
  fi
done

mkdir -p "$(dirname "$SOURCE_DIR")" "$OUTPUT_DIR"
if [[ ! -d "$SOURCE_DIR/.git" ]]; then
  git clone --depth 1 --branch "$VERSION" https://github.com/riscv-software-src/opensbi.git "$SOURCE_DIR"
else
  git -C "$SOURCE_DIR" fetch --depth 1 origin "refs/tags/$VERSION:refs/tags/$VERSION"
  git -C "$SOURCE_DIR" checkout --detach "$VERSION"
fi

# VisionFive 2 (StarFive JH7110) uses the generic OpenSBI platform with
# firmware text placed at 0x40000000. This is an OpenSBI artifact only;
# it does not include U-Boot, a board DTB, or the Gravity Kernel.
make -C "$SOURCE_DIR" clean O="$SOURCE_DIR/build"
make -C "$SOURCE_DIR" -j"$JOBS" \
  CROSS_COMPILE=riscv64-linux-gnu- \
  PLATFORM=generic \
  FW_TEXT_START=0x40000000 \
  FW_OPTIONS=0 \
  O="$SOURCE_DIR/build"

install -m 0644 "$SOURCE_DIR/build/platform/generic/firmware/fw_dynamic.bin" "$OUTPUT_DIR/fw_dynamic.bin"
install -m 0644 "$SOURCE_DIR/build/platform/generic/firmware/fw_dynamic.elf" "$OUTPUT_DIR/fw_dynamic.elf"
cp "$SOURCE_DIR/COPYING.BSD" "$OUTPUT_DIR/COPYING.BSD"
git -C "$SOURCE_DIR" rev-parse HEAD > "$OUTPUT_DIR/opensbi-source-commit.txt"
cat > "$OUTPUT_DIR/BUILD-INFO.txt" <<'INFO'
OpenSBI v1.9 generic fw_dynamic build for the StarFive VisionFive 2 / JH7110 boot address.
Build setting: FW_TEXT_START=0x40000000, FW_OPTIONS=0.
Not a complete boot image: no SPL, U-Boot, board DTB, or Gravity Kernel is included.
Build success does not prove boot or hardware support; test with the intended board boot chain.
INFO
file "$OUTPUT_DIR/fw_dynamic.bin" "$OUTPUT_DIR/fw_dynamic.elf"
