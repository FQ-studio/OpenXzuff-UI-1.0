# Boot integration

Boot is currently a design/integration task, not a completed loader.

- RV64 bring-up path: OpenSBI-style payload entry in `kernel/arch/riscv/entry.S`.
- ARM64 PC path: target UEFI AArch64 first; the current assembly entry is only a stub and is not a UEFI PE/COFF application.
- Each board needs documented firmware settings, memory map, device tree or ACPI data, and verified load/entry contract before a bootable image can be claimed.

Do not use one board's load address or peripheral map as if it applied to every board.