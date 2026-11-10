# OpenXzuff UI 1.0 — Architecture Status

## Current source layout

- `kernel/`: Rust `no_std` RV64 kernel bring-up, including OpenSBI entry, FDT parsing, page/heap allocators, Sv39 bootstrap mappings, trap entry, scheduler and scalar IPC.
- `Kernel/`: C++ kernel contracts and host-testable architecture policy. The C++ syscall/trap policy is not yet the dispatcher used by the Rust RV64 kernel.
- `sys/`: system-service registry and future HAL/driver layers.
- `gui/`: software framebuffer, window manager and desktop-shell work.
- `user/app/`: application manifests and planned Lua/Luau runtimes and installer.
- `runtime/`, `graphics3d/`, `app-hub/`, `boot/`, `boards/`: separate runtime, renderer, service and board-support work; inspect each subsystem's own tests before treating it as integrated.

## Verified RV64 kernel baseline

The current `main` branch has a QEMU-bootable RV64 kernel image. The latest source revision, `cbc196338e124005ea95958a10c430d3f3ade3a0`, passed [RV64 S-Mode Kernel run #134](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/42768638698) on 2026-11-10.

That run executed:

- Host Rust tests: **21 passed, 0 failed**.
- Cross-build target: `riscv64imac-unknown-none-elf`; output is a statically linked RV64 ELF with entry point `0x80200000`.
- OpenSBI v1.3 + QEMU `virt`, 256 MiB, one hart.
- Serial assertions for FDT/timebase, Sv39 bootstrap mapping, physical-page and heap allocator smoke tests, S-mode trap-frame round-trip, scheduler state, SBI TIME preemption/context switching, register/private-stack preservation, explicit U-to-S trap-stack switching and user-SP preservation, U-mode IPC empty-queue `-EAGAIN`, IPC request/reply, U-mode Yield and U-mode Exit.

The run artifact contains the ELF, raw binary and QEMU serial log. Exact results and scope limits are recorded in [the 2026-11-10 checkpoint](checkpoints/2026-11-10-afternoon.md).

## Implemented, but deliberately bounded

- The scheduler test runtime has two S-mode tasks and one U-mode test task on a single hart.
- The Rust-native syscall contract uses `a7` for the syscall ID and `a0..a5` for arguments/results. Current exercised calls are Exit=0, SendMessage=1, ReceiveMessage=2 and Yield=4.
- IPC currently transports one scalar machine word per message through fixed-capacity, non-blocking mailboxes. User pointers are not accepted.
- Sv39 is a bootstrap identity map for one 1 GiB window. Only the probe's code pages and stack pages are user-accessible; this is not a per-process address-space manager.
- U-mode trap entry switches to a supervisor-owned stack before building the trap frame. The QEMU probe now asserts that the saved frame occupies the supervisor stack and that the saved user `sp` remains at its expected initial top. The current test uses one U-mode task and one static supervisor trap stack.

## Not yet established

A successful QEMU run is not proof of production readiness or physical-board compatibility. The following remain separate work:

- Physical SiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot and timer validation; no LAVA hardware result is claimed here.
- Multi-hart/SMP locking, per-hart trap stacks and concurrent scheduler/IPC operation.
- Multiple independent U-mode processes with per-process supervisor trap stacks and address spaces.
- General process loading, user-memory allocation, safe user-buffer copy-in/copy-out, blocking IPC and complete process lifecycle.
- Exhaustive U-mode GPR/CSR preservation tests and explicit U-mode timer-preemption coverage.
- A stable Rust-to-C++ syscall ABI bridge; the active RV64 path is Rust-native.
- Production HAL/device drivers, graphical desktop/compositor and complete application/service integration.

A directory, host-only test, or QEMU smoke test must not be described as a completed subsystem beyond the exact scope above.

## Package naming

- `.lap`: Luau App Package.
- `.luap`: Lua App Package.

Keep the two formats distinct and document their manifests and validation rules before implementing installation.
