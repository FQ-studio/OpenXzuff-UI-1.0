//! Checked register geometry for the RISC-V Platform-Level Interrupt Controller.
//!
//! This module validates the standard PLIC register windows against an already
//! translated MMIO resource. It does not map the resource, perform volatile IO,
//! discover FDT interrupt contexts, or enable/claim interrupts. The caller must
//! obtain source and context counts from validated platform firmware metadata.

use crate::mmio::{MmioError, MmioRegion};

const PRIORITY_BASE: u64 = 0x000000;
const PENDING_BASE: u64 = 0x001000;
const ENABLE_BASE: u64 = 0x002000;
const ENABLE_CONTEXT_STRIDE: u64 = 0x80;
const CONTEXT_BASE: u64 = 0x200000;
const CONTEXT_STRIDE: u64 = 0x1000;
const MAX_SOURCE_ID: u32 = 1023;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlicError {
    InvalidSourceCount,
    InvalidContextCount,
    InvalidSourceId,
    InvalidContextId,
    AddressOverflow,
    RegionTooSmall,
    Mmio(MmioError),
}

/// Address and bit mask for a PLIC pending/enable word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlicBitRegister {
    pub address: u64,
    pub mask: u32,
}

/// Validated standard PLIC register layout for one translated MMIO resource.
///
/// `source_count` is the highest valid interrupt source ID (1..=1023), as
/// represented by the RISC-V PLIC `riscv,ndev` property. `context_count` is
/// the number of entries in the controller's resolved `interrupts-extended`
/// context list. Neither value is guessed by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlicLayout {
    region: MmioRegion,
    source_count: u32,
    context_count: u32,
}

impl PlicLayout {
    pub fn new(
        region: MmioRegion,
        source_count: u32,
        context_count: u32,
    ) -> Result<Self, PlicError> {
        if source_count == 0 || source_count > MAX_SOURCE_ID {
            return Err(PlicError::InvalidSourceCount);
        }
        if context_count == 0 {
            return Err(PlicError::InvalidContextCount);
        }

        // IDs 0..=ndev occupy the priority and enable/pending bitmaps.
        let words = (source_count as u64 + 32) / 32;
        let priority_end = (source_count as u64 + 1)
            .checked_mul(4)
            .ok_or(PlicError::AddressOverflow)?;
        let pending_end = PENDING_BASE
            .checked_add(words.checked_mul(4).ok_or(PlicError::AddressOverflow)?)
            .ok_or(PlicError::AddressOverflow)?;
        let last_context = (context_count as u64 - 1)
            .checked_mul(1)
            .ok_or(PlicError::AddressOverflow)?;
        let enable_start = ENABLE_BASE
            .checked_add(last_context.checked_mul(ENABLE_CONTEXT_STRIDE)
                .ok_or(PlicError::AddressOverflow)?)
            .ok_or(PlicError::AddressOverflow)?;
        let enable_end = enable_start
            .checked_add(words.checked_mul(4).ok_or(PlicError::AddressOverflow)?)
            .ok_or(PlicError::AddressOverflow)?;
        let context_start = CONTEXT_BASE
            .checked_add(last_context.checked_mul(CONTEXT_STRIDE)
                .ok_or(PlicError::AddressOverflow)?)
            .ok_or(PlicError::AddressOverflow)?;
        let context_end = context_start.checked_add(8).ok_or(PlicError::AddressOverflow)?;

        Self::require_window(&region, PRIORITY_BASE, priority_end)?;
        Self::require_window(&region, PENDING_BASE, pending_end)?;
        Self::require_window(&region, enable_start, enable_end)?;
        Self::require_window(&region, context_start, context_end)?;

        Ok(Self { region, source_count, context_count })
    }

    fn require_window(region: &MmioRegion, start: u64, end: u64) -> Result<(), PlicError> {
        if start >= end {
            return Err(PlicError::AddressOverflow);
        }
        let absolute_start = region.base()
            .checked_add(start)
            .ok_or(PlicError::AddressOverflow)?;
        if !region.contains_range(absolute_start, end - start) {
            return Err(PlicError::RegionTooSmall);
        }
        region.register_address(start, 4).map_err(PlicError::Mmio)?;
        region.register_address(end - 4, 4).map_err(PlicError::Mmio)?;
        Ok(())
    }

    fn register(&self, offset: u64) -> Result<u64, PlicError> {
        self.region.register_address(offset, 4).map_err(PlicError::Mmio)
    }

    fn check_source(&self, source_id: u32) -> Result<u64, PlicError> {
        if source_id == 0 || source_id > self.source_count {
            return Err(PlicError::InvalidSourceId);
        }
        Ok(source_id as u64)
    }

    fn check_context(&self, context: u32) -> Result<u64, PlicError> {
        if context >= self.context_count {
            return Err(PlicError::InvalidContextId);
        }
        Ok(context as u64)
    }

    /// Priority register for a non-zero interrupt source ID.
    pub fn priority_address(&self, source_id: u32) -> Result<u64, PlicError> {
        let source = self.check_source(source_id)?;
        self.register(PRIORITY_BASE + source.checked_mul(4).ok_or(PlicError::AddressOverflow)?)
    }

    /// Pending bitmap word and bit for a non-zero interrupt source ID.
    pub fn pending_bit(&self, source_id: u32) -> Result<PlicBitRegister, PlicError> {
        let source = self.check_source(source_id)?;
        let word_offset = (source / 32).checked_mul(4).ok_or(PlicError::AddressOverflow)?;
        Ok(PlicBitRegister {
            address: self.register(PENDING_BASE.checked_add(word_offset)
                .ok_or(PlicError::AddressOverflow)?)?,
            mask: 1u32 << (source % 32),
        })
    }

    /// Per-context enable bitmap word and bit for an interrupt source.
    pub fn enable_bit(
        &self,
        context: u32,
        source_id: u32,
    ) -> Result<PlicBitRegister, PlicError> {
        let context = self.check_context(context)?;
        let source = self.check_source(source_id)?;
        let context_offset = context.checked_mul(ENABLE_CONTEXT_STRIDE)
            .ok_or(PlicError::AddressOverflow)?;
        let word_offset = (source / 32).checked_mul(4).ok_or(PlicError::AddressOverflow)?;
        let offset = ENABLE_BASE.checked_add(context_offset)
            .and_then(|value| value.checked_add(word_offset))
            .ok_or(PlicError::AddressOverflow)?;
        Ok(PlicBitRegister {
            address: self.register(offset)?,
            mask: 1u32 << (source % 32),
        })
    }

    pub fn threshold_address(&self, context: u32) -> Result<u64, PlicError> {
        let context = self.check_context(context)?;
        let offset = CONTEXT_BASE.checked_add(
            context.checked_mul(CONTEXT_STRIDE).ok_or(PlicError::AddressOverflow)?
        ).ok_or(PlicError::AddressOverflow)?;
        self.register(offset)
    }

    pub fn claim_complete_address(&self, context: u32) -> Result<u64, PlicError> {
        let threshold = self.threshold_address(context)?;
        threshold.checked_add(4).ok_or(PlicError::AddressOverflow)
    }

    pub const fn source_count(&self) -> u32 { self.source_count }
    pub const fn context_count(&self) -> u32 { self.context_count }
}

#[cfg(test)]
mod tests {
    use super::{PlicError, PlicLayout};
    use crate::mmio::MmioRegion;

    fn qemu_virt_plic() -> PlicLayout {
        // QEMU virt exposes a 6 MiB PLIC window; the FDT supplies ndev=95.
        // This fixture supplies two resolved contexts for geometry testing.
        let region = MmioRegion::new(0x0c00_0000, 0x0060_0000, 8, 4).unwrap();
        PlicLayout::new(region, 95, 2).unwrap()
    }

    #[test]
    fn plic_layout_calculates_qemu_virt_register_addresses() {
        let plic = qemu_virt_plic();
        assert_eq!(plic.priority_address(1), Ok(0x0c00_0004));
        assert_eq!(plic.priority_address(95), Ok(0x0c00_017c));
        assert_eq!(plic.pending_bit(32), Ok(super::PlicBitRegister {
            address: 0x0c00_1004, mask: 1,
        }));
        assert_eq!(plic.enable_bit(1, 95), Ok(super::PlicBitRegister {
            address: 0x0c00_2088, mask: 0x8000_0000,
        }));
        assert_eq!(plic.threshold_address(1), Ok(0x0c20_1000));
        assert_eq!(plic.claim_complete_address(1), Ok(0x0c20_1004));
        assert_eq!(plic.source_count(), 95);
        assert_eq!(plic.context_count(), 2);
    }

    #[test]
    fn plic_layout_rejects_invalid_source_and_context_counts() {
        let region = MmioRegion::new(0x0c00_0000, 0x0040_0000, 8, 4).unwrap();
        assert_eq!(PlicLayout::new(region, 0, 2), Err(PlicError::InvalidSourceCount));
        assert_eq!(PlicLayout::new(region, 1024, 2), Err(PlicError::InvalidSourceCount));
        assert_eq!(PlicLayout::new(region, 96, 0), Err(PlicError::InvalidContextCount));
    }

    #[test]
    fn plic_layout_rejects_out_of_range_source_and_context_ids() {
        let plic = qemu_virt_plic();
        assert_eq!(plic.priority_address(0), Err(PlicError::InvalidSourceId));
        assert_eq!(plic.priority_address(96), Err(PlicError::InvalidSourceId));
        assert_eq!(plic.pending_bit(0), Err(PlicError::InvalidSourceId));
        assert_eq!(plic.enable_bit(2, 1), Err(PlicError::InvalidContextId));
        assert_eq!(plic.threshold_address(2), Err(PlicError::InvalidContextId));
    }

    #[test]
    fn plic_layout_rejects_resource_smaller_than_last_context_window() {
        let region = MmioRegion::new(0x0c00_0000, 0x0020_1007, 8, 4).unwrap();
        assert_eq!(PlicLayout::new(region, 95, 2), Err(PlicError::RegionTooSmall));
    }

    #[test]
    fn plic_layout_rejects_absolute_context_address_overflow() {
        // Earlier register windows fit below u64::MAX, but the context block
        // starts beyond the remaining physical address space.
        let base = u64::MAX - 0x3003;
        let region = MmioRegion::new(base, 0x3003, 8, 4).unwrap();
        assert_eq!(
            PlicLayout::new(region, 96, 2),
            Err(PlicError::AddressOverflow),
        );
    }
}
