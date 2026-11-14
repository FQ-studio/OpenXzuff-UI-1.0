# Gravity Kernel VM hardening checkpoint — 2026-11-13

## Scope

This checkpoint applies to the open PR branch `feature/win32-gaming-gravity-port`, not to `main`. It hardens the existing bounded Sv39 probe heap; it does not add general process creation or dynamic page-table roots.

## Changes

- Derive the 2 MiB user heap VA from the 1 GiB bootstrap identity-window base, using a fixed 256 MiB offset. A window based at `0x80000000` uses `0x90000000`; a window based at `0x40000000` uses `0x50000000`.
- Move user protection conversion into a host-testable policy module and enforce W^X for both MapMemory and ProtectMemory.
- Add physical allocator allocation-state queries.
- Make UnmapMemory preflight the entire range and allocator ownership before mutation, invalidate mappings, scrub frames under the kernel root, then return frames to the allocator.
- Keep unmap bounded in stack usage: use two passes instead of a 4 KiB scratch array on the 4 KiB supervisor trap stack.
- Extend the U-mode QEMU probe to reject W+X MapMemory/ProtectMemory, verify failed multi-page unmap is atomic, and reject a second unmap.
- Update ABI and MM documentation to distinguish the bounded probe heap from a general VM manager.

## Validation result

RV64 S-Mode Kernel run #227 completed with SUCCESS on commit `ebc880c07d9d1803093df54cdf0735799675b94d`.

- Host tests: 34 passed, 0 failed.
- Kernel ELF build: succeeded.
- OpenSBI v1.3 + QEMU: succeeded.
- Serial assertions: Sv39 bootstrap and W^X, S/U context switching, timer preemption, IPC, MapMemory, ProtectMemory, W^X rejection, transactional UnmapMemory and double-unmap rejection all emitted PASS.
- Artifact: `openxzuff-gravity-kernel-rv64-s-mode` uploaded by the workflow.

VisionFive 2 readiness run #129 is a host/contract check only; no physical-board or LAVA boot result is claimed.
