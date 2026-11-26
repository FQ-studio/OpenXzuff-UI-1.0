# OpenXzuff Hardware Abstraction Layer

The host-testable HAL boundary currently contains:

- validated MMIO resource descriptors with overflow-safe sub-range checks;
- firmware-discovered device descriptors carrying a device-tree path, exact
  compatible string, register window, and optional interrupt metadata;
- a device catalog that rejects malformed descriptors and duplicate paths.

This is an integration contract only. It does not map MMIO, configure clocks or
resets, route interrupts, or probe hardware. Those operations must be supplied
by the RV64 kernel's FDT/MMIO/PLIC layer before a production driver may bind.
