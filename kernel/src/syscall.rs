pub const ENOSYS: isize = 38;
pub const EINVAL: isize = 22;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syscall {
    TaskOpen = 0x0100, TaskExit = 0x0101, TaskYield = 0x0102,
    MemAlloc = 0x0200, MemMap = 0x0201, MemUnmap = 0x0202,
    IpcSend = 0x0300, IpcReceive = 0x0301, IpcCall = 0x0302,
    FbInfo = 0x0400, FbMap = 0x0401, FbPresent = 0x0402,
    InputRead = 0x0500, InputWait = 0x0501,
    ClockNow = 0x0600, ClockSleep = 0x0601,
    CapGrant = 0x0700, CapRevoke = 0x0701,
}

/// Result of decoding a syscall. Exit is distinct from a normal return because
/// it must eventually be consumed by the process manager/scheduler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyscallOutcome {
    Return(isize),
    ExitRequested { status: isize },
}

/// RV64 syscall ABI: a7 = number, a0-a5 = arguments, a0 = return value.
/// Services other than TaskExit remain fail-closed until their backing
/// subsystem, memory validation and capability checks are implemented.
pub fn dispatch(number: usize, args: [usize; 6]) -> SyscallOutcome {
    match number {
        x if x == Syscall::TaskExit as usize => {
            SyscallOutcome::ExitRequested { status: args[0] as isize }
        }
        _ => SyscallOutcome::Return(-ENOSYS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_syscall_fails_closed() {
        assert_eq!(dispatch(0x7fff, [0; 6]), SyscallOutcome::Return(-ENOSYS));
    }

    #[test]
    fn declared_ids_are_stable() {
        assert_eq!(Syscall::TaskOpen as usize, 0x0100);
        assert_eq!(Syscall::TaskExit as usize, 0x0101);
        assert_eq!(Syscall::IpcCall as usize, 0x0302);
        assert_eq!(Syscall::CapRevoke as usize, 0x0701);
    }

    #[test]
    fn task_exit_captures_status_without_treating_it_as_return() {
        assert_eq!(
            dispatch(Syscall::TaskExit as usize, [23, 0, 0, 0, 0, 0]),
            SyscallOutcome::ExitRequested { status: 23 }
        );
    }

    #[test]
    fn unimplemented_services_return_enosys() {
        for number in [
            Syscall::TaskOpen as usize, Syscall::TaskYield as usize,
            Syscall::MemAlloc as usize, Syscall::MemMap as usize,
            Syscall::MemUnmap as usize, Syscall::IpcSend as usize,
            Syscall::IpcReceive as usize, Syscall::IpcCall as usize,
            Syscall::FbInfo as usize, Syscall::FbMap as usize,
            Syscall::FbPresent as usize, Syscall::InputRead as usize,
            Syscall::InputWait as usize, Syscall::ClockNow as usize,
            Syscall::ClockSleep as usize, Syscall::CapGrant as usize,
            Syscall::CapRevoke as usize,
        ] {
            assert_eq!(dispatch(number, [0; 6]), SyscallOutcome::Return(-ENOSYS));
        }
    }
}
