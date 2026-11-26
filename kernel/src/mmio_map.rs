//! Pure Sv39 MMIO page-range planning shared by the kernel mapper and host tests.
//!
//! Identity-mapped device resources are supported throughout the lower
//! canonical Sv39 virtual-address half (0..2^38), subject to the kernel's
//! bounded intermediate-page-table pool.

pub const PAGE_SIZE: u64 = 4096;
pub const LEVEL1_SPAN: u64 = 2 * 1024 * 1024;
pub const SV39_LOW_WINDOW_END: u64 = 1 << 38;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MmioMapPlanError {
    Empty,
    AddressOverflow,
    OutsideLowWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MmioMapPlan {
    /// Sv39 root-page-table slot (VPN[2]).
    pub root_index: usize,
    pub page_start: u64,
    pub page_end: u64,
    /// Global 2 MiB table indexes, useful for diagnostics and host tests.
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
        .ok_or(MmioMapPlanError::AddressOverflow)? & !(PAGE_SIZE - 1);

    if page_start >= SV39_LOW_WINDOW_END
        || rounded_end > SV39_LOW_WINDOW_END
        || page_start >= rounded_end
    {
        return Err(MmioMapPlanError::OutsideLowWindow);
    }

    let root_index = usize::try_from(page_start >> 30)
        .map_err(|_| MmioMapPlanError::OutsideLowWindow)?;
    let first_level1_index = usize::try_from(page_start / LEVEL1_SPAN)
        .map_err(|_| MmioMapPlanError::OutsideLowWindow)?;
    let last_level1_index = usize::try_from((rounded_end - 1) / LEVEL1_SPAN)
        .map_err(|_| MmioMapPlanError::OutsideLowWindow)?;
    let page_count = usize::try_from((rounded_end - page_start) / PAGE_SIZE)
        .map_err(|_| MmioMapPlanError::AddressOverflow)?;

    Ok(MmioMapPlan {
        root_index,
        page_start,
        page_end: rounded_end,
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
        assert_eq!(plan.root_index, 0);
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
        assert_eq!(plan.root_index, 0);
        assert_eq!(plan.page_start, 0x1000_0000);
        assert_eq!(plan.page_end, 0x1000_1000);
        assert_eq!(plan.first_level1_index, 128);
        assert_eq!(plan.last_level1_index, 128);
        assert_eq!(plan.page_count, 1);
        assert_eq!(plan.table_count, 1);
    }

    #[test]
    fn high_physical_resource_inside_sv39_low_half_is_supported() {
        // Representative high MMIO window (well above the first GiB).
        let plan = plan(0x0fd8_0000_00, 0x0040_0000).unwrap();
        assert_eq!(plan.root_index, 63);
        assert_eq!(plan.page_start, 0x0ffd_8000_000);
        assert_eq!(plan.page_count, 1024);
    }

    #[test]
    fn resource_crossing_a_one_gib_root_boundary_is_planned() {
        let plan = plan(0x3fff_f000, 0x3000).unwrap();
        assert_eq!(plan.root_index, 0);
        assert_eq!(plan.page_start, 0x3fff_f000);
        assert_eq!(plan.page_end, 0x4000_2000);
        assert_eq!(plan.first_level1_index, 511);
        assert_eq!(plan.last_level1_index, 512);
        assert_eq!(plan.table_count, 2);
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
