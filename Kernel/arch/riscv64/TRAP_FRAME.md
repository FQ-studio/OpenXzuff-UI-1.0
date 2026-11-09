# RV64 trap-frame and entry contract

The live Rust TrapFrame in kernel/src/trap.rs and the assembly paths in kernel/src/trap.S and kernel/src/umode_trap.S use this 288-byte, 16-byte-aligned layout:

| Offset | Field | Meaning |
|---:|---|---|
| 0–255 | x[0..31] | Interrupted integer registers, indexed by architectural register number |
| 256 | sepc | Faulting or interrupted instruction address |
| 264 | sstatus | Saved supervisor status, including SPP/SPIE privilege-return state |
| 272 | scause | Trap or interrupt cause |
| 280 | stval | Trap-specific value, including page-fault address |

For S-origin traps, x[2] stores the interrupted S-mode stack pointer. For U-origin traps, the entry swaps to the supervisor-owned stack through sscratch and stores the interrupted U-mode stack pointer in x[2]. x[0] is represented as zero.

## Live paths

- S-mode entry saves all integer GPRs and supervisor trap CSRs, calls gravity_s_trap_handler, then restores the frame selected by the scheduler and returns with sret.
- U-mode entry uses sscratch to switch to a supervisor-owned trap stack before saving registers.
- The common restore path inspects SPP and restores sscratch for either S-mode or U-mode context.
- Supervisor timer interrupts are dispatched from both S-origin and U-origin contexts.
- U-mode ECALL uses a7 for the syscall ID and a0–a5 for arguments. ABI v1 advances sepc by four bytes for ECALL.
- QEMU probes exercise GPR preservation, a U-mode page fault against supervisor-only memory, SendMessage returning -ENOSYS, and a real Yield context switch.

The C++ openxzuff::kernel::riscv64::TrapFrame header is a host-testable layout mirror. Passing its static assertions alone is not evidence of hardware trap operation; the RV64 QEMU workflow supplies that separate evidence.
