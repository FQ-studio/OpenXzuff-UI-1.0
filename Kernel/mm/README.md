# Kernel Memory Management

## Implemented: RV64 bootstrap Sv39 translation and bounded probe heap

The executable Rust kernel in lowercase `kernel/src/vm.rs` now builds separate supervisor-kernel and probe-user roots and activates Sv39.

- One 1 GiB identity-mapped bootstrap window covers the linked kernel and firmware FDT.
- Level-1 2 MiB leaves outside the linked image are supervisor RW and non-executable.
- The linked kernel image is split with a dedicated 4 KiB level-0 table:
  - `.text`: supervisor RX.
  - `.rodata`: supervisor R, not writable or executable.
  - `.data` and `.bss`: supervisor RW, non-executable.
- The probe-user root mirrors supervisor mappings but grants U access only to the built-in probe's RX code pages, RW stack pages and allocated leaves in a bounded 2 MiB heap window. The heap VA is derived from the 1 GiB bootstrap window base (offset 256 MiB), so it does not assume QEMU's 0x80000000 RAM base.
- Task selection switches `satp` before restoring the selected frame and executes `sfence.vma`.
- The built-in U-mode probe has live MapMemory, ProtectMemory and UnmapMemory syscalls. Physical frames are allocated only from the identity-mapped window, zero-filled before mapping, ownership-checked before unmap, scrubbed before free, and followed by a TLB flush. User PTE permissions enforce W^X.
- Linker section boundaries are page-aligned so section permissions cannot share a writable/executable page.

The RV64 S-Mode Kernel PR check #225 passed the bounded Map/Protect/Unmap path under OpenSBI/QEMU on 12 November 2026. The new W^X, allocator ownership and atomic-unmap checks are pending revalidation.

## Explicit limitations

This remains a bounded bring-up implementation, not a general virtual-memory manager. It uses static page-table storage, one 1 GiB identity window and one statically provisioned probe-user address space with a 2 MiB heap arena. The linked image and probe must fit in the same 2 MiB L1 region. Dynamic per-process roots, page-table-page allocation/reclamation, arbitrary process map/unmap, demand paging, copyin/copyout, ASIDs, remote TLB shootdown and SMP-safe page-table mutation are not implemented. The heap arena is still single-task and fixed-size; it is not a general process heap.
