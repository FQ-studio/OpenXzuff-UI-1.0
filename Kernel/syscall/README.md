# System Call Boundary

The stable Gravity ABI v1 syscall IDs are declared in `Kernel/include/openxzuff/kernel/contracts.hpp`. The host-testable C++ dispatcher in `Kernel/src/syscall.cpp` remains a policy-boundary test double with injected hooks; it currently routes Exit (0) and Yield (4) through those hooks and returns `-ENOSYS` for SendMessage (1), ReceiveMessage (2), MapMemory (3), and unknown IDs.

The live RV64 path is separate and executes in `kernel/src/trap.rs` and `kernel/src/runtime.rs`:

- Exit (0): exits the built-in U-mode probe task and schedules another runnable task.
- SendMessage (1): sends one scalar word to a destination task's bounded FIFO mailbox.
- ReceiveMessage (2): non-blocking receive; returns payload in a0 and sender task ID in a1, or `-EAGAIN`.
- Yield (4): saves the current U-mode trap frame, rotates the run queue, and resumes the task later.

The live IPC implementation is single-hart and does not dereference user pointers. ABI v1 returning calls preserve a1-a5 except where a syscall defines a secondary output (ReceiveMessage uses a1). The RV64 register-save layout is specified by `Kernel/include/openxzuff/kernel/riscv64/trap_frame.hpp`.

The C++ dispatcher does not install a trap vector, switch tasks, implement hardware IPC, map pages, or execute ecall. Its status must not be conflated with the live Rust RV64 runtime.
