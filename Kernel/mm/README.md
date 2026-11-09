# Kernel Memory Management

## Implemented: RV64 bootstrap Sv39 translation

The early RV64 kernel now enables Sv39 in satp using kernel/src/vm.rs:

- One 1 GiB identity-mapped window containing the linked kernel and firmware FDT.
- Level-1 2 MiB supervisor leaves for the bootstrap region.
- Dedicated 4 KiB level-0 mappings for the isolated U-mode probe text and its user stack.
- Kernel image, page tables, supervisor stacks, and other bootstrap memory remain U=0.
- A QEMU U-mode load from __kernel_start must fault, while the dedicated U-mode code and stack remain accessible.

The RV64 OpenSBI/QEMU workflow verifies that execution continues after satp is switched to Sv39 and that the U-mode isolation probe completes.

## Explicit limitations

This is an early-boot address-space proof, not a complete virtual-memory manager. The current implementation has one bootstrap root and one 1 GiB window; it does not yet provide per-process page-table roots, dynamic map/unmap, page-fault demand paging, copyin/copyout, W^X section permissions for the whole kernel, ASIDs, TLB shootdown, or SMP-safe page-table mutation. The 2 MiB supervisor bootstrap leaves are intentionally permissive RWX and must be tightened before general user processes are loaded.
