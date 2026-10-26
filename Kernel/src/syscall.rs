//! Gravity Kernel RV64 syscall dispatch core.
//!
//! This module is deliberately independent of trap-entry assembly and the scheduler.
//! The RV64 trap handler must pass a7 and a0..a5 here, then honor ProcessExited by
//! switching away from the terminated process. It must never return to that process.

pub const SYS_EXIT: usize = 0;
pub const SYS_SEND_MESSAGE: usize = 1;
pub const SYS_RECEIVE_MESSAGE: usize = 2;
pub const SYS_MAP_MEMORY: usize = 3;
pub const SYS_YIELD: usize = 4;

/// Gravity ABI v1 uses negative errno-style values for failures.
pub const ENOSYS: isize = -38;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyscallOutcome {
    /// The syscall returns normally; place this value in a0.
    Return(isize),
    /// The current process must be marked terminated and must not resume.
    ProcessExited { status: i32 },
}

/// Dispatch one syscall using the Gravity ABI v1 register convention.
///
/// `syscall_id` comes from a7. `args[0]` comes from a0.
/// Exit is represented as a control-flow outcome, not an ordinary return value:
/// the trap/scheduler integration must consume it and switch to another runnable
/// process (or enter the kernel's idle path). Do not execute sret for this process.
pub fn dispatch(syscall_id: usize, args: [usize; 6]) -> SyscallOutcome {
    match syscall_id {
        SYS_EXIT => SyscallOutcome::ProcessExited {
            status: args[0] as i32,
        },
        // These IDs are reserved by ABI v1 but their services are not implemented.
        SYS_SEND_MESSAGE | SYS_RECEIVE_MESSAGE | SYS_MAP_MEMORY | SYS_YIELD => {
            SyscallOutcome::Return(ENOSYS)
        }
        _ => SyscallOutcome::Return(ENOSYS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_zero_requests_successful_process_termination() {
        assert_eq!(
            dispatch(SYS_EXIT, [0, 0, 0, 0, 0, 0]),
            SyscallOutcome::ProcessExited { status: 0 }
        );
    }

    #[test]
    fn exit_preserves_nonzero_status_as_signed_i32() {
        assert_eq!(
            dispatch(SYS_EXIT, [7, 0, 0, 0, 0, 0]),
            SyscallOutcome::ProcessExited { status: 7 }
        );
        assert_eq!(
            dispatch(SYS_EXIT, [(-1isize) as usize, 0, 0, 0, 0, 0]),
            SyscallOutcome::ProcessExited { status: -1 }
        );
    }

    #[test]
    fn reserved_syscalls_are_not_misreported_as_implemented() {
        for id in [SYS_SEND_MESSAGE, SYS_RECEIVE_MESSAGE, SYS_MAP_MEMORY, SYS_YIELD, 99] {
            assert_eq!(
                dispatch(id, [0; 6]),
                SyscallOutcome::Return(ENOSYS),
                "syscall {id} must report ENOSYS until its handler exists"
            );
        }
    }
}
