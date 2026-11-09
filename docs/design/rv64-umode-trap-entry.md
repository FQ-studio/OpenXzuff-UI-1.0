# RV64 U-mode trap entry — implementation contract

Status: design/implementation gate; no U-mode trap entry is claimed implemented.

## Verified repository baseline

- `kernel/src/trap.S` currently allocates a 288-byte frame directly from the interrupted `sp`. This is valid for the current S-mode-only smoke paths, but is not safe for a U-mode trap because it would use the user stack as kernel trap storage.
- `kernel/src/trap.rs` installs the direct-mode `stvec` vector and dispatches supervisor timer interrupts and S-mode breakpoints. It does not dispatch cause 8 (U-mode environment call).
- `kernel/src/runtime.rs` creates two S-mode tasks and restores their contexts through `sret`; its task frames have SPP set.
- `Kernel/arch/riscv64/trap_dispatch.cpp` is a host-testable policy layer, not currently linked into the Rust `no_std` kernel image.
- The C++ `TrapFrame` and Rust `TrapFrame` both currently describe the same 288-byte integer-register/CSR layout, but there is no cross-language ABI bridge.

## Required entry protocol

The U-mode path must not dereference or allocate on the untrusted user stack before switching to a supervisor-owned stack.

1. Maintain a per-hart supervisor trap-stack top. Before entering U-mode, write that top to `sscratch`.
2. At the first instructions of the trap vector, distinguish U-origin from S-origin using the saved privilege state (`sstatus.SPP`) and a carefully specified `sscratch` convention.
3. For a U-origin trap, atomically exchange `sp` with `sscratch`, so the first frame allocation is on the supervisor stack. Preserve the user `sp` in the frame; do not lose it during nested/interrupt handling.
4. Save all integer registers and `sepc`, `sstatus`, `scause`, and `stval` in the documented 288-byte frame. Keep 16-byte alignment before calling Rust.
5. Dispatch only `scause == 8` as U-mode `ecall`; advance `sepc` by exactly four bytes for a returning ECALL. Do not advance for a terminating syscall or a fault.
6. Before returning to U-mode, restore the user `sp`, restore the per-hart supervisor stack top into `sscratch`, restore the saved register set and execute `sret`.
7. Keep the existing S-mode timer-preemption path working. S-origin traps must not accidentally consume the U-mode `sscratch` stack pointer.

## Required tests before enabling the path

- Assembly-level U-to-S stack-switch test: prove the handler's frame lies inside the supervisor trap-stack range and the user stack pointer is unchanged in the saved frame.
- U-mode `ecall` test under QEMU: return a known value in `a0`, verify `sepc` advances by four, and verify execution resumes in U-mode.
- Register preservation test for all integer GPRs except architectural `x0`, plus stack alignment.
- Negative tests for non-ECALL U traps and S-mode ECALLs; they must not be misrouted to the user syscall dispatcher.
- Regression run of the existing S-mode breakpoint round-trip and two-task timer-preemption/context-switch smoke test.

## Integration boundary

Do not call the C++ dispatcher directly from Rust until a stable `no_std`-compatible shared syscall ABI is defined. First implement a Rust-native U-mode syscall dispatch interface with the same documented register contract (ID in `a7`, args in `a0..a5`, result in `a0`). A later C ABI bridge can reuse the host-tested C++ policy only after its hooks, error representation, and termination semantics are made compatible.

## Current decision

Do not patch `trap.S` by simply adding an `sscratch` swap to the existing S-mode frame path. That would break the current S-mode task context path or risk using an invalid stack on S-origin traps. Implement the privilege-aware entry/return split and its QEMU test as one reviewed change.
