//! Pure Sv39 user-memory policy shared by the kernel and host tests.
pub const SV39_WINDOW_SIZE: usize = 1 << 30;
pub const USER_HEAP_OFFSET: usize = 0x1000_0000;
pub const USER_HEAP_SIZE: usize = 2 * 1024 * 1024;
pub const PROT_READ: u32 = 1;
pub const PROT_WRITE: u32 = 2;
pub const PROT_EXECUTE: u32 = 4;
pub const PROT_USER: u32 = 8;
const PTE_V: u64 = 1 << 0;
const PTE_R: u64 = 1 << 1;
const PTE_W: u64 = 1 << 2;
const PTE_X: u64 = 1 << 3;
const PTE_U: u64 = 1 << 4;
const PTE_A: u64 = 1 << 6;
const PTE_D: u64 = 1 << 7;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectionError { UnknownBits, UserBitRequired, NoAccess, WriteWithoutRead, WriteExecute }
pub fn user_leaf_flags(protection: u32) -> Result<u64, ProtectionError> {
    let allowed = PROT_READ | PROT_WRITE | PROT_EXECUTE | PROT_USER;
    if protection & !allowed != 0 { return Err(ProtectionError::UnknownBits); }
    if protection & PROT_USER == 0 { return Err(ProtectionError::UserBitRequired); }
    if protection & (PROT_READ | PROT_WRITE | PROT_EXECUTE) == 0 { return Err(ProtectionError::NoAccess); }
    if protection & PROT_WRITE != 0 && protection & PROT_READ == 0 { return Err(ProtectionError::WriteWithoutRead); }
    if protection & PROT_WRITE != 0 && protection & PROT_EXECUTE != 0 { return Err(ProtectionError::WriteExecute); }
    let mut flags = PTE_V | PTE_U | PTE_A;
    if protection & PROT_READ != 0 { flags |= PTE_R; }
    if protection & PROT_WRITE != 0 { flags |= PTE_W | PTE_D; }
    if protection & PROT_EXECUTE != 0 { flags |= PTE_X; }
    Ok(flags)
}
pub const fn requests_write_execute(protection: u32) -> bool {
    protection & (PROT_WRITE | PROT_EXECUTE) == (PROT_WRITE | PROT_EXECUTE)
}
pub fn user_heap_base(window_base: usize) -> Option<usize> {
    if window_base & (SV39_WINDOW_SIZE - 1) != 0 { return None; }
    let heap_base = window_base.checked_add(USER_HEAP_OFFSET)?;
    let heap_end = heap_base.checked_add(USER_HEAP_SIZE)?;
    let window_end = window_base.checked_add(SV39_WINDOW_SIZE)?;
    if heap_base < window_base || heap_end > window_end { return None; }
    Some(heap_base)
}
pub const fn l1_region_index(address: usize) -> usize { (address >> 21) & 0x1ff }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heap_base_tracks_the_bootstrap_window() {
        assert_eq!(user_heap_base(0x8000_0000), Some(0x9000_0000));
        assert_eq!(user_heap_base(0x4000_0000), Some(0x5000_0000));
        assert_eq!(user_heap_base(0x8000_0001), None);
    }
    #[test]
    fn heap_base_rejects_address_overflow() {
        assert_eq!(user_heap_base(usize::MAX & !(SV39_WINDOW_SIZE - 1)), None);
    }
    #[test]
    fn user_protection_allows_r_rw_or_x_and_rejects_wx() {
        assert!(user_leaf_flags(PROT_READ | PROT_USER).is_ok());
        assert!(user_leaf_flags(PROT_READ | PROT_WRITE | PROT_USER).is_ok());
        assert!(user_leaf_flags(PROT_EXECUTE | PROT_USER).is_ok());
        assert_eq!(user_leaf_flags(PROT_READ | PROT_WRITE | PROT_EXECUTE | PROT_USER), Err(ProtectionError::WriteExecute));
    }
    #[test]
    fn user_protection_rejects_invalid_bits_and_access_modes() {
        assert_eq!(user_leaf_flags(0), Err(ProtectionError::UserBitRequired));
        assert_eq!(user_leaf_flags(PROT_USER), Err(ProtectionError::NoAccess));
        assert_eq!(user_leaf_flags(PROT_WRITE | PROT_USER), Err(ProtectionError::WriteWithoutRead));
        assert_eq!(user_leaf_flags(PROT_READ | 0x10 | PROT_USER), Err(ProtectionError::UnknownBits));
    }
}
