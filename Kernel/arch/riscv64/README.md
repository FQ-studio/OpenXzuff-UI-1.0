# RV64 Architecture Layer

The uppercase `Kernel/arch/riscv64` directory is the host-testable C++ contract layer. It contains the RV64 trap-frame layout mirror and a policy-level U-mode trap dispatcher; it does not contain the live assembly entry.

The executable RV64 bring-up path is in the separate lowercase `kernel/` tree:

- `trap.S` saves/restores S-mode GPRs and supervisor trap CSRs.
- `umode_trap.S` switches from U-mode to a supervisor-owned trap stack through `sscratch`, then routes to the live Rust handler.
- The common restore path returns to either S-mode or U-mode through `sret`.
- Two distinct Sv39 roots keep the kernel mapping supervisor-only while the probe-user root grants U access only to its RX code and RW stack pages; the scheduler changes `satp` before restoring each task frame.
- The linked kernel image uses section-aware W^X PTEs: `.text` is supervisor RX, `.rodata` is supervisor R, and `.data/.bss` are supervisor RW. Other bootstrap identity mappings are supervisor RW/NX.
- SBI TIME interrupts preempt the fixed S/U-mode test contexts and drive real saved-frame context switches.
- Live U-mode ECALL handling implements Exit, SendMessage, ReceiveMessage, and Yield for the bootstrap runtime.

The RV64 S-Mode Kernel workflow verifies the section permissions and runtime paths under OpenSBI/QEMU. Run #154 passed on 12 November 2026. This does not establish physical-board boot, general process isolation, a full interrupt-controller implementation, or integration of the C++ dispatcher with the Rust hardware path.
