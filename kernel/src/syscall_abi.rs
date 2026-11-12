//! Stable RV64 U-mode ecall ABI shared with the Win32 Gaming adapter.
#![allow(dead_code)]

pub const EXIT: usize = 0;
pub const SEND_MESSAGE: usize = 1;
pub const RECEIVE_MESSAGE: usize = 2;
pub const MAP_MEMORY: usize = 3;
pub const YIELD: usize = 4;

pub const EAGAIN: isize = -11;
pub const EFAULT: isize = -14;
pub const ENOMEM: isize = -12;
pub const ENOSYS: isize = -38;
pub const EINVAL: isize = -22;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syscall_numbers_match_the_public_v1_contract() {
        assert_eq!([EXIT, SEND_MESSAGE, RECEIVE_MESSAGE, MAP_MEMORY, YIELD],
                   [0, 1, 2, 3, 4]);
    }

    #[test]
    fn errno_values_match_the_c_host_abi() {
        assert_eq!([EAGAIN, EFAULT, ENOMEM, ENOSYS, EINVAL],
                   [-11, -14, -12, -38, -22]);
    }
}
