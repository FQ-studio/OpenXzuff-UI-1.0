# OpenXzuff UI 1.0 — Architecture Status

## Current source layout

- `kernel/`: Rust `no_std` RV64 kernel bring-up, including OpenSBI entry, FDT parsing, page/heap allocators, Sv39 bootstrap mappings, trap entry, scheduler and scalar IPC.
- `Kernel/`: C++ kernel contracts and host-testable architecture policy. The C++ syscall/trap policy is not yet the dispatcher used by the Rust RV64 kernel.
- `sys/`: system-service registry and future HAL/driver layers.
- `gui/`: software framebuffer, window manager and desktop-shell work.
- `user/app/`: application manifests and planned Lua/Luau runtimes and installer.
- `runtime/`, `graphics3d/`, `app-hub/`, `boot/`, `boards/`: separate runtime, renderer, service and board-support work; inspect each subsystem's own tests before treating it as integrated.

## Verified RV64 kernel baseline

The current `main` branch has a QEMU-bootable RV64 kernel image. The latest RV64-validated source revision, `c3ca6626e4ba47482669bc6f41346b9da413fd45`, passed [RV64 S-Mode Kernel run #147](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42972561614) on 2026-11-11.

That run executed:

- Host Rust tests: **23 passed, 0 failed**; the suite includes FDT timebase 32/64-bit parsing, zero/missing/bad-width rejection, RAM/reservation parsing, allocator, scheduler and IPC cases.
- Cross-build target: `riscv64imac-unknown-none-elf`; output is a statically linked RV64 ELF with entry point `0x80200000`.
- OpenSBI v1.3 + QEMU `virt`, 256 MiB, one hart.
- Serial assertions for FDT/timebase, Sv39 bootstrap mapping, all 16 per-task supervisor trap-stack slots being page-aligned and supervisor-only, physical-page and heap allocator smoke tests, S-mode trap-frame round-trip, scheduler state, SBI TIME preemption/context switching, explicit U-mode timer preemption and saved-frame resume, unknown U-mode syscall `-ENOSYS` handling, U-to-S trap-stack switching and user-SP preservation, U-mode IPC empty-queue `-EAGAIN`, IPC request/reply, U-mode Yield and U-mode Exit.

The [run #147 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42972561614/artifacts/13833589291) contains the ELF, raw binary and QEMU serial log. Exact results and scope limits are recorded in [the 2026-11-11 checkpoint](checkpoints/2026-11-11-afternoon.md).


## Verified OpenXzuff UI Core host baseline

[OpenXzuff UI Runtime, OS Contracts and 3D run #153](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971393352) passed on 2026-11-11 for source commit `3e8f1ed6c8b3725085391a0e39f4e9cbb4e1250a`:

- Root CTest: **14/14 passed**; Xzuff Go compiler/.xexe fixture validation passed.
- Xzuff Go / xgo cross-compiled and packaged for Android ARM64 using NDK 27.2.12479018.
- Luau runtime and .lap package smoke tests passed.
- JavaScript host tests: **2/2**; standard Lua host tests: **2/2**; OpenXzuff3D tests: **1/1**; rendered-scene artifact verification passed.
- Removed the unused `clamp01` helper in `graphics3d/src/renderer.cpp`; the renderer's unused-function warning is absent from run #153. Two non-fatal unused-parameter warnings remain in the vendored Luau `Ast.h` headers.

[Run #153 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42971393352/artifacts/13833493328) contains the host/runtime smoke outputs.

## Implemented, but deliberately bounded

- The scheduler test runtime has two S-mode tasks and one U-mode test task on a single hart.
- The Rust-native syscall contract uses `a7` for the syscall ID and `a0..a5` for arguments/results. Exercised calls are Exit=0, SendMessage=1, ReceiveMessage=2 and Yield=4; ID 5 is a test-only assertion that the U-mode timer-preempted frame resumed, not a public application ABI. The U-mode probe also sends unknown ID 99 and verifies `-ENOSYS` (`-38`).
- IPC currently transports one scalar machine word per message through fixed-capacity, non-blocking mailboxes. User pointers are not accepted.
- Sv39 is a bootstrap identity map for one 1 GiB window. Only the probe's code pages and stack pages are user-accessible; this is not a per-process address-space manager.
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
