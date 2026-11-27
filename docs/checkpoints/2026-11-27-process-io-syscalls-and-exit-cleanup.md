# 2026-11-27 — U-mode file syscalls and complete process-handle cleanup

## Repository and branch
- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- PR: [#17 — Close process-owned VFS handles on task exit](https://github.com/FQ-studio/OpenXzuff-UI-1.0/pull/17)
- Branch: `feat/process-io-exit-cleanup`
- Base: `feat/fdt-ranges-root-translation` at `fbfd723e068f2ec34e48bf25dc5f15743e3c90f4`
- PR head at this checkpoint: `f5190b4f2530e0bcc19b2657971f748c09cfe3fb`
- PR remains open; no merge into `main` is claimed.

## Verified baseline before the latest hardening
PR head `077a1cd7ff204f00f3e8d041cdd4884bd9b3df8e` had completed these workflows:
- RV64 S-Mode Kernel [#45942407469](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45942407469): **SUCCESS**; Rust host tests **141/141**, RV64 ecall adapter cross-build, release kernel ELF, OpenSBI/QEMU smoke. QEMU serial log includes UART RX interrupt burst, process-owned VFS capability isolation, and process-exit handle cleanup.
- OpenXzuff UI Core [#45942407438](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45942407438): **SUCCESS**; OS contract CTest **16/16**, JavaScript **2/2**, Lua **2/2**, OpenXzuff3D **1/1**, and artifact upload. Luau build/package smoke and render verification completed in the workflow.
- Win32 Gravity Bridge [#45942407477](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45942407477): **SUCCESS**; host tests **2/2** and RV64 adapter cross-compile.
- VisionFive 2 readiness [#45942407484](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45942407484): **SUCCESS**; host CTest **16/16** and board-contract checks. This is not a physical board boot.

The RV64 kernel binary emitted 21 non-fatal warnings. These remain cleanup work.

## Code review findings and fixes on this checkpoint
1. The first Exit implementation required `closed_handles == 1`. This incorrectly made task exit fail for a process that owned more than one handle, despite `HandleTable::close_all` being a bulk cleanup operation.
2. The U-mode boot smoke now leaves **two** live file capabilities for the exiting process. Exit accepts any positive cleanup count and requires the post-cleanup handle count to be zero. The QEMU assertion now checks the invariant `remaining=0`, rather than hard-coding one closed handle.
3. OPEN now returns `-EINVAL` for invalid rights and `-EFAULT` only for an invalid user path range.
4. READ/WRITE now return `-EINVAL` for zero or oversized transfer lengths and `-EFAULT` for invalid user buffers.
5. The U-mode assembly probe now checks invalid OPEN rights and zero-length READ error semantics.
6. The ABI document and PR description were corrected to state that OPEN/READ/WRITE/CLOSE (IDs 9–12) are already wired through the live U-mode trap path, while remaining limited to built-in task ID 2 and one static user address space.

## Latest verification status
The code/documentation updates have pushed PR head `f5190b4`. New PR Actions were observed:
- RV64 S-Mode Kernel [#45950514800](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45950514800): **QUEUED** at checkpoint time.
- VisionFive 2 readiness [#45950514840](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45950514840): **QUEUED**.
- Win32 Gravity Bridge [#45950514803](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45950514803): **QUEUED**.
- OpenXzuff UI Core [#45950514848](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45950514848): **IN PROGRESS**.

No PASS is claimed for these latest runs until their job logs complete.

## Current implementation boundary
- OPEN opens existing regular files only.
- RAM VFS is bounded and volatile; there is no persistent storage or mount system.
- READ/WRITE are capped at 256 bytes; OPEN path copy is capped at 128 bytes.
- The live file syscall handlers are currently restricted to built-in U-mode task ID 2. Process identity is resolved from TaskManager for that task, but this is not yet a general multi-process syscall service.
- Exit closes ProcessIo handles for the built-in U-mode task. General wait/reap and complete task-resource teardown remain incomplete.
- The user probe still uses one statically provisioned user address space; independent per-process Sv39 roots are not implemented.
- The RV64 NDK is still freestanding C object-only; it has no CRT/libc, production linker, relocation pipeline, or general executable launcher.
- QEMU smoke is bounded validation, not long-duration stability or physical-board boot.

## Next engineering steps
1. Confirm Actions for head `f5190b4`, especially the two-handle QEMU exit path and invalid-argument assertions.
2. Add negative U-mode tests for READ/WRITE invalid user buffers, cross-page ranges, oversized transfers, and foreign/stale handles.
3. Generalize OPEN/READ/WRITE/CLOSE from task-ID-2-only probe handlers to process-derived current-task dispatch.
4. Implement independent Sv39 roots with per-process page-table allocation, switching and teardown.
5. Wire process wait/reap and all task-owned resources, then move RAM VFS toward a storage backend.
