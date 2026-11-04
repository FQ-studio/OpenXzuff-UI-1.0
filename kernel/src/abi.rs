//! Gravity Kernel ABI v1 — stable RV64 userspace contract primitives.
//! CPU-level register/trap rules follow the RISC-V RV64 psABI and Privileged ISA.
//! This module defines only the OS-level wire contract; it does not pretend that
//! unimplemented kernel services are available.
pub const ABI_VERSION_MAJOR: u16 = 1;
pub const ABI_VERSION_MINOR: u16 = 0;
pub const PAGE_SIZE: usize = 4096;
pub const USER_VA_END: u64 = 1u64 << 38; // Sv39 lower canonical half; exclusive.

pub mod errno {
    pub const EPERM: isize = 1;
    pub const ENOENT: isize = 2;
    pub const EIO: isize = 5;
    pub const E2BIG: isize = 7;
    pub const EBADF: isize = 9;
    pub const EAGAIN: isize = 11;
    pub const ENOMEM: isize = 12;
    pub const EACCES: isize = 13;
    pub const EFAULT: isize = 14;
    pub const EBUSY: isize = 16;
    pub const EEXIST: isize = 17;
    pub const EINVAL: isize = 22;
    pub const ENOSPC: isize = 28;
    pub const ENOSYS: isize = 38;
    pub const ENOTSUP: isize = 95;
    pub const ETIMEDOUT: isize = 110;
    pub const ECANCELED: isize = 125;
}

/// Stable syscall numbers. Never renumber an assigned value in ABI v1.
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syscall {
    AbiInfo = 0x0000,
    TaskOpen = 0x0100, TaskExit = 0x0101, TaskYield = 0x0102,
    SystemShutdown = 0x0103, SystemOn = 0x0104, SystemReboot = 0x0105,
    MemAlloc = 0x0200, MemMap = 0x0201, MemUnmap = 0x0202,
    IpcSend = 0x0300, IpcReceive = 0x0301, IpcCall = 0x0302,
    FbInfo = 0x0400, FbMap = 0x0401, FbPresent = 0x0402,
    InputRead = 0x0500, InputWait = 0x0501,
    ClockNow = 0x0600, ClockSleep = 0x0601,
    CapGrant = 0x0700, CapRevoke = 0x0701,
}

pub const SYSCALL_REGISTER: &str = "a7";
pub const ARGUMENT_REGISTERS: [&str; 6] = ["a0", "a1", "a2", "a3", "a4", "a5"];
pub const RETURN_REGISTER: &str = "a0";
pub const SECOND_RETURN_REGISTER: &str = "a1";

/// Syscall result encoding: non-negative values are success; negative values
/// are -errno. This matches the conventional RISC-V Unix-style result shape.
pub const fn encode_error(error: isize) -> isize { -error }

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AbiInfo {
    pub major: u16,
    pub minor: u16,
    pub xlen: u16,
    pub page_size: u16,
    pub features: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskHandle {
    pub id: u32,
    pub generation: u32,
}
impl TaskHandle {
    pub const INVALID: Self = Self { id: 0, generation: 0 };
    pub const fn is_valid(self) -> bool { self.id != 0 && self.generation != 0 }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UserRange {
    pub address: u64,
    pub length: u64,
}
impl UserRange {
    /// Validate a non-empty, non-overflowing range in Sv39 lower-half user VA.
    /// This checks address shape only; the VM must still verify mapped pages and
    /// permissions before dereferencing it.
    pub fn checked(address: u64, length: u64) -> Result<Self, isize> {
        if length == 0 { return Err(errno::EINVAL); }
        let end = address.checked_add(length).ok_or(errno::EFAULT)?;
        if address >= USER_VA_END || end > USER_VA_END { return Err(errno::EFAULT); }
        Ok(Self { address, length })
    }
    pub const fn end(self) -> u64 { self.address + self.length }
}

pub mod mem_flags {
    pub const READ: u64 = 1 << 0;
    pub const WRITE: u64 = 1 << 1;
    pub const EXEC: u64 = 1 << 2;
    pub const USER: u64 = 1 << 3;
    pub const SHARED: u64 = 1 << 4;
    pub const PRIVATE: u64 = 1 << 5;
    pub const KNOWN: u64 = READ | WRITE | EXEC | USER | SHARED | PRIVATE;
    pub const fn valid(flags: u64) -> bool {
        flags != 0 && flags & !KNOWN == 0 &&
        !(flags & SHARED != 0 && flags & PRIVATE != 0) &&
        !(flags & (WRITE | EXEC) == (WRITE | EXEC))
    }
}

pub mod cap_rights {
    pub const TASK_CONTROL: u64 = 1 << 0;
    pub const MEMORY: u64 = 1 << 1;
    pub const IPC_SEND: u64 = 1 << 2;
    pub const IPC_RECEIVE: u64 = 1 << 3;
    pub const DISPLAY: u64 = 1 << 4;
    pub const INPUT: u64 = 1 << 5;
    pub const CLOCK: u64 = 1 << 6;
    pub const POWER: u64 = 1 << 7;
    pub const KNOWN: u64 = (1 << 8) - 1;
    pub const fn valid(rights: u64) -> bool { rights != 0 && rights & !KNOWN == 0 }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IpcMessage {
    pub sender: TaskHandle,
    pub tag: u64,
    pub words: [u64; 6],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockTime {
    pub seconds: u64,
    pub nanoseconds: u32,
}
impl ClockTime {
    pub const fn is_normalized(self) -> bool { self.nanoseconds < 1_000_000_000 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn syscall_numbers_are_stable() {
        assert_eq!(Syscall::AbiInfo as usize, 0);
        assert_eq!(Syscall::TaskOpen as usize, 0x0100);
        assert_eq!(Syscall::MemMap as usize, 0x0201);
        assert_eq!(Syscall::IpcCall as usize, 0x0302);
        assert_eq!(Syscall::CapRevoke as usize, 0x0701);
    }
    #[test]
    fn user_ranges_reject_null_overflow_and_kernel_half() {
        assert_eq!(UserRange::checked(0, 8), Ok(UserRange { address: 0, length: 8 }));
        assert_eq!(UserRange::checked(1, 0), Err(errno::EINVAL));
        assert_eq!(UserRange::checked(u64::MAX - 3, 8), Err(errno::EFAULT));
        assert_eq!(UserRange::checked(USER_VA_END - 4, 8), Err(errno::EFAULT));
        assert_eq!(UserRange::checked(USER_VA_END - 8, 8).unwrap().end(), USER_VA_END);
    }
    #[test]
    fn memory_flags_are_well_formed() {
        assert!(mem_flags::valid(mem_flags::READ | mem_flags::USER));
        assert!(!mem_flags::valid(mem_flags::WRITE | mem_flags::EXEC));
        assert!(!mem_flags::valid(mem_flags::SHARED | mem_flags::PRIVATE));
        assert!(!mem_flags::valid(1 << 63));
    }
    #[test]
    fn capability_rights_reject_unknown_bits() {
        assert!(cap_rights::valid(cap_rights::IPC_SEND | cap_rights::IPC_RECEIVE));
        assert!(!cap_rights::valid(0));
        assert!(!cap_rights::valid(1 << 63));
    }
    #[test]
    fn clock_time_requires_normalized_nanoseconds() {
        assert!(ClockTime { seconds: 1, nanoseconds: 999_999_999 }.is_normalized());
        assert!(!ClockTime { seconds: 1, nanoseconds: 1_000_000_000 }.is_normalized());
    }
}
