//! User-space syscall ABI for Gravity Kernel on RV64.
 //! 
//! ABI: a7 = syscall number, a0-a5 = arguments, a0 = result.
//! These wrappers issue ECALL; the kernel may return -ENOSYS until the
//! corresponding scheduler/process service is implemented.
//!
//! This module is source-level ABI scaffolding, not proof that user mode,
//! address-space isolation, or task switching is already operational.

#![no_std]

pub const TASK_OPEN: usize = 0x0100;
pub const TASK_EXIT: usize = 0x0101;
pub const TASK_YIELD: usize = 0x0102;
pub const SYSTEM_SHUTDOWN: usize = 0x0103;
pub const SYSTEM_ON: usize = 0x0104;
pub const SYSTEM_REBOOT: usize = 0x0105;

/// Invoke a Gravity syscall. Only build this implementation for RV64.
#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub unsafe fn syscall6(
    number: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> isize {
    let mut result = a0;
    core::arch::asm!(
        "ecall",
        inlateout("a0") result,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
        in("a5") a5,
        in("a7") number,
        options(nostack)
    );
    result as isize
}

/// Host builds cannot issue a RISC-V ECALL; return -ENOSYS for compile/test use.
#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
pub unsafe fn syscall6(
    _number: usize,
    _a0: usize,
    _a1: usize,
    _a2: usize,
    _a3: usize,
    _a4: usize,
    _a5: usize,
) -> isize {
    -38
}

/// Request cooperative yield. Currently returns -ENOSYS until the kernel
/// trap-return path can schedule user contexts.
#[inline]
pub fn task_yield() -> isize {
    unsafe { syscall6(TASK_YIELD, 0, 0, 0, 0, 0, 0) }
}

/// Request task creation. The entry address is not yet accepted by the kernel:
/// callers must not assume this creates or runs a task until memory validation,
/// task context setup, and dispatcher integration are implemented.
#[inline]
pub fn task_open(entry: usize, argument: usize) -> isize {
    unsafe { syscall6(TASK_OPEN, entry, argument, 0, 0, 0, 0) }
}

/// Request platform power-on. This returns -ENOSYS until a supported power controller exists.\n#[inline]\npub fn system_on() -> isize {\n    unsafe { syscall6(SYSTEM_ON, 0, 0, 0, 0, 0, 0) }\n}\n\n/// Request a whole-system reboot. Kernel/firmware support is platform-dependent.\n#[inline]\npub fn system_reboot() -> isize {\n    unsafe { syscall6(SYSTEM_REBOOT, 0, 0, 0, 0, 0, 0) }\n}\n\n/// Request a whole-system shutdown. This must be restricted by kernel policy\n/// before general user applications are allowed to invoke it.\n#[inline]\npub fn system_shutdown() -> isize {\n    unsafe { syscall6(SYSTEM_SHUTDOWN, 0, 0, 0, 0, 0, 0) }\n}\n\n/// Request termination of the calling task with an exit status.
/// Current kernel behavior is a safe halt fallback, not per-task cleanup.
#[inline]
pub fn task_exit(status: isize) -> isize {
    unsafe { syscall6(TASK_EXIT, status as usize, 0, 0, 0, 0, 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syscall_numbers_match_kernel_abi() {
        assert_eq!(TASK_OPEN, 0x0100);
        assert_eq!(TASK_EXIT, 0x0101);
        assert_eq!(TASK_YIELD, 0x0102);\n        assert_eq!(SYSTEM_SHUTDOWN, 0x0103);\n        assert_eq!(SYSTEM_ON, 0x0104);\n        assert_eq!(SYSTEM_REBOOT, 0x0105);
    }

    #[cfg(not(target_arch = "riscv64"))]
    #[test]
    fn host_stub_fails_closed() {
        assert_eq!(unsafe { syscall6(TASK_YIELD, 0, 0, 0, 0, 0, 0) }, -38);
    }
}
