//! Validated MMIO resource descriptors for the RV64 platform layer.
//!
//! This module validates FDT-derived physical ranges only. It deliberately does
//! not create a virtual mapping or expose a raw volatile pointer; the VM/HAL
//! mapping layer must establish device-memory attributes before any register IO.

use crate::fdt::FdtRegion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MmioError {
    UnknownSize,
    Empty,
    TooSmall,
    InvalidMinimumSize,
    InvalidAlignment,
    MisalignedBase,
    AddressOverflow,
    UnsupportedAccessWidth,
    MisalignedOffset,
    OutOfBounds,
}

/// A checked physical MMIO window. The end address is exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MmioRegion {
    base: u64,
    size: u64,
    end: u64,
    alignment: u64,
}

impl MmioRegion {
    /// Construct a descriptor from a translated FDT root-address resource.
    ///
    /// Callers must pass FdtMappedNode::root_reg, never the untranslated
    /// parent-bus reg. This validates arithmetic and basic alignment only;
    /// it does not authorize or perform an MMIO mapping.
    pub fn from_translated_fdt_region(
        region: FdtRegion,
        minimum_size: u64,
        alignment: u64,
    ) -> Result<Self, MmioError> {
        let size = region.size.ok_or(MmioError::UnknownSize)?;
        Self::new(region.bus_address, size, minimum_size, alignment)
    }

    pub fn new(
        base: u64,
        size: u64,
        minimum_size: u64,
        alignment: u64,
    ) -> Result<Self, MmioError> {
        if minimum_size == 0 {
            return Err(MmioError::InvalidMinimumSize);
        }
        if size == 0 {
            return Err(MmioError::Empty);
        }
        if size < minimum_size {
            return Err(MmioError::TooSmall);
        }
        if alignment == 0 || !alignment.is_power_of_two() {
            return Err(MmioError::InvalidAlignment);
        }
        if base & (alignment - 1) != 0 {
            return Err(MmioError::MisalignedBase);
        }
        let end = base.checked_add(size).ok_or(MmioError::AddressOverflow)?;
        Ok(Self { base, size, end, alignment })
    }

    pub const fn base(&self) -> u64 { self.base }
    pub const fn size(&self) -> u64 { self.size }
    pub const fn end(&self) -> u64 { self.end }
    pub const fn alignment(&self) -> u64 { self.alignment }

    /// Return a physical register address only when the complete access fits.
    /// Supported widths are 8, 16, 32 and 64 bits; no memory access is done.
    pub fn register_address(
        &self,
        offset: u64,
        access_width: u64,
    ) -> Result<u64, MmioError> {
        if !matches!(access_width, 1 | 2 | 4 | 8) {
            return Err(MmioError::UnsupportedAccessWidth);
        }
        // Establish the complete access is in-bounds before reporting alignment.
        // This also makes offsets near u64::MAX fail closed as OutOfBounds.
        let access_end = offset.checked_add(access_width).ok_or(MmioError::OutOfBounds)?;
        if access_end > self.size {
            return Err(MmioError::OutOfBounds);
        }
        if offset % access_width != 0 {
            return Err(MmioError::MisalignedOffset);
        }
        let address = self.base.checked_add(offset).ok_or(MmioError::AddressOverflow)?;
        if address % access_width != 0 {
            return Err(MmioError::MisalignedOffset);
        }
        Ok(address)
    }

    /// Check that a non-empty subrange is fully contained in this window.
    pub fn contains_range(&self, base: u64, size: u64) -> bool {
        if size == 0 { return false; }
        match base.checked_add(size) {
            Some(end) => base >= self.base && end <= self.end,
            None => false,
        }
    }
}
