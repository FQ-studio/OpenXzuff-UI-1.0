# Kernel Memory Management

## Implemented: RV64 bootstrap Sv39 translation

The executable Rust kernel in lowercase `kernel/src/vm.rs` now builds separate supervisor-kernel and probe-user roots and activates Sv39.

- One 1 GiB identity-mapped bootstrap window covers the linked kernel and firmware FDT.
- Level-1 2 MiB leaves outside the linked image are supervisor RW and non-executable.
- The linked kernel image is split with a dedicated 4 KiB level-0 table:
  - `.text`: supervisor RX.
  - `.rodata`: supervisor R, not writable or executable.
  - `.data` and `.bss`: supervisor RW, non-executable.
- The probe-user root mirrors supervisor mappings but grants U access only to the built-in probe's RX code pages and RW stack pages.
- Task selection switches `satp` before restoring the selected frame and executes `sfence.vma`.
- Linker section boundaries are page-aligned so section permissions cannot share a writable/executable page.

The RV64 S-Mode Kernel workflow checks these PTE permissions in both roots, user/kernel isolation, timer preemption, syscall behavior and IPC under OpenSBI/QEMU. Run #154 passed these checks on 12 November 2026.

## Explicit limitations

This remains a bootstrap mapping, not a general virtual-memory manager. It uses static page-table storage, one 1 GiB identity window and one statically provisioned probe-user address space. The linked image and probe must fit in the same 2 MiB L1 region. Dynamic per-process roots, page-table-page allocation/reclamation, arbitrary map/unmap, demand paging, copyin/copyout, ASIDs, TLB shootdown and SMP-safe page-table mutation are not implemented.
