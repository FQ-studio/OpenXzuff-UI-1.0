# ARM64 PC (AArch64 + UEFI)

Initial ARM64 PC target is UEFI AArch64. A real implementation needs an EFI application/loader in PE/COFF format, UEFI memory-map handling, GOP framebuffer discovery, ACPI/DT discovery, exception-level setup, GIC/timer setup, and architecture-specific kernel entry. `kernel/arch/arm64/entry.S` is only a stub and is not a bootable UEFI application.
