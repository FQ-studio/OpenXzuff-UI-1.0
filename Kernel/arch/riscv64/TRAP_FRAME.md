# RV64 S-mode trap-frame contract

The openxzuff::kernel::riscv64::TrapFrame type defines the C++ memory layout expected by a future supervisor-mode trap entry:

| Offset | Field | Meaning |
|---:|---|---|
| 0–255 | x[0..31] | Interrupted integer registers, indexed by architectural register number |
| 256 | sepc | Faulting or interrupted instruction address |
| 264 | sstatus | Saved supervisor status |
| 272 | scause | Trap cause |
| 280 | stval | Trap-specific value |

The frame is 288 bytes and 16-byte aligned. x[0] is always zero; x[2] is the interrupted stack pointer. The syscall ABI reads the syscall ID from x[17] (a7) and arguments from x[10..15] (a0..a5).

This is only a host-testable layout contract. No trap vector, register-save assembly, sret path, privilege transition, or hardware interrupt handling is implemented by this file.
