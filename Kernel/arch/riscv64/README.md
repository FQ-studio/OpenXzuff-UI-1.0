# RV64 Architecture Layer

The uppercase `Kernel/arch/riscv64` directory is the host-testable C++ contract layer. It contains the RV64 trap-frame layout mirror and a policy-level U-mode trap dispatcher; it does not contain the live assembly entry.

The executable RV64 bring-up path is in the separate lowercase `kernel/` tree:

- `trap.S` saves/restores S-mode GPRs and supervisor trap CSRs.
- `umode_trap.S` switches from U-mode to a supervisor-owned trap stack through `sscratch`, then routes to the live Rust handler.
- The common restore path returns to either S-mode or U-mode through `sret`.
- SBI TIME interrupts preempt the fixed S/U-mode test contexts and drive real saved-frame context switches.
- Live U-mode ECALL handling implements Exit, SendMessage, ReceiveMessage, and Yield for the bootstrap runtime.

OpenSBI/QEMU workflow assertions provide evidence for these paths. They do not establish physical-board boot, general process isolation, a full interrupt-controller implementation, or integration of the C++ dispatcher with the Rust hardware path.
