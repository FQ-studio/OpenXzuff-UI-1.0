# OpenXzuff UI 1.0 — Architecture Status

## Current source layout

- `kernel/`: Rust `no_std` RV64 kernel bring-up, including OpenSBI entry, FDT parsing, page/heap allocators, Sv39 bootstrap mappings, trap entry, scheduler and scalar IPC.
- `Kernel/`: C++ kernel contracts and host-testable architecture policy. The C++ syscall/trap policy is not yet the dispatcher used by the Rust RV64 kernel.
- `sys/`: system-service registry and future HAL/driver layers.
- `gui/`: software framebuffer, window manager and desktop-shell work.
- `user/app/`: application manifests and planned Lua/Luau runtimes and installer.
- `runtime/`, `graphics3d/`, `app-hub/`, `boot/`, `boards/`: separate runtime, renderer, service and board-support work; inspect each subsystem's own tests before treating it as integrated.

## Verified RV64 kernel baseline

The latest RV64-validated kernel source is commit `0f8d39591b6b8b9a650fe3ea8c17a647b9984d7d`. [RV64 S-Mode Kernel run #156](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461040) completed successfully on 2026-11-12.

That run executed:

- Host Rust tests: **23 passed, 0 failed**.
- RV64 cross-build target: `riscv64imac-unknown-none-elf`; kernel ELF build succeeded.
- OpenSBI + QEMU `virt`, 256 MiB, one hart; all required serial assertions passed.
- Full-page Sv39 permission verification: every 4 KiB page in linker-defined `.text`, `.rodata`, `.data` and `.bss` was checked in both roots. The verifier accounts for the U-mode probe text in `.text` and private user stack in `.bss`.
- W^X policy: kernel text RX, rodata R, data/BSS RW; U-mode probe text U-RX, private user stack U-RW; kernel and supervisor trap-stack pages remain supervisor-only.
- All 16 per-task supervisor trap-stack pages were checked in both kernel and probe-user roots.
- Existing FDT/timebase, page/heap allocator, S-mode trap round-trip, scheduler, timer preemption, U-mode timer saved-frame resume, unknown syscall `-ENOSYS`, U-mode IPC, Yield and Exit smoke assertions also passed.

Relevant serial evidence includes:

- `Sv39 kernel W^X section permissions: PASS; text=RX rodata=R data=RW`
- `Per-task supervisor trap stacks: PASS; slots=16; supervisor-only`
- `U-mode timer preemption + saved-frame resume: PASS`
- `U-mode unknown syscall -ENOSYS: PASS`
- `U-mode Yield syscall context switch + Sv39 isolation: PASS`
- `U-mode Exit syscall + scheduler task exit: PASS`

The [run #156 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461040/artifacts/13920398891) contains the ELF, raw binary and QEMU serial log. QEMU's 15-second smoke-test timeout is accepted only after the required serial assertions pass; it is not a long-duration stability test.

The W^X layout and range-verification contract are documented in [RV64 Sv39 bootstrap mappings and W^X verification](design/rv64-sv39-wx.md).

## Verified OpenXzuff UI Core host baseline

[OpenXzuff UI Runtime, OS Contracts and 3D run #166](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461047) completed successfully on 2026-11-12 for the same source revision as the RV64 run above, `0f8d39591b6b8b9a650fe3ea8c17a647b9984d7d`.

- Root CTest: **14/14 passed**; OS contracts, Xzuff Go compiler, Coder Checker and `.xexe` fixture validation passed.
- Xzuff Go / xgo cross-compiled and packaged for Android ARM64 with NDK 27.2.12479018. The workflow verified the resulting `xgo`, `xzuff-go` and `xzuff-check` binaries as AArch64 Android executables. This is a native Termux command-line toolchain, not an APK.
- Luau runtime build, source smoke test and `.lap` package smoke test passed.
- JavaScript host tests: **2/2**; standard Lua host tests: **2/2**; OpenXzuff3D tests: **1/1**; rendered-scene artifact verification passed.
- Two non-fatal unused-parameter warnings remain in the vendored Luau `Ast.h` headers.

[Run #166 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461047/artifacts/13920654059) contains the host/runtime smoke outputs and Android ARM64 toolchain package.

## VisionFive 2 host readiness

[VisionFive 2 Kernel Readiness run #49](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461210) completed successfully on the same source revision. CMake host build, strict-warning compilation, CTest **14/14**, and board boot-contract source checks passed. The workflow explicitly notes that these checks do not prove physical board boot or full FDT correctness; no physical VisionFive 2 or LAVA result is claimed.

## Implemented, but deliberately bounded

- The scheduler test runtime has two S-mode tasks and one U-mode test task on a single hart.
- The Rust-native syscall contract uses `a7` for the syscall ID and `a0..a5` for arguments/results. Exercised calls are Exit=0, SendMessage=1, ReceiveMessage=2 and Yield=4; IDs 5 and 6 are test-only assertions for U-mode timer-frame resume and the verified `-ENOSYS` negative test; neither is a public application ABI. The U-mode probe sends unknown ID 99 and checks `-ENOSYS` (`-38`) before asking the kernel to emit its PASS marker.
- IPC currently transports one scalar machine word per message through fixed-capacity, non-blocking mailboxes. User pointers are not accepted.
- Sv39 is a bootstrap identity map for one 1 GiB window. The verifier walks every 4 KiB page in `.text`, `.rodata`, `.data` and `.bss` in both roots, with explicit U-mode probe-text and user-stack exceptions; only the probe's code pages and stack pages are user-accessible. This is not a per-process address-space manager.
- U-mode trap entry switches to the supervisor trap stack assigned to the current scheduler task before building the 288-byte frame. Sixteen page-aligned slots are reserved by `TaskId`; QEMU verifies their layout and checks every slot is supervisor-only in the user page table. The current runtime still exercises only one U-mode task (task 2) and one user stack on one hart; allocating slots does not yet prove multi-U-task or SMP operation.

## Not yet established

A successful QEMU run is not proof of production readiness or physical-board compatibility. The following remain separate work:

- Physical SiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot and timer validation; no LAVA hardware result is claimed here.
- Multi-hart/SMP locking, per-hart trap stacks and concurrent scheduler/IPC operation.
- Multiple independent U-mode processes with distinct user stacks, address spaces, lifecycle and actual simultaneous use of the per-task trap-stack slots.
- General process loading, user-memory allocation, safe user-buffer copy-in/copy-out, blocking IPC and complete process lifecycle.
- Exhaustive U-mode GPR/CSR preservation across ECALL, timer and context-switch paths; the current timer test proves a U-mode timer interrupt and saved-frame resume, but not every register sentinel.
- A stable Rust-to-C++ syscall ABI bridge; the active RV64 path is Rust-native.
- Production HAL/device drivers, graphical desktop/compositor and complete application/service integration.

A directory, host-only test, or QEMU smoke test must not be described as a completed subsystem beyond the exact scope above.

## Package naming

- `.lap`: Luau App Package.
- `.luap`: Lua App Package.

Keep the two formats distinct and document their manifests and validation rules before implementing installation.
