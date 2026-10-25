pub const ENOSYS: isize = 38;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syscall {
    TaskCreate = 0x0100, TaskExit = 0x0101, TaskYield = 0x0102,
    MemAlloc = 0x0200, MemMap = 0x0201, MemUnmap = 0x0202,
    IpcSend = 0x0300, IpcReceive = 0x0301, IpcCall = 0x0302,
    FbInfo = 0x0400, FbMap = 0x0401, FbPresent = 0x0402,
    InputRead = 0x0500, InputWait = 0x0501,
    ClockNow = 0x0600, ClockSleep = 0x0601,
    CapGrant = 0x0700, CapRevoke = 0x0701,
}

/// Reserved ABI values remain fail-closed until a service has memory validation
/// and capability enforcement. a7 carries the number; a0-a5 carry arguments.
pub fn dispatch(_number: usize, _args: [usize; 6]) -> isize { -ENOSYS }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_syscall_fails_closed() { assert_eq!(dispatch(0x7fff, [0; 6]), -ENOSYS); }
    #[test]
    fn declared_ids_are_stable() {
        assert_eq!(Syscall::TaskCreate as usize, 0x0100);
        assert_eq!(Syscall::IpcCall as usize, 0x0302);
        assert_eq!(Syscall::CapRevoke as usize, 0x0701);
    }
}
