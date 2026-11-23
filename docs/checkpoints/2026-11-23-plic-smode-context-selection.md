# 2026-11-23 — PLIC S-mode context selection (verified)

## Branch and revision

- Branch: `feat/fdt-ranges-root-translation`
- Verified PR head: `99f3f5f659a0707834ca6074d1612fb115272708`
- PR #15 remains open and targets `feature/win32-gaming-gravity-port`; it is not merged into `main`.

## Implementation

- Resolve `interrupts-extended` CPU interrupt-controller phandles to CPU nodes and architectural hart IDs.
- Read hart IDs from CPU `reg` using `/cpus/#address-cells`; do not infer them from node unit-address text.
- Require an enabled `riscv,cpu-intc` child with `interrupt-controller` and exactly one interrupt cell.
- Select cause 9 (S-mode external interrupt) for the boot hart passed by OpenSBI.
- Preserve original PLIC context indexes when OpenSBI masks an M-mode entry with `0xffffffff`.
- Early boot validates the selected context's threshold and claim/complete register addresses.
- QEMU workflow asserts the one-hart virt selection: hart 0, context 1, cause 9.

## Exact-head CI evidence

All four workflows associated with head `99f3f5f` completed successfully:

1. RV64 S-Mode Kernel — run [#45085120693](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45085120693): **SUCCESS**.
   - Host FDT/MMIO/PLIC/kernel test suite: **110 passed, 0 failed**.
   - RV64 release kernel build completed; `file` identified a statically linked ELF64 RISC-V executable.
   - RV64 Win32 Gravity ecall adapter cross-compile completed.
   - OpenSBI + QEMU serial assertions passed, including `FDT PLIC S-mode context: PASS; hart=0; context=1; cause=9`, UART/PLIC Sv39 mapping, timer preemption, context switching, U-mode saved-frame resume, IPC, and Sv39 map/protect/unmap.
   - Artifact ID `14779101843` contains the kernel ELF/binary, serial log and QEMU DTB/DTS.
   - QEMU was terminated by the workflow's expected 15-second timeout after the required serial assertions; this is a smoke test, not a long-duration stability test.

2. OpenXzuff UI Runtime/Core — run [#45085120804](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45085120804): **SUCCESS**.
   - Root CTest: **16/16**.
   - Xzuff Go generated and validated a 44-byte RV64 `.xexe` fixture whose entry exits with status 42.
   - Gravity RV64 NDK example built.
   - Xzuff Go / xgo Android ARM64 toolchain cross-compiled and packaged.
   - Luau source and `.lap` package smoke tests passed.
   - JavaScript CTest: **2/2**; Lua CTest: **2/2**; OpenXzuff3D CTest: **1/1**; render verification step succeeded.
   - Artifact ID `14778588784` uploaded.

3. VisionFive 2 Kernel Readiness — run [#45085120726](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45085120726): **SUCCESS**, 16/16 host CTests and board contract checks. This is not a physical-board boot.

4. Win32 Gravity Backend Bridge — run [#45085120701](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45085120701): **SUCCESS** for both host bridge tests (2/2) and the RV64 adapter cross-compile; output was identified as ELF64 RISC-V.

## Warnings and limits

- The RV64 release build succeeds but emits 31 Rust dead-code/unused warnings. They are not build failures; clean them only when the associated modules are integrated rather than suppressing them globally.
- The Core build emits warnings from vendored Luau sources (including bidi-control and unused-parameter diagnostics); Core workflow still completed successfully.
- Context selection is discovery/validation only. It does **not** yet perform volatile PLIC register access, configure priorities, enable source bits, set `sie.SEIE`, claim/complete interrupts, or prove end-to-end external interrupt delivery.
- UART is discovered and mapped but is not initialized through a physical UART driver.
- The Sv39 evidence covers kernel and probe-user roots, not a complete independent address space for every process.
- No physical HiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot is established.

## Next implementation

1. Add a bounds-checked volatile PLIC register-access layer using the validated mapped region and selected context.
2. Exercise safe context threshold/readback and claim-register no-pending behavior in QEMU before enabling any interrupt source.
3. Parse the UART interrupt specifier from FDT, initialize the appropriate UART behind the polling-console HAL, then add a QEMU external-interrupt round-trip test.
4. Continue to independent process address spaces, blocking IPC and persistent VFS after the device path has real integration evidence.

Do not repeat compatible-node discovery or the context-selection parser without a new failing regression.
