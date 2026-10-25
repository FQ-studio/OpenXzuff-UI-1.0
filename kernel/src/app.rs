pub const HEADER_SIZE: usize = 48;
pub const VERSION: u16 = 1;
pub const ARCH_RV64: u32 = 0x3436_5652;
pub const FLAG_NONE: u32 = 0;
pub const EINVAL: isize = 22;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppInfo {
    pub architecture: u32,
    pub flags: u32,
    pub entry_offset: u64,
    pub image_size: u64,
    pub memory_size: u64,
}

fn le16(p: &[u8], n: usize) -> u16 { u16::from_le_bytes([p[n], p[n + 1]]) }
fn le32(p: &[u8], n: usize) -> u32 { u32::from_le_bytes([p[n], p[n + 1], p[n + 2], p[n + 3]]) }
fn le64(p: &[u8], n: usize) -> u64 { u64::from_le_bytes([p[n], p[n+1], p[n+2], p[n+3], p[n+4], p[n+5], p[n+6], p[n+7]]) }

/// Validate GAPP v1 metadata only. This never maps or executes the image.
pub fn validate(file: &[u8]) -> Result<AppInfo, isize> {
    if file.len() < HEADER_SIZE || &file[..4] != b"GAPP"
        || le16(file, 4) != VERSION || le16(file, 6) as usize != HEADER_SIZE {
        return Err(-EINVAL);
    }
    let info = AppInfo {
        architecture: le32(file, 8),
        flags: le32(file, 12),
        entry_offset: le64(file, 16),
        image_size: le64(file, 24),
        memory_size: le64(file, 32),
    };
    if le64(file, 40) != 0 || info.architecture != ARCH_RV64 || info.flags != FLAG_NONE
        || info.image_size == 0 || info.image_size > (file.len() - HEADER_SIZE) as u64
        || info.entry_offset >= info.image_size || info.memory_size < info.image_size {
        return Err(-EINVAL);
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn valid() -> [u8; HEADER_SIZE + 4] {
        let mut b = [0u8; HEADER_SIZE + 4];
        b[..4].copy_from_slice(b"GAPP");
        b[4..6].copy_from_slice(&VERSION.to_le_bytes());
        b[6..8].copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        b[8..12].copy_from_slice(&ARCH_RV64.to_le_bytes());
        b[24..32].copy_from_slice(&4u64.to_le_bytes());
        b[32..40].copy_from_slice(&8u64.to_le_bytes());
        b[HEADER_SIZE..].copy_from_slice(b"CODE");
        b
    }
    #[test]
    fn accepts_valid_rv64_package_metadata() {
        let b = valid();
        let info = validate(&b).unwrap();
        assert_eq!(info.architecture, ARCH_RV64);
        assert_eq!((info.entry_offset, info.image_size, info.memory_size), (0, 4, 8));
    }
    #[test]
    fn rejects_bad_magic_version_architecture_entry_size_and_reserved_fields() {
        let mut b = valid(); b[0] = b'X'; assert_eq!(validate(&b), Err(-EINVAL));
        let mut b = valid(); b[4] = 99; assert_eq!(validate(&b), Err(-EINVAL));
        let mut b = valid(); b[8] = 0; assert_eq!(validate(&b), Err(-EINVAL));
        let mut b = valid(); b[16] = 4; assert_eq!(validate(&b), Err(-EINVAL));
        let mut b = valid(); b[24..32].copy_from_slice(&u64::MAX.to_le_bytes()); assert_eq!(validate(&b), Err(-EINVAL));
        let mut b = valid(); b[32..40].copy_from_slice(&3u64.to_le_bytes()); assert_eq!(validate(&b), Err(-EINVAL));
        let mut b = valid(); b[40] = 1; assert_eq!(validate(&b), Err(-EINVAL));
        assert_eq!(validate(&[0; HEADER_SIZE - 1]), Err(-EINVAL));
    }
}
