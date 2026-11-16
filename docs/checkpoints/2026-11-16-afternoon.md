# 2026-11-16 afternoon — Gravity Kernel / OpenXzuff Core integration

## Repository state

- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Working branch: `feature/win32-gaming-gravity-port`
- Code validation revision: `66449dcb3af4ae6e9d70011e690e102ac7df7bdf`
- PR #12 remains open; no merge was performed.
- RV64-only kernel target remains unchanged.

## Implemented this session

### FDT compatible-node discovery

- Added `FdtRegion` and borrowed-name `FdtNode` result types.
- Added `find_compatible_node(blob, wanted)`.
- Exact NUL-delimited compatible-list matching is retained.
- A candidate is available only when its own and all ancestor `status` properties are absent, `okay` or `ok`.
- The first `reg` tuple is decoded using the parent bus's `#address-cells` and `#size-cells`.
- Missing `reg` is represented as `None`; zero size-cells yields an address with unknown size.
- Unselected buses may declare up to four address/size cells without aborting the scan. A selected `reg` must fit the current u64 representation (one/two address cells and zero/one/two size cells).
- Compatible string lists are fully validated even after a matching entry is found.
- The returned address is a raw parent-bus address. No `ranges` translation or MMIO mapping is performed.

### QEMU boot integration

- Kernel boot now probes known UART compatible candidates for FU740, JH7110, TH1520 and QEMU's `ns16550a`.
- QEMU `virt` reported:
  - `FDT UART compatible-node discovery: PASS; raw bus reg parsed`
  - raw bus address `0x10000000`
  - size `0x100`
- The kernel does not initialize or access this MMIO address. SBI legacy console remains the active output path.
- RV64 workflow now requires the FDT UART discovery marker in the QEMU serial log.

## Validation evidence

### RV64 S-Mode Kernel

[Run #43816421114](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421114) — **SUCCESS** on code revision `66449dcb...`.

- Rust host tests: **67 passed, 0 failed**.
- Win32 Gravity RV64 ecall adapter cross-compile: success.
- RV64 kernel ELF build: success.
- OpenSBI + QEMU `virt` boot: success; required serial assertions passed, including the new FDT UART discovery marker and existing Sv39, trap, timer-preemption, context-switch, IPC and U-mode syscall/memory assertions.
- QEMU is stopped by the workflow's 15-second smoke timeout after required markers. This is not a long-duration stress or production-stability test.
- [Kernel ELF, binary and QEMU serial-log artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421114/artifacts/14203180179).

### VisionFive 2 readiness

[Run #43816421040](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421040) — **SUCCESS**.

- Strict-warning host build: success.
- CTest: **16/16 passed**.
- Board boot-contract checks: success.
- This is not a physical VisionFive 2 boot or LAVA result.

### Win32 Gravity bridge

[Run #43816421214](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421214) — **SUCCESS**.

- Host bridge CTest: **2/2 passed**.
- RV64 adapter cross-compile: success.

### OpenXzuff UI Core

[Run #43816421109](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421109) is **IN PROGRESS** at checkpoint creation.

Completed successfully:
- OS contract and Xzuff Go compiler/Coder Checker tests.
- 44-byte RV64 `.xexe` fixture compile and host-loader validation.
- Gravity RV64 NDK example build.
- Android ARM64 Xzuff Go / xgo cross-compile and package.
- Gravity SDK/NDK package.

The Luau runtime build step is still running. JavaScript, standard Lua and OpenXzuff3D smoke tests have not yet been reported by this run. Do not claim the Core workflow complete until it finishes.

## Failures diagnosed and corrected

1. The new FDT test fixture initially used a structure offset of 72 although its reservation map contained only the 16-byte terminator; its correct structure offset is 56. The original RAM fixture has a nonzero reservation plus terminator and correctly retains offset 72. The two offsets are now separate and the host suite passes 67/67.
2. The first QEMU attempt reported FDT UART discovery as an error. The parser was adjusted to tolerate wider cell counts on unrelated buses while keeping selected resource decoding bounded to u64. The subsequent QEMU run emitted the required PASS marker and raw UART bus register. No MMIO access was added.

## Documentation updated

- `boards/riscv64/DRIVER_HAL_MATRIX.md`: new FDT discovery boundary and raw `reg` caveat.
- `docs/ARCHITECTURE_STATUS.md`: 2026-11-16 integration evidence and remaining limitations.
- This checkpoint records the exact validation scope.

## Remaining work — not complete

- Parent-bus `ranges` translation and validated MMIO mapping.
- HAL implementation, UART init/read/flush, clock/reset providers, PLIC discovery/routing.
- Physical HiFive Unmatched, VisionFive 2 and Lichee Pi 4A boot validation; LAVA.
- Dynamic task stacks/frames and independent per-process Sv39 page tables.
- Safe user-buffer copy-in/copy-out, general VM teardown and blocking IPC.
- VFS/handle integration, persistent block storage, mount support and permissions.
- General executable loader/linker and complete Win32 PE/DLL/ABI compatibility.
- Full graphics, input, audio and system-service integration.

No 100% completion, production stability, physical-board boot or Windows game execution is claimed.
