# 2026-11-22 — Sv39 device-MMIO mapping checkpoint

## Branch and changes

Branch: `feat/fdt-ranges-root-translation`

The generic PLIC compatible fixture correction is now included in the same verified test path. The earlier 96/97 failure was a literal backslash-zero in the fixture; the corrected NUL-terminated string is covered by the passing host suite.

New implementation:
- `kernel/src/mmio_map.rs`: pure 4 KiB page-range planner, including overflow and low-window validation. Host tests cover the QEMU PLIC (1536 pages / three 2 MiB L0 tables), UART (one page), unaligned resources, overflow and out-of-window rejection.
- `kernel/src/vm.rs`: maps translated device windows as identity-mapped 4 KiB leaves. The same supervisor-only RW/NX mapping is installed in both kernel and probe-user roots so S-mode handlers can access devices while a U-mode root is active. It verifies every page's identity translation and PTE permissions in both roots.
- `kernel/src/main.rs`: the FDT-discovered UART and PLIC resources are passed to the VM mapper after Sv39 bootstrap; mapping failures halt boot.
- `kernel/allocator-tests/src/lib.rs`: includes the new planner tests in the host test target.
- `.github/workflows/rv64-s-mode-kernel.yml`: QEMU serial assertions now require both device mappings.
- `boards/riscv64/DRIVER_HAL_MATRIX.md`: documents the implemented boundary and remaining driver work.

Key implementation commits:
- `009bb1276614e5afd107ee49b82c9a1e6ac848e0` — mapping planner
- `a608a9db3299fe50b4397f2cb379e0783da6b59f` — VM mapper
- `860ad422f5dfbe7074fb2314a305014f28940713` — FDT resource integration
- `eb5af0842ad7768eccc4efdf30af30ab0aab5a62` — host test inclusion
- `b47f1558522ad28a5e49401b8b7089777c51f116` — CI assertions

## Verified CI evidence

RV64 S-Mode Kernel run #44998153700 on code commit `b47f155`: **SUCCESS**.
- Host allocator/FDT/MMIO/PLIC/planner suite: **106 passed, 0 failed**.
- RV64 kernel release build: completed; output identified as a statically linked 64-bit RISC-V ELF.
- RV64 Win32 Gravity ecall adapter compile: completed.
- OpenSBI + QEMU boot: completed all serial assertions, including UART and PLIC MMIO mapping in both roots, Sv39 isolation/W^X, timer preemption, context switching, U-mode saved-frame resume, IPC send/receive, and user memory map/protect/unmap.
- QEMU's 15-second timeout termination is expected for this smoke workflow; the log showed no unexpected exit and the required assertions were reached.
- Artifact `openxzuff-gravity-kernel-rv64-s-mode` (ELF, binary, serial log, QEMU DTB/DTS) was uploaded by run #44998153700.

Additional checks on the same code commit:
- VisionFive 2 Kernel Readiness run #44998153706: **SUCCESS** (host smoke and board contract checks only; not physical-board boot).
- Win32 Gravity Backend Bridge run #44998153690: **SUCCESS** (host mock and RV64 adapter cross-compile).

The later commits `218c9bf6d0f424686c7bdeb71651e0d5c76ddc0a` and `db1c49d1c6aa3487ce9df0d7dba04c812dbe3c73` update documentation and remove a duplicate comment; they do not change mapping logic. The later-head RV64 S-Mode Kernel run #44998249038 on `db1c49d` has since completed **SUCCESS** with the same 106/106 host tests, RV64 release ELF build, and QEMU UART/PLIC mapping plus kernel smoke assertions. Its kernel ELF/binary/serial-log/DTB artifact is available until 2026-12-22. The OpenXzuff UI Runtime, OS Contracts and 3D run #44998249043 was still in progress at the latest check.

## Exact implementation boundary

The current mapper:
- accepts translated resources below the first 1 GiB only;
- uses a fixed pool of eight 4 KiB level-0 page tables;
- maps each covering page identity-mapped with supervisor RW, U=0 and X=0;
- shares those supervisor-only mappings in kernel and probe-user roots;
- checks the resulting physical translations and permissions page by page.

Base Sv39 does not define a standard cacheability/device-memory PTE field. Device classification remains platform-PMA-defined; explicit I/O ordering fences are issued after page-table installation. The mapper does **not** perform volatile register access, UART initialization, PLIC initialization, interrupt claim/complete, or real external-interrupt delivery.

No physical HiFive Unmatched, VisionFive 2 or Lichee Pi 4A boot is established by these CI results.

## Next work — do not repeat FDT discovery without a new regression

1. Confirm the later-head RV64 and OpenXzuff Runtime Actions runs.
2. Resolve `interrupts-extended` entries to CPU hart IDs and select the boot hart's S-mode external-interrupt PLIC context.
3. Add a real PLIC register-access layer (priority, enable, threshold, claim/complete), then an injected QEMU external-interrupt round-trip test.
4. Initialize the UART behind the narrow polling-console HAL, including any board-specific clock/reset and register-shift requirements.
5. Generalize the MMIO mapper beyond the initial low 1 GiB window and fixed table pool before claiming broader board support.
