# OpenSBI integration contract (RV64)

OpenXzuff UI is an **S-mode SBI client**; OpenSBI is external M-mode runtime firmware. Do not copy OpenSBI into the kernel or assume the OS itself implements M-mode firmware.

## Entry contract

On entry from a compatible firmware/boot stage, the RISC-V boot convention provides:
- `a0`: boot hart ID
- `a1`: physical address of the firmware-provided FDT, aligned to 8 bytes
- kernel executes in S-mode when entered through OpenSBI
- interrupts are not assumed enabled; the kernel must establish its trap vector and interrupt state
- parse the supplied FDT before initializing RAM, UART, interrupt controllers, timers, or other devices

The exact boot chain depends on board firmware. Do not assume every board jumps directly from OpenSBI to the kernel; U-Boot may be the next stage.

## SBI client implemented in Gravity Kernel

`Kernel/include/openxzuff/kernel/sbi.hpp` and `Kernel/src/sbi.cpp` provide a small RV64 ecall wrapper for:
- BASE: get SBI specification version and probe an extension
- TIME: set a timer deadline
- HSM: start a secondary hart
- SRST: request system shutdown/reboot

The wrapper does not replace interrupt-controller setup, trap handling, scheduler integration, or FDT parsing. Check extension availability before calling optional services; handle every SBI error return. HSM startup additionally requires a valid S-mode entry point and per-hart startup protocol.

## Build and test status

- Host builds intentionally return SBI error `-2` (not supported) instead of executing an ecall.
- A host smoke test can verify that guard.
- RV64 cross-compilation and hardware boot tests are still required.
- OpenSBI firmware binaries and board DTBs are **not bundled** by these source changes. Use the firmware supplied by the board/vendor or build a pinned OpenSBI release with its own license notices.

Reference: [RISC-V SBI specification](https://docs.riscv.org/reference/sbi/intro.html), [OpenSBI project](https://github.com/riscv/opensbi).