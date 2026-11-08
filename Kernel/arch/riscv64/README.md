# RV64 Architecture Layer

Reserved for the C++ Gravity Kernel RV64 architecture layer.

Status: this uppercase `Kernel/arch/riscv64` directory remains a contract-layer scaffold. The separate executable Rust S-mode bring-up image lives in lowercase `kernel/`; it now has an OpenSBI/QEMU-tested S-mode breakpoint trap round-trip. That does not provide U-mode entry, syscall integration, or physical-board boot.
