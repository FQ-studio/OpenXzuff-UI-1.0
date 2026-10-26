# Gravity Kernel RV64 — Exit syscall

Status: **Rust dispatch core implemented; not yet wired to a bootable RV64 trap path.**

## ABI v1

- syscall ID: `0` (`Exit`)
- `a7`: syscall ID
- `a0`: process exit status, interpreted as signed 32-bit
- instruction: `ecall`
- successful dispatch result: `ProcessExited { status }`, not a normal return value

Minimal user-side request:

```asm
addi a0, zero, 0    # status = success
addi a7, zero, 0    # Gravity Exit = 0
ecall
```

## Implementation

`Kernel/src/syscall.rs` implements the ABI dispatch core. It returns an explicit
`ProcessExited` control-flow outcome for Exit. Reserved syscall IDs currently return
`-38` (`ENOSYS`) so that definitions are not mistaken for working services.

## Required integration before this is a real running syscall

1. RV64 trap entry saves the required registers and reads `a7` plus `a0..a5`.
2. Trap dispatch calls `dispatch`.
3. On `ProcessExited`, the process manager records the status and marks the current
   process terminated.
4. The scheduler switches to another runnable process or the idle path. It must **not**
   restore the exited process and execute `sret` back to it.
5. Add a QEMU RV64 test that executes `ecall` from user mode and verifies the exit status.

These integration pieces are not claimed to exist yet. The host-side Rust tests verify
syscall selection and exit-status handling only; they do not prove privilege transitions,
process cleanup, scheduling, or physical-board boot.
