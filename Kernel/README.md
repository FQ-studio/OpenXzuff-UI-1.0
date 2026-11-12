# OpenXzuff UI — Gravity Kernel contract layer

The uppercase `Kernel/` tree is the host-testable C++ contract layer for Gravity Kernel. It is not the executable boot image. The separate lowercase `kernel/` tree contains the Rust RV64 S-mode bring-up kernel that is built and exercised under OpenSBI/QEMU.

## Host-testable C++ contract layer

Implemented here:

- Typed Gravity syscall IDs and a dispatcher using injected hooks for Exit and Yield.
- A host-testable U-mode `ecall` policy: syscall ID from `a7`, arguments from `a0..a5`, result in `a0`, and a four-byte `sepc` advance for returning calls.
- An RV64 trap-frame layout mirror with compile-time offset, size, and alignment assertions.
- An SBI call wrapper with a host fallback.

The C++ dispatcher still returns `-ENOSYS` for SendMessage, ReceiveMessage, MapMemory, and unknown IDs. It is a policy/test boundary and is not the live hardware syscall implementation.

## Separate Rust RV64 runtime

The executable image in lowercase `kernel/` currently has an OpenSBI/QEMU-tested bring-up path:

- Firmware FDT parsing, including memory ranges and timer timebase frequency.
- Sv39 bootstrap translation with distinct supervisor-only kernel and probe-user roots. The linked image has section-aware W^X permissions: `.text` supervisor RX, `.rodata` supervisor R, and `.data/.bss` supervisor RW; the other bootstrap identity-mapped regions are supervisor RW/NX.
- The probe-user root grants U access only to its RX text and RW stack. Task switches change `satp` and execute `sfence.vma`.
- Physical page allocator and a fixed 64 KiB kernel heap with smoke tests.
- S-mode and U-mode trap entry paths, a 288-byte RV64 GPR/CSR frame, `sscratch` supervisor-stack switching, and a common `sret` restore path.
- SBI TIME timer preemption using a 10 ms quantum derived from the FDT timebase.
- A fixed three-context probe runtime: two S-mode tasks with private stacks and one U-mode test task. Timer traps save the interrupted frame, rotate the round-robin scheduler, and restore the selected task.
- Live U-mode Exit and Yield syscalls.
- A bounded scalar IPC transport: per-task FIFO mailboxes, eight messages per mailbox, SendMessage and non-blocking ReceiveMessage syscalls, and a U-mode-to-kernel-task request/reply round-trip.

The `RV64 S-Mode Kernel` GitHub Actions workflow builds the RV64 ELF, runs host tests against the allocator/FDT/scheduler/IPC modules, boots it with OpenSBI in QEMU, and checks the serial assertions. Run #154 completed successfully on 12 November 2026; it is evidence for this QEMU configuration only.

## Explicit limitations

The Rust runtime remains a bring-up kernel, not a general-purpose OS. It has one statically provisioned probe-user root, not a dynamic per-process VM manager. It does not yet have a general `.xexe` loader, page-table-page allocation/reclamation, arbitrary process mappings, user-buffer copyin/copyout, blocking IPC, endpoint capabilities, process resource reaping, SMP-safe scheduling/IPC, a complete interrupt controller/driver stack, or a physical-board LAVA boot result. The uppercase C++ contract layer and lowercase Rust runtime must not be described as one integrated implementation until a real integration boundary is built and tested.
