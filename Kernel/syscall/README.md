# System Call Boundary

The stable Gravity ABI v1 syscall IDs are declared in Kernel/include/openxzuff/kernel/contracts.hpp. The host-testable dispatcher in Kernel/src/syscall.cpp currently routes:

- Exit (0) to an injected process-exit hook.
- Yield (4) to an injected scheduler hook.
- SendMessage (1), ReceiveMessage (2), MapMemory (3), and unknown IDs to -ENOSYS.

Returning calls use a0 for the result or negative errno. ABI v1 preserves a1-a5. The RV64 register-save layout is specified by Kernel/include/openxzuff/kernel/riscv64/trap_frame.hpp.

The dispatcher is deliberately independent of hardware. It does not install a trap vector, switch tasks, implement IPC, map pages, or execute ecall. Those require separate kernel and QEMU/hardware milestones.
