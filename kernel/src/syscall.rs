pub use crate::abi::{Syscall, errno::ENOSYS};
use crate::abi::{encode_error, errno};

/// Result of decoding a syscall. Exit/control requests are distinct from normal
/// returns because they must be consumed by the process/platform manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyscallOutcome {
    Return(isize),
    ExitRequested { status: isize },
    ShutdownRequested,
    RebootRequested,
}

/// Gravity ABI v1: a7 = syscall number; a0-a5 = six arguments;
/// a0 = result (-errno on failure). Unavailable services fail closed.
pub fn dispatch(number: usize, args: [usize; 6]) -> SyscallOutcome {
    match number {
        x if x == Syscall::TaskExit as usize =>
            SyscallOutcome::ExitRequested { status: args[0] as isize },
        x if x == Syscall::SystemShutdown as usize => SyscallOutcome::ShutdownRequested,
        x if x == Syscall::SystemReboot as usize => SyscallOutcome::RebootRequested,
        _ => SyscallOutcome::Return(encode_error(errno::ENOSYS)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_and_unimplemented_syscalls_fail_closed() {
        assert_eq!(dispatch(0x7fff, [0; 6]), SyscallOutcome::Return(-ENOSYS));
        for number in [
            Syscall::AbiInfo as usize, Syscall::TaskOpen as usize, Syscall::TaskYield as usize,
            Syscall::SystemOn as usize, Syscall::MemAlloc as usize, Syscall::MemMap as usize,
            Syscall::MemUnmap as usize, Syscall::IpcSend as usize, Syscall::IpcReceive as usize,
            Syscall::IpcCall as usize, Syscall::FbInfo as usize, Syscall::FbMap as usize,
            Syscall::FbPresent as usize, Syscall::InputRead as usize, Syscall::InputWait as usize,
            Syscall::ClockNow as usize, Syscall::ClockSleep as usize,
            Syscall::CapGrant as usize, Syscall::CapRevoke as usize,
        ] {
            assert_eq!(dispatch(number, [0; 6]), SyscallOutcome::Return(-ENOSYS));
        }
    }
    #[test]
    fn task_exit_preserves_exit_status() {
        assert_eq!(dispatch(Syscall::TaskExit as usize, [23, 0, 0, 0, 0, 0]),
            SyscallOutcome::ExitRequested { status: 23 });
    }
    #[test]
    fn power_requests_are_distinct() {
        assert_eq!(dispatch(Syscall::SystemReboot as usize, [0; 6]), SyscallOutcome::RebootRequested);
        assert_eq!(dispatch(Syscall::SystemShutdown as usize, [0; 6]), SyscallOutcome::ShutdownRequested);
    }
}
