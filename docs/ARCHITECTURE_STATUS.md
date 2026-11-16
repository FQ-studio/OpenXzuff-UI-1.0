# OpenXzuff UI 1.0 — Architecture Status

## Current source layout

- `kernel/`: Rust `no_std` RV64 kernel bring-up, including OpenSBI entry, FDT parsing, page/heap allocators, Sv39 bootstrap mappings, trap entry, scheduler and scalar IPC.
- `Kernel/`: C++ kernel contracts and host-testable architecture policy. The C++ syscall/trap policy is not yet the dispatcher used by the Rust RV64 kernel.
- `sys/`: system-service registry and future HAL/driver layers.
- `gui/`: software framebuffer, window manager and desktop-shell work.
- `user/app/`: application manifests and planned Lua/Luau runtimes and installer.
- `runtime/`, `graphics3d/`, `app-hub/`, `boot/`, `boards/`: separate runtime, renderer, service and board-support work; inspect each subsystem's own tests before treating it as integrated.

## Historical RV64 kernel baseline (2026-11-12)

The RV64-validated source for this historical baseline is commit `0f8d39591b6b8b9a650fe3ea8c17a647b9984d7d`. [RV64 S-Mode Kernel run #156](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461040) completed successfully on 2026-11-12.

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

## Historical OpenXzuff UI Core host baseline (2026-11-12)

[OpenXzuff UI Runtime, OS Contracts and 3D run #166](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461047) completed successfully on 2026-11-12 for the same source revision as the RV64 run above, `0f8d39591b6b8b9a650fe3ea8c17a647b9984d7d`.

- Root CTest: **14/14 passed**; OS contracts, Xzuff Go compiler, Coder Checker and `.xexe` fixture validation passed.
- Xzuff Go / xgo cross-compiled and packaged for Android ARM64 with NDK 27.2.12479018. The workflow verified the resulting `xgo`, `xzuff-go` and `xzuff-check` binaries as AArch64 Android executables. This is a native Termux command-line toolchain, not an APK.
- Luau runtime build, source smoke test and `.lap` package smoke test passed.
- JavaScript host tests: **2/2**; standard Lua host tests: **2/2**; OpenXzuff3D tests: **1/1**; rendered-scene artifact verification passed.
- Two non-fatal unused-parameter warnings remain in the vendored Luau `Ast.h` headers.

[Run #166 artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461047/artifacts/13920654059) contains the host/runtime smoke outputs and Android ARM64 toolchain package.

## Historical VisionFive 2 host readiness (2026-11-12)

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


## 2026-11-15 integration update — feature/win32-gaming-gravity-port

- The feature branch includes a coordinated process/task metadata layer and routes the live three-task test scheduler through it. This is not yet a dynamic process engine: runtime task frames/stacks remain statically provisioned, and the user VM remains one built-in probe address space.
- FDT RAM parsing now rejects memory nodes whose explicit `status` is not `okay` or `ok`; missing status remains enabled. Host tests cover disabled, reserved, okay and ok values. This is memory-node filtering only, not general device-node discovery or `ranges` translation.
- The Gravity SDK starter defines the RV64 ABI v1 syscall and protection constants and provides inline wrappers for Exit, scalar IPC and Yield. The Gravity NDK starter compiles a freestanding C smoke object only; it does not link or launch applications.
- Xzuff Go's signed-i32 exit stub uses ADDIW for sign-extension after LUI, including the INT32_MAX boundary. The compiler still supports only a single `main` returning an integer literal.
- The Core workflow now includes the SDK/NDK compile-only check and is triggered by SDK, NDK and Win32 bridge source changes.
- No physical-board boot, production HAL, PLIC, persistent filesystem, multi-process user VM, general executable loader, or Win32 binary execution is claimed.


## Latest integration validation — 2026-11-15

Source branch: `feature/win32-gaming-gravity-port`, latest code commit `4fcb7fe34da3d984de5a46a29a110bd1c525dd10`. The branch has been synchronized with `main` at `ead5ccec0d675dd6274b814379219ec05ba0fdd6`; GitHub compare confirms 0 commits behind main; the branch contains the current main head. PR #12 remains open and mergeable; it has not been merged.

### Gravity Kernel RV64

[RV64 S-Mode Kernel run #43688778785](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688778785) completed successfully on the latest code commit.

- Win32 Gravity RV64 adapter cross-compile: passed with the shared SDK ABI header in its include path.
- Rust host tests: **60 passed, 0 failed**.
- Kernel ELF cross-build: passed.
- OpenSBI + QEMU `virt` smoke boot: passed all workflow serial assertions, including FDT RAM parsing, Sv39 W^X, S-mode trap save/restore, timer preemption/context switch, U-mode timer saved-frame resume, IPC, U-mode Map/Protect/Unmap, Yield and Exit.
- QEMU is intentionally terminated by the workflow's 15-second timeout after all required assertions. This is not a long-duration stress or production-stability test.
- [RV64 ELF, binary and QEMU serial-log artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688778785/artifacts/14151163776).

### Board-readiness host checks

[VisionFive 2 Kernel Readiness run #43688778856](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688778856) completed successfully:

- Strict-warning host build: passed.
- CTest: **16/16 passed**.
- Documented RV64 boot-contract checks: passed.
- This is host/contract validation only; no physical VisionFive 2 boot or LAVA hardware result is claimed.

### Win32 Gravity bridge and shared SDK ABI

[Win32 Gaming Gravity Backend Bridge run #43688778782](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688778782) completed successfully on the latest code commit:

- Host bridge CTest: **2/2 passed**.
- RV64 adapter cross-compile: passed.
- Adapter disassembly: **8 `ecall` instructions**.
- `gravity_host_abi.h` now derives syscall IDs and protection flags from `sdk/gravity/sysroot/usr/include/gravity/abi.h`; CMake and both adapter workflows include the shared sysroot. Host compile-time assertions verify the mapping.

These checks validate the host mock contract and adapter code generation. They do not show that the C++ adapter is linked into the kernel ELF or invoked by the built-in QEMU U-mode task.

### OpenXzuff UI Core and SDK/NDK

[Core run #43688540402](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688540402) completed successfully on the immediately preceding documentation/CI revision:

- Root CTest: **16/16 passed**.
- Xzuff Go `.xexe` fixture: 44-byte RV64 image generated; entry returns status 42; host loader validation passed.
- Gravity RV64 NDK: freestanding C object compiled and disassembly contained `ecall`.
- Xzuff Go / xgo Android ARM64 cross-build and package: passed.
- Luau runtime build and source/package smoke: passed.
- JavaScript CTest: **2/2**; standard Lua CTest: **2/2**; OpenXzuff3D CTest: **1/1**; render smoke and image verification: passed.
- [Core smoke artifacts](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688540402/artifacts/14150619731).

The newer [Core run #43688778784](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688778784) for the shared-SDK/Win32 integration commit completed successfully. Root CTest **16/16**, the 44-byte `.xexe` return-42 fixture and host loader validation, Gravity NDK object build plus `ecall` disassembly check, Android ARM64 Xzuff Go cross-build/package, Luau runtime/source/package smoke, JavaScript **2/2**, standard Lua **2/2**, OpenXzuff3D **1/1**, render smoke and image verification all passed. [Latest Core smoke artifacts](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43688778784/artifacts/14151582419).

### Remaining integration boundary

- FDT RAM parsing rejects unavailable memory nodes; `has_compatible` now matches exact compatible entries only on nodes whose own and ancestor `status` is absent or `okay`/`ok`, and rejects malformed status strings. It still returns only a boolean: node paths, `reg` resources, inherited bus cell widths, `ranges` translation, clocks/resets and interrupt-parent routing are not complete.
- HAL MMIO validation, UART initialization/receive/flush, PLIC routing and physical-board boot remain unverified.
- Process/task manager coordinates metadata with a fixed-capacity scheduler, but runtime stacks/frames are static; there are not yet independent dynamic per-process Sv39 roots, safe copy-in/copy-out or full VM teardown.
- RAM VFS is bounded and in-memory; handles are not fully backed by VFS/device objects. Persistent block storage, mounts and file permissions remain incomplete.
- IPC remains scalar and non-blocking; no general wait queues or shared-memory IPC.
- No general `.xexe` loader/linker, PE32/PE32+ loader, Win32 API/DLL compatibility runtime or x86/x64-to-RV64 translator exists. No Windows game is claimed to run.
- No production stability or 100% Core completion is claimed.


## 2026-11-16 afternoon — FDT node discovery and QEMU integration

Code revision: `66449dcb3af4ae6e9d70011e690e102ac7df7bdf` on
`feature/win32-gaming-gravity-port`.

### FDT discovery

- Added `find_compatible_node`, which selects the first node whose exact
  NUL-delimited compatible list matches and whose own/ancestor status is
  available.
- Returns the borrowed node name and first `reg` tuple. The tuple is decoded
  using the parent bus's `#address-cells` and `#size-cells`; size is optional
  when the bus declares zero size cells.
- Unselected buses can declare up to four address/size cells without aborting
  discovery. A selected resource must fit the current u64 decoder (one/two
  address cells, zero/one/two size cells).
- The result remains a raw parent-bus address. Ancestor `ranges` translation,
  validated MMIO mapping, clocks, resets and interrupt routing are not
  implemented by this change.
- Boot diagnostics probe the three board families' UART compatible candidates
  plus the QEMU `ns16550a` fallback. This does not initialize or access UART
  MMIO.

### Evidence on code revision

- [RV64 S-Mode Kernel run #43816421114](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421114) — **SUCCESS**.
  - Host Rust tests: **67 passed, 0 failed**.
  - Win32 Gravity RV64 adapter cross-compile: passed.
  - RV64 kernel ELF build: passed.
  - OpenSBI + QEMU `virt`: all required serial assertions passed, including
    FDT UART compatible-node discovery. QEMU reported raw bus address
    `0x10000000`, size `0x100` for the `ns16550a` candidate.
  - QEMU is stopped by the 15-second smoke timeout after required markers;
    this is not long-duration stability evidence.
  - [Kernel ELF, binary and serial log artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421114/artifacts/14203180179).
- [VisionFive 2 Kernel Readiness #43816421040](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421040) — **SUCCESS**; strict-warning host build and CTest **16/16**. This is not a physical-board boot.
- [Win32 Gravity Bridge #43816421214](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421214) — **SUCCESS**; host CTest **2/2** and RV64 adapter cross-compile passed.
- [OpenXzuff UI Core #43816421109](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421109) — **SUCCESS** on the same code revision `66449dcb3af4ae6e9d70011e690e102ac7df7bdf`.
  - Root CTest: **16/16 passed**.
  - Xzuff Go/Coder Checker tests passed; compiler produced a 44-byte RV64 `.xexe` fixture whose entry exits with status 42; host-loader validation passed.
  - Gravity RV64 NDK example compiled; Android ARM64 Xzuff Go/xgo cross-build and package succeeded.
  - Luau runtime build, source smoke and `.lap` pack/load smoke passed.
  - JavaScript CTest **2/2**, standard Lua CTest **2/2**, OpenXzuff3D CTest **1/1**, render smoke and image verification passed.
  - [Core smoke-test artifact](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43816421109/artifacts/14202259895).
  - Vendored Luau emitted non-fatal bidi-character and unused-parameter warnings.

### Remaining boundaries

- No ancestor `ranges` translation or validated MMIO region; UART is discovered,
  not initialized or read/written by the kernel driver.
- HAL, physical UART, PLIC, clocks/resets, physical-board boot and LAVA remain
  incomplete.
- Runtime tasks/stacks are still statically provisioned; independent
  per-process Sv39 roots, general VM teardown and safe user-buffer copies remain
  incomplete.
- VFS remains bounded RAM-only; persistent storage, mount integration and
  handle-backed filesystem objects remain incomplete.
- IPC remains scalar and non-blocking. No general Win32 PE/DLL loader or
  x86/x64-to-RV64 execution layer exists.
- No production-stability or 100% Core completion claim is made.
