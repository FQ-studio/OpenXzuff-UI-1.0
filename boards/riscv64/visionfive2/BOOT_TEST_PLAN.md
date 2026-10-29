# VisionFive 2 — Gravity Kernel Boot Test Plan

This plan is the board-specific path to reach the same evidence standard used for the user-reported SiFive HiFive Unmatched baseline. Passing host CI is not a hardware boot result.

## Current verified repository state

- OpenSBI v1.9 generic `fw_dynamic` build is configured for the VisionFive 2 boot address setting `FW_TEXT_START=0x40000000`, `FW_OPTIONS=0`.
- The firmware workflow produces an OpenSBI-only artifact. It does not package SPL, U-Boot, a board DTB, or Gravity Kernel.
- Host CMake smoke tests and source-contract checks are automated.
- Actual VisionFive 2 boot is **not verified** yet.

## Required boot chain

1. Identify the exact VisionFive 2 board revision and its current boot media/firmware setup.
2. Use a compatible board boot chain (typically the board's SPL/U-Boot plus OpenSBI). Do not replace the board's complete boot chain with the OpenSBI-only artifact.
3. Have the bootloader load the Gravity Kernel image and the board-provided FDT into RAM.
4. Transfer control to the kernel in supervisor mode (S-mode) with the documented entry contract: hart ID in `a0`, FDT address in `a1`.
5. Capture UART output at the board's confirmed console settings. The documented UART0 address and baud rate are candidates only; confirm them against the actual FDT/board revision.

## Test gates

### Gate A — CI/build
- OpenSBI artifact files exist and are non-empty.
- Host smoke tests compile with strict warnings and pass.
- FDT parser is compiled and tested by a real Rust test target before relying on it for hardware memory initialization. A grep/source-contract check alone is insufficient.

### Gate B — first serial boot
Record the board revision, boot media, firmware versions, kernel commit, DTB source, and complete serial log. Require a visible kernel entry marker and safe FDT validation result before touching RAM allocator or device drivers.

### Gate C — kernel foundations
- Validate FDT header and every structure/string/reservation range.
- Read RAM ranges and reserved regions from the actual FDT; never hard-code RAM size.
- Initialize UART using the verified device description.
- Add timer/interrupt-controller support only after its FDT nodes and compatible strings are identified.
- Verify secondary-hart startup separately; do not claim SMP support merely because multiple harts are listed in FDT.

### Gate D — repeatability
- Perform at least 10 cold boots and 10 warm/reboot cycles with saved serial logs.
- No hangs, invalid memory ranges, or unexplained traps.
- Repeat the same checks after each kernel/boot-chain change.

## Safety rules

- Do not flash the OpenSBI-only artifact as if it were a complete bootable image.
- Do not overwrite the board's existing boot media until a recovery path and known-good backup are available.
- Use the board's matching DTB and firmware for its exact revision.
- Mark VisionFive 2 **Stable** only after real-board evidence for the required gates is attached to the release/CI record.

## Status

**In progress — CI scaffolding exists; hardware boot and kernel-level FDT tests remain unverified.**
