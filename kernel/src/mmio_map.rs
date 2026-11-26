//! Pure Sv39 MMIO page-range planning shared by the kernel mapper and host tests.
//!
//! The first HAL mapping window deliberately covers physical addresses below
//! 1 GiB, where the initial RV64 board targets expose their UART and PLIC.

pub const PAGE_SIZE: u64 = 4096;
pub const LEVEL1_SPAN: u64 = 2 * 1024 * 1024;
pub const SV39_LOW_WINDOW_END: u64 = 1 << 38;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MmioMapPlanError {
    Empty,
    AddressOverflow,
    OutsideLowWindow,
    #[test]
    fn high_physical_resource_inside_sv39_low_half_is_supported() {
        let plan = plan(0x0ffd_8000_000, 0x0040_0000).unwrap();
        assert_eq!(plan.root_index, 3);
        assert_eq!(plan.page_start, 0x0ffd_8000_000);
        assert_eq!(plan.page_count, 1024);
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MmioMapPlan {
    pub root_index: usize,
    pub page_start: u64,
    pub page_end: u64,
    pub first_level1_index: usize,
    pub last_level1_index: usize,
    pub page_count: usize,
    pub table_count: usize,
}

/// Round a checked physical resource out to 4 KiB pages and identify the
/// 2 MiB Sv39 level-0 tables needed for an identity mapping.
pub fn plan(base: u64, size: u64) -> Result<MmioMapPlan, MmioMapPlanError> {
    if size == 0 {
        return Err(MmioMapPlanError::Empty);
    }
    let end = base.checked_add(size).ok_or(MmioMapPlanError::AddressOverflow)?;
    let page_start = base & !(PAGE_SIZE - 1);
    let rounded_end = end.checked_add(PAGE_SIZE - 1)
        .ok_or(MmioMapPlanError::AddressOverflow)?;
    let page_end = rounded_end & !(PAGE_SIZE - 1);

    if page_start >= SV39_LOW_WINDOW_END || page_end > SV39_LOW_WINDOW_END || page_start >= page_end {
        return Err(MmioMapPlanError::OutsideLowWindow);
    }

    let root_index = usize::try_from(page_start >> 30)
        .map_err(|_| MmioMapPlanError::OutsideLowWindow)?;
    let first_level1_index = usize::try_from(page_start / LEVEL1_SPAN)
        .map_err(|_| MmioMapPlanError::OutsideLowWindow)?;
    let last_level1_index = usize::try_from((page_end - 1) / LEVEL1_SPAN)
        .map_err(|_| MmioMapPlanError::OutsideLowWindow)?;
    let page_count = usize::try_from((page_end - page_start) / PAGE_SIZE)
        .map_err(|_| MmioMapPlanError::AddressOverflow)?;

    Ok(MmioMapPlan {
        root_index,
        page_start,
        page_end,
        first_level1_index,
        last_level1_index,
        page_count,
        table_count: last_level1_index - first_level1_index + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::{plan, MmioMapPlanError};

    #[test]
    fn qemu_virt_plic_uses_three_level0_tables() {
        let plan = plan(0x0c00_0000, 0x0060_0000).unwrap();
        assert_eq!(plan.page_start, 0x0c00_0000);
        assert_eq!(plan.page_end, 0x0c60_0000);
        assert_eq!(plan.first_level1_index, 96);
        assert_eq!(plan.last_level1_index, 98);
        assert_eq!(plan.page_count, 1536);
        assert_eq!(plan.table_count, 3);
    }

    #[test]
    fn qemu_virt_uart_uses_one_page() {
        let plan = plan(0x1000_0000, 0x100).unwrap();
        assert_eq!(plan.page_start, 0x1000_0000);
        assert_eq!(plan.page_end, 0x1000_1000);
        assert_eq!(plan.first_level1_index, 128);
        assert_eq!(plan.last_level1_index, 128);
        assert_eq!(plan.page_count, 1);
        assert_eq!(plan.table_count, 1);
    }

    #[test]
    fn unaligned_register_window_maps_only_its_covering_page() {
        let plan = plan(0x1000_0004, 0x100).unwrap();
        assert_eq!(plan.page_start, 0x1000_0000);
        assert_eq!(plan.page_end, 0x1000_1000);
        assert_eq!(plan.page_count, 1);
    }

    #[test]
    fn overflowing_resource_is_rejected() {
        assert_eq!(
            plan(u64::MAX - 0x7ff, 0x1000),
            Err(MmioMapPlanError::AddressOverflow),
        );
    }

    #[test]
    fn resource_outside_sv39_low_canonical_window_is_rejected() {
        assert_eq!(
            plan(0x4000_0000_0000, 0x1000),
            Err(MmioMapPlanError::OutsideLowWindow),
        );
    }
}
