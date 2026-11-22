//! Sv39 bootstrap plus a bounded dynamic heap arena for one built-in U-mode task.
//! The kernel root is supervisor-only; the user root grants U access to its RX
//! probe text, RW stack, and separately allocated pages in a 2 MiB heap window.
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

const PAGE: usize = 4096;
const L1_SPAN: usize = 2 * 1024 * 1024;
const L2_SPAN: usize = 1024 * 1024 * 1024;
const ENTRIES: usize = 512;
const V: u64 = 1 << 0;
const R: u64 = 1 << 1;
const W: u64 = 1 << 2;
const X: u64 = 1 << 3;
const U: u64 = 1 << 4;
const A: u64 = 1 << 6;
const D: u64 = 1 << 7;
const SUPERVISOR_RW: u64 = V | R | W | A | D;
const SUPERVISOR_RX: u64 = V | R | X | A;
const SUPERVISOR_R: u64 = V | R | A;
const SV39_MODE: usize = 8usize << 60;
const SATP_PPN_MASK: usize = (1usize << 44) - 1;
const MMIO_TABLE_COUNT: usize = 8;
const DEVICE_SUPERVISOR_RW: u64 = V | R | W | A | D;
static USER_HEAP_BASE: AtomicUsize = AtomicUsize::new(0);
static KERNEL_WINDOW_BASE: AtomicUsize = AtomicUsize::new(0);
const USER_HEAP_SIZE: usize = crate::vm_policy::USER_HEAP_SIZE;

#[derive(Clone, Copy)]
#[repr(C, align(4096))]
struct PageTable([u64; ENTRIES]);

static mut KERNEL_ROOT: PageTable = PageTable([0; ENTRIES]);
static mut KERNEL_LEVEL1: PageTable = PageTable([0; ENTRIES]);
static mut KERNEL_IMAGE_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static mut USER_ROOT: PageTable = PageTable([0; ENTRIES]);
static mut USER_LEVEL1: PageTable = PageTable([0; ENTRIES]);
static mut USER_CODE_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static mut USER_STACK_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static mut USER_HEAP_LEVEL0: PageTable = PageTable([0; ENTRIES]);
// Shared supervisor-only device mappings are present in both roots so S-mode
// trap handlers can access them while a U-mode process root is active.
static mut MMIO_LEVEL1: PageTable = PageTable([0; ENTRIES]);
static mut MMIO_LEVEL0_TABLES: [PageTable; MMIO_TABLE_COUNT] = [PageTable([0; ENTRIES]); MMIO_TABLE_COUNT];
static KERNEL_SATP: AtomicUsize = AtomicUsize::new(0);
static USER_SATP: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmError {
    InvalidRange, OutsideBootstrapWindow, UserRegionsOverlap,
    UserRegionTooLarge, KernelImageTooLarge, MisalignedTable, SatpReadback,
    NoMemory, InvalidProtection, AddressNotMapped, OutsideUserHeap,
    MmioNotReady, MmioConflict, MmioTableExhausted,
    MmioPlan(crate::mmio_map::MmioMapPlanError),
}

fn table_pte(address: usize) -> u64 { (((address as u64) >> 12) << 10) | V }
fn leaf_pte(address: usize, flags: u64) -> u64 { (((address as u64) >> 12) << 10) | flags }
fn pte_table_address(pte: u64) -> usize {
    (((pte >> 10) & ((1u64 << 44) - 1)) << 12) as usize
}

fn overlaps_page(address: usize, page_end: usize, start: usize, end: usize) -> bool {
    start < end && address < end && start < page_end
}

unsafe fn fill_kernel_image_region(
    table: *mut PageTable, region_base: usize,
    text_start: usize, text_end: usize, rodata_start: usize, rodata_end: usize,
) {
    for i in 0..ENTRIES {
        let address = region_base + i * PAGE;
        let page_end = address + PAGE;
        let flags = if overlaps_page(address, page_end, text_start, text_end) {
            SUPERVISOR_RX
        } else if overlaps_page(address, page_end, rodata_start, rodata_end) {
            SUPERVISOR_R
        } else {
            SUPERVISOR_RW
        };
        (*table).0[i] = leaf_pte(address, flags);
    }
}

unsafe fn fill_user_region(
    table: *mut PageTable, region_base: usize,
    code_start: usize, code_end: usize, stack_start: usize, stack_end: usize,
    text_start: usize, text_end: usize, rodata_start: usize, rodata_end: usize,
) {
    for i in 0..ENTRIES {
        let address = region_base + i * PAGE;
        let page_end = address + PAGE;
        let code_page = overlaps_page(address, page_end, code_start, code_end);
        let stack_page = overlaps_page(address, page_end, stack_start, stack_end);
        let flags = if code_page { V | R | X | U | A }
            else if stack_page { V | R | W | U | A | D }
            else if overlaps_page(address, page_end, text_start, text_end) {
                SUPERVISOR_RX
            } else if overlaps_page(address, page_end, rodata_start, rodata_end) {
                SUPERVISOR_R
            } else {
                SUPERVISOR_RW
            };
        (*table).0[i] = leaf_pte(address, flags);
    }
}

unsafe fn lookup_mapping(
    root_address: usize,
    virtual_address: usize,
) -> Option<(usize, u64)> {
    let root = &*(root_address as *const PageTable);
    let pte2 = root.0[(virtual_address >> 30) & 0x1ff];
    if pte2 & V == 0 { return None; }
    if pte2 & (R | W | X) != 0 {
        let physical = pte_table_address(pte2) | (virtual_address & (L2_SPAN - 1));
        return Some((physical, pte2 & 0x3ff));
    }
    let level1 = &*(pte_table_address(pte2) as *const PageTable);
    let pte1 = level1.0[(virtual_address >> 21) & 0x1ff];
    if pte1 & V == 0 { return None; }
    if pte1 & (R | W | X) != 0 {
        let physical = pte_table_address(pte1) | (virtual_address & (L1_SPAN - 1));
        return Some((physical, pte1 & 0x3ff));
    }
    let level0 = &*(pte_table_address(pte1) as *const PageTable);
    let pte0 = level0.0[(virtual_address >> 12) & 0x1ff];
    if pte0 & V == 0 || pte0 & (R | W | X) == 0 { return None; }
    let physical = pte_table_address(pte0) | (virtual_address & (PAGE - 1));
    Some((physical, pte0 & 0x3ff))
}

unsafe fn lookup_flags(root_address: usize, virtual_address: usize) -> Option<u64> {
    lookup_mapping(root_address, virtual_address).map(|(_, flags)| flags)
}

/// Construct separate kernel and probe-user roots and activate the kernel root.
pub unsafe fn enable_bootstrap(
    kernel_start: usize, kernel_end: usize, fdt_start: usize, fdt_end: usize,
    user_code_start: usize, user_code_end: usize,
    user_stack_start: usize, user_stack_end: usize,
    text_start: usize, text_end: usize,
    rodata_start: usize, rodata_end: usize,
    data_start: usize, data_end: usize,
    bss_start: usize, bss_end: usize,
) -> Result<(), VmError> {
    if kernel_start >= kernel_end || fdt_start >= fdt_end
        || user_code_start >= user_code_end || user_stack_start >= user_stack_end
        || text_start >= text_end || rodata_start > rodata_end
        || data_start > data_end || bss_start >= bss_end {
        return Err(VmError::InvalidRange);
    }
    if user_code_start < user_stack_end && user_stack_start < user_code_end {
        return Err(VmError::UserRegionsOverlap);
    }
    let base = kernel_start & !(L2_SPAN - 1);
    let end = base.checked_add(L2_SPAN).ok_or(VmError::InvalidRange)?;
    let heap_base = crate::vm_policy::user_heap_base(base)
        .ok_or(VmError::OutsideBootstrapWindow)?;
    let heap_end = heap_base.checked_add(USER_HEAP_SIZE)
        .ok_or(VmError::OutsideBootstrapWindow)?;
    if heap_base < base || heap_end > end {
        return Err(VmError::OutsideBootstrapWindow);
    }
    let in_window = |start: usize, finish: usize| {
        finish.checked_sub(1).map(|last| start >= base && start < end && last >= base && last < end)
            .unwrap_or(false)
    };
    if !in_window(kernel_start, kernel_end) || !in_window(fdt_start, fdt_end)
        || !in_window(user_code_start, user_code_end)
        || !in_window(user_stack_start, user_stack_end) {
        return Err(VmError::OutsideBootstrapWindow);
    }
    let image_region = (kernel_start - base) / L1_SPAN;
    let image_region_end = (kernel_end - 1 - base) / L1_SPAN;
    let code_region = (user_code_start - base) / L1_SPAN;
    let code_region_end = (user_code_end - 1 - base) / L1_SPAN;
    let stack_region = (user_stack_start - base) / L1_SPAN;
    let stack_region_end = (user_stack_end - 1 - base) / L1_SPAN;
    if code_region != code_region_end || stack_region != stack_region_end {
        return Err(VmError::UserRegionTooLarge);
    }
    if image_region != image_region_end
        || code_region != image_region || stack_region != image_region
        || user_code_start < text_start || user_code_end > text_end
        || user_stack_start < bss_start || user_stack_end > bss_end {
        return Err(VmError::KernelImageTooLarge);
    }
    for (start, finish) in [
        (text_start, text_end), (rodata_start, rodata_end),
        (data_start, data_end), (bss_start, bss_end),
    ] {
        if start < finish
            && (!in_window(start, finish)
                || (start - base) / L1_SPAN != image_region
                || (finish - 1 - base) / L1_SPAN != image_region) {
            return Err(VmError::KernelImageTooLarge);
        }
    }

    let kernel_root = core::ptr::addr_of_mut!(KERNEL_ROOT);
    let kernel_level1 = core::ptr::addr_of_mut!(KERNEL_LEVEL1);
    let kernel_image_l0 = core::ptr::addr_of_mut!(KERNEL_IMAGE_LEVEL0);
    let user_root = core::ptr::addr_of_mut!(USER_ROOT);
    let user_level1 = core::ptr::addr_of_mut!(USER_LEVEL1);
    let code_l0 = core::ptr::addr_of_mut!(USER_CODE_LEVEL0);
    let stack_l0 = core::ptr::addr_of_mut!(USER_STACK_LEVEL0);
    let heap_l0 = core::ptr::addr_of_mut!(USER_HEAP_LEVEL0);
    let mmio_level1 = core::ptr::addr_of_mut!(MMIO_LEVEL1);
    let mmio_level0_tables = core::ptr::addr_of_mut!(MMIO_LEVEL0_TABLES);
    for address in [kernel_root as usize, kernel_level1 as usize, kernel_image_l0 as usize, user_root as usize,
        user_level1 as usize, code_l0 as usize, stack_l0 as usize, heap_l0 as usize,
        mmio_level1 as usize, mmio_level0_tables as usize] {
        if address & (PAGE - 1) != 0 { return Err(VmError::MisalignedTable); }
    }

    (*kernel_root).0.fill(0);
    (*kernel_level1).0.fill(0);
    (*kernel_image_l0).0.fill(0);
    (*user_root).0.fill(0);
    (*user_level1).0.fill(0);
    (*code_l0).0.fill(0);
    (*stack_l0).0.fill(0);
    (*mmio_level1).0.fill(0);
    let mmio_tables = mmio_level0_tables.cast::<PageTable>();
    for index in 0..MMIO_TABLE_COUNT {
        (*mmio_tables.add(index)).0.fill(0);
    }

    for i in 0..ENTRIES {
        (*kernel_level1).0[i] = leaf_pte(base + i * L1_SPAN, SUPERVISOR_RW);
    }
    fill_kernel_image_region(
        kernel_image_l0, base + image_region * L1_SPAN,
        text_start, text_end, rodata_start, rodata_end,
    );
    (*kernel_level1).0[image_region] = table_pte(kernel_image_l0 as usize);
    (*user_level1).0.copy_from_slice(&(*kernel_level1).0);
    let heap_region = crate::vm_policy::l1_region_index(heap_base);
    if heap_region == image_region || heap_region == code_region || heap_region == stack_region {
        return Err(VmError::UserRegionTooLarge);
    }
    (*heap_l0).0.fill(0);
    (*user_level1).0[heap_region] = table_pte(
        heap_l0 as usize,
    );

    fill_user_region(
        code_l0, base + code_region * L1_SPAN,
        user_code_start, user_code_end, user_stack_start, user_stack_end,
        text_start, text_end, rodata_start, rodata_end,
    );
    (*user_level1).0[code_region] = table_pte(code_l0 as usize);
    if stack_region != code_region {
        fill_user_region(
            stack_l0, base + stack_region * L1_SPAN,
            user_code_start, user_code_end, user_stack_start, user_stack_end,
            text_start, text_end, rodata_start, rodata_end,
        );
        (*user_level1).0[stack_region] = table_pte(stack_l0 as usize);
    }

    let root_index = (base >> 30) & 0x1ff;
    (*kernel_root).0[root_index] = table_pte(kernel_level1 as usize);
    (*user_root).0[root_index] = table_pte(user_level1 as usize);
    let kernel_satp = SV39_MODE | (((kernel_root as usize) >> 12) & SATP_PPN_MASK);
    let user_satp = SV39_MODE | (((user_root as usize) >> 12) & SATP_PPN_MASK);
    if kernel_satp == user_satp { return Err(VmError::MisalignedTable); }
    KERNEL_WINDOW_BASE.store(base, Ordering::Release);
    USER_HEAP_BASE.store(heap_base, Ordering::Release);
    KERNEL_SATP.store(kernel_satp, Ordering::Release);
    USER_SATP.store(user_satp, Ordering::Release);
    write_satp(kernel_satp);

    let readback: usize;
    asm!("csrr {value}, satp", value = out(reg) readback, options(nomem, nostack));
    if readback != kernel_satp { return Err(VmError::SatpReadback); }
    Ok(())
}

#[inline]
fn write_satp(satp: usize) {
    unsafe { asm!("csrw satp, {satp}", "sfence.vma zero, zero",
        satp = in(reg) satp, options(nostack)); }
}

pub fn switch_to_kernel() -> bool {
    let satp = KERNEL_SATP.load(Ordering::Acquire);
    if satp == 0 { return false; }
    write_satp(satp);
    true
}

pub fn switch_to_user() -> bool {
    let satp = USER_SATP.load(Ordering::Acquire);
    if satp == 0 { return false; }
    write_satp(satp);
    true
}

/// Install an identity-mapped, supervisor-only Sv39 window for an FDT-validated
/// device resource. The first HAL pass is deliberately limited to physical
/// addresses below 1 GiB and uses 4 KiB leaves. Base Sv39 has no cacheability
/// PTE field: platform PMA defines the region as device memory.
pub fn map_kernel_mmio_region(region: crate::mmio::MmioRegion) -> Result<(), VmError> {
    let plan = crate::mmio_map::plan(region.base(), region.size())
        .map_err(VmError::MmioPlan)?;
    if plan.table_count > MMIO_TABLE_COUNT {
        return Err(VmError::MmioTableExhausted);
    }
    let kernel_satp = KERNEL_SATP.load(Ordering::Acquire);
    let user_satp = USER_SATP.load(Ordering::Acquire);
    if kernel_satp == 0 || user_satp == 0 || !active() {
        return Err(VmError::MmioNotReady);
    }
    let current: usize;
    unsafe { asm!("csrr {value}, satp", value = out(reg) current, options(nomem, nostack)); }
    if current != kernel_satp {
        return Err(VmError::MmioNotReady);
    }

    let kernel_root_address = pte_table_root(kernel_satp);
    let user_root_address = pte_table_root(user_satp);
    unsafe {
        let mmio_level1 = core::ptr::addr_of_mut!(MMIO_LEVEL1);
        let mmio_level0_tables = core::ptr::addr_of_mut!(MMIO_LEVEL0_TABLES).cast::<PageTable>();
        let root_entry = table_pte(mmio_level1 as usize);

        for root_address in [kernel_root_address, user_root_address] {
            let root = &mut *(root_address as *mut PageTable);
            let existing = root.0[0];
            if existing != 0 && existing != root_entry {
                return Err(VmError::MmioConflict);
            }
        }

        let mut address = plan.page_start;
        while address < plan.page_end {
            let level1_index = (address / L1_SPAN as u64) as usize;
            let level0_index = ((address % L1_SPAN as u64) / PAGE as u64) as usize;
            let level1_entry = (*mmio_level1).0[level1_index];
            let table = if level1_entry & V != 0 {
                if level1_entry & (R | W | X) != 0 {
                    return Err(VmError::MmioConflict);
                }
                let table_address = pte_table_address(level1_entry);
                let mut found = core::ptr::null_mut();
                for slot in 0..MMIO_TABLE_COUNT {
                    let candidate = mmio_level0_tables.add(slot);
                    if candidate as usize == table_address {
                        found = candidate;
                        break;
                    }
                }
                if found.is_null() {
                    return Err(VmError::MmioConflict);
                }
                found
            } else {
                let mut free_table = core::ptr::null_mut();
                for slot in 0..MMIO_TABLE_COUNT {
                    let candidate = mmio_level0_tables.add(slot);
                    let mut used = false;
                    for index in 0..ENTRIES {
                        let entry = (*mmio_level1).0[index];
                        if entry & V != 0 && entry & (R | W | X) == 0
                            && pte_table_address(entry) == candidate as usize {
                            used = true;
                            break;
                        }
                    }
                    if !used {
                        free_table = candidate;
                        break;
                    }
                }
                if free_table.is_null() {
                    return Err(VmError::MmioTableExhausted);
                }
                (*free_table).0.fill(0);
                (*mmio_level1).0[level1_index] = table_pte(free_table as usize);
                free_table
            };

            let page_entry = leaf_pte(address as usize, DEVICE_SUPERVISOR_RW);
            let existing = (*table).0[level0_index];
            if existing != 0 && existing != page_entry {
                return Err(VmError::MmioConflict);
            }
            (*table).0[level0_index] = page_entry;
            address += PAGE as u64;
        }

        (*((kernel_root_address) as *mut PageTable)).0[0] = root_entry;
        (*((user_root_address) as *mut PageTable)).0[0] = root_entry;
        asm!("fence iorw, iorw", "sfence.vma zero, zero", options(nostack));
    }

    if verify_kernel_mmio_region(region) {
        Ok(())
    } else {
        Err(VmError::MmioConflict)
    }
}

/// Verify identity translation and supervisor RW/NX permissions in both roots.
/// U-mode remains unable to access the mapping even while its root is active.
pub fn verify_kernel_mmio_region(region: crate::mmio::MmioRegion) -> bool {
    let plan = match crate::mmio_map::plan(region.base(), region.size()) {
        Ok(plan) => plan,
        Err(_) => return false,
    };
    let kernel_satp = KERNEL_SATP.load(Ordering::Acquire);
    let user_satp = USER_SATP.load(Ordering::Acquire);
    if kernel_satp == 0 || user_satp == 0 { return false; }
    let required = V | R | W | A | D;
    let forbidden = X | U;
    let roots = [pte_table_root(kernel_satp), pte_table_root(user_satp)];
    let mut address = plan.page_start as usize;
    let end = plan.page_end as usize;
    while address < end {
        for root in roots {
            let mapping = unsafe { lookup_mapping(root, address) };
            match mapping {
                Some((physical, flags))
                    if physical == address
                        && flags & required == required
                        && flags & forbidden == 0 => {}
                _ => return false,
            }
        }
        address += PAGE;
    }
    true
}

fn user_leaf_flags(protection: u32) -> Result<u64, VmError> {
    crate::vm_policy::user_leaf_flags(protection).map_err(|_| VmError::InvalidProtection)
}

fn user_heap_page_range(address: usize, length: usize) -> Result<(usize, usize), VmError> {
    if length == 0 || address & (PAGE - 1) != 0 {
        return Err(VmError::InvalidRange);
    }
    let rounded = length.checked_add(PAGE - 1)
        .ok_or(VmError::InvalidRange)? & !(PAGE - 1);
    let end = address.checked_add(rounded).ok_or(VmError::InvalidRange)?;
    let heap_base = USER_HEAP_BASE.load(Ordering::Acquire);
    let heap_end = heap_base.checked_add(USER_HEAP_SIZE)
        .ok_or(VmError::OutsideUserHeap)?;
    if heap_base == 0 || address < heap_base || end > heap_end {
        return Err(VmError::OutsideUserHeap);
    }
    Ok((address, end))
}

/// Map zero-filled physical pages into the single built-in U-mode process arena.
/// The initial implementation is deliberately bounded to one 2 MiB Sv39 leaf.
pub fn map_user_memory(
    allocator: &mut crate::page_alloc::PageAllocator,
    hint: usize,
    length: usize,
    protection: u32,
) -> Result<usize, VmError> {
    if !active() || !current_is_user() {
        return Err(VmError::OutsideUserHeap);
    }
    let flags = user_leaf_flags(protection)?;
    if length == 0 {
        return Err(VmError::InvalidRange);
    }
    let rounded = length.checked_add(PAGE - 1)
        .ok_or(VmError::InvalidRange)? & !(PAGE - 1);
    if rounded == 0 || rounded > USER_HEAP_SIZE {
        return Err(VmError::OutsideUserHeap);
    }
    let pages = rounded / PAGE;
    let heap_base = USER_HEAP_BASE.load(Ordering::Acquire);
    let window_base = KERNEL_WINDOW_BASE.load(Ordering::Acquire);
    let window_end = window_base.checked_add(L2_SPAN).ok_or(VmError::OutsideBootstrapWindow)?;
    if heap_base == 0 || window_base == 0 { return Err(VmError::OutsideBootstrapWindow); }
    let table = unsafe { &mut *core::ptr::addr_of_mut!(USER_HEAP_LEVEL0) };
    let first = if hint == 0 {
        let mut found = None;
        for start in 0..=ENTRIES - pages {
            if table.0[start..start + pages].iter().all(|pte| pte & V == 0) {
                found = Some(start);
                break;
            }
        }
        found.ok_or(VmError::NoMemory)?
    } else {
        let (start, end) = user_heap_page_range(hint, length)?;
        let first = (start - heap_base) / PAGE;
        let count = (end - start) / PAGE;
        if count != pages || !table.0[first..first + pages].iter().all(|pte| pte & V == 0) {
            return Err(VmError::AddressNotMapped);
        }
        first
    };

    for offset in 0..pages {
        let physical = match allocator.alloc_in_range(window_base, window_end) {
            Some(page) => page,
            None => {
                for rollback in 0..offset {
                    let pte = table.0[first + rollback];
                    let old_page = pte_table_address(pte);
                    table.0[first + rollback] = 0;
                    let _ = allocator.free(old_page);
                }
                return Err(VmError::NoMemory);
            }
        };
        /* The user root remaps the heap VA region, so zero physical pages
           under the kernel root before returning to the user address space. */
        if !switch_to_kernel() {
            let _ = allocator.free(physical);
            for rollback in 0..offset {
                let old_page = pte_table_address(table.0[first + rollback]);
                table.0[first + rollback] = 0;
                let _ = allocator.free(old_page);
            }
            let _ = switch_to_user();
            return Err(VmError::SatpReadback);
        }
        unsafe { core::ptr::write_bytes(physical as *mut u8, 0, PAGE); }
        table.0[first + offset] = leaf_pte(physical, flags);
        if !switch_to_user() {
            return Err(VmError::SatpReadback);
        }
    }
    unsafe { asm!("sfence.vma zero, zero", options(nostack)); }
    Ok(heap_base + first * PAGE)
}

/// Unmap owned user pages transactionally, invalidate translations, scrub the
/// frames under the supervisor root, then return them to the physical allocator.
/// Two passes avoid a large temporary array on the 4 KiB supervisor trap stack.
pub fn unmap_user_memory(
    allocator: &mut crate::page_alloc::PageAllocator,
    address: usize,
    length: usize,
) -> Result<(), VmError> {
    if !active() || !current_is_user() {
        return Err(VmError::OutsideUserHeap);
    }
    let (start, end) = user_heap_page_range(address, length)?;
    let heap_base = USER_HEAP_BASE.load(Ordering::Acquire);
    let first = (start - heap_base) / PAGE;
    let pages = (end - start) / PAGE;
    let table = unsafe { &mut *core::ptr::addr_of_mut!(USER_HEAP_LEVEL0) };
    for offset in 0..pages {
        let pte = table.0[first + offset];
        let flags = pte & 0x3ff;
        if flags & (V | U) != (V | U)
            || flags & (R | W | X) == 0
            || (flags & W != 0 && (flags & R == 0 || flags & X != 0)) {
            return Err(VmError::AddressNotMapped);
        }
        let physical = pte_table_address(pte);
        if !allocator.is_allocated(physical) {
            return Err(VmError::AddressNotMapped);
        }
        for prior in 0..offset {
            if pte_table_address(table.0[first + prior]) == physical {
                return Err(VmError::AddressNotMapped);
            }
        }
    }
    if !switch_to_kernel() {
        return Err(VmError::SatpReadback);
    }
    for offset in 0..pages {
        let index = first + offset;
        let physical = pte_table_address(table.0[index]);
        table.0[index] = 0;
        unsafe { core::ptr::write_bytes(physical as *mut u8, 0, PAGE); }
        if allocator.free(physical).is_err() { crate::halt(); }
    }
    unsafe { asm!("sfence.vma zero, zero", options(nostack)); }
    if !switch_to_user() { crate::halt(); }
    Ok(())
}

/// Change permissions only for pages already owned by the user heap arena.
pub fn protect_user_memory(
    address: usize,
    length: usize,
    protection: u32,
) -> Result<(), VmError> {
    if !active() || !current_is_user() {
        return Err(VmError::OutsideUserHeap);
    }
    let flags = user_leaf_flags(protection)?;
    let (start, end) = user_heap_page_range(address, length)?;
    let heap_base = USER_HEAP_BASE.load(Ordering::Acquire);
    let first = (start - heap_base) / PAGE;
    let pages = (end - start) / PAGE;
    let table = unsafe { &mut *core::ptr::addr_of_mut!(USER_HEAP_LEVEL0) };
    if !table.0[first..first + pages].iter().all(|pte| pte & (V | U) == (V | U)) {
        return Err(VmError::AddressNotMapped);
    }
    for index in first..first + pages {
        let physical = table.0[index] & !0x3ff;
        table.0[index] = physical | flags;
    }
    unsafe { asm!("sfence.vma zero, zero", options(nostack)); }
    Ok(())
}

pub fn current_is_user() -> bool {
    let current: usize;
    unsafe { asm!("csrr {value}, satp", value = out(reg) current, options(nomem, nostack)); }
    let user = USER_SATP.load(Ordering::Acquire);
    user != 0 && current == user
}

pub fn roots_are_distinct() -> bool {
    let kernel = KERNEL_SATP.load(Ordering::Acquire);
    let user = USER_SATP.load(Ordering::Acquire);
    kernel != 0 && user != 0 && kernel != user
}

pub fn active() -> bool {
    let satp: usize;
    unsafe { asm!("csrr {value}, satp", value = out(reg) satp, options(nomem, nostack)); }
    (satp >> 60) == 8
}

fn pte_table_root(satp: usize) -> usize { (satp & SATP_PPN_MASK) << 12 }

unsafe fn has_permissions(root: usize, address: usize, required: u64, forbidden: u64) -> bool {
    lookup_flags(root, address)
        .map(|flags| flags & required == required && flags & forbidden == 0)
        .unwrap_or(false)
}

/// Check every 4 KiB leaf in a page-aligned range, not just its first page.
unsafe fn has_permissions_range(
    root: usize, start: usize, end: usize, required: u64, forbidden: u64,
) -> bool {
    if start > end || start & (PAGE - 1) != 0 || end & (PAGE - 1) != 0 {
        return false;
    }
    let mut address = start;
    while address < end {
        if !has_permissions(root, address, required, forbidden) {
            return false;
        }
        address = match address.checked_add(PAGE) {
            Some(next) => next,
            None => return false,
        };
    }
    true
}

/// Check a section whose designated user pages intentionally differ from the
/// surrounding supervisor-only mapping (U-mode text inside .text or stack in .bss).
unsafe fn has_permissions_range_with_user_override(
    root: usize,
    start: usize,
    end: usize,
    user_start: usize,
    user_end: usize,
    supervisor_required: u64,
    supervisor_forbidden: u64,
    user_required: u64,
    user_forbidden: u64,
) -> bool {
    if start >= end || user_start >= user_end
        || user_start < start || user_end > end
        || start & (PAGE - 1) != 0 || end & (PAGE - 1) != 0
        || user_start & (PAGE - 1) != 0 || user_end & (PAGE - 1) != 0 {
        return false;
    }
    let mut address = start;
    while address < end {
        let page_end = match address.checked_add(PAGE) {
            Some(next) => next,
            None => return false,
        };
        let (required, forbidden) =
            if overlaps_page(address, page_end, user_start, user_end) {
                (user_required, user_forbidden)
            } else {
                (supervisor_required, supervisor_forbidden)
            };
        if !has_permissions(root, address, required, forbidden) {
            return false;
        }
        address = page_end;
    }
    true
}

/// Verify every mapped page in the kernel sections, U-mode text/stack and
/// supervisor trap stack. U-mode probe pages are explicit exceptions inside
/// the otherwise supervisor-only .text and .bss ranges.
pub unsafe fn verify_isolation(
    user_code_start: usize, user_code_end: usize,
    user_stack_start: usize, user_stack_end: usize,
    kernel_page: usize,
    supervisor_stack_start: usize, supervisor_stack_end: usize,
    text_start: usize, text_end: usize,
    rodata_start: usize, rodata_end: usize,
    data_start: usize, data_end: usize,
    bss_start: usize, bss_end: usize,
) -> bool {
    let kernel_satp = KERNEL_SATP.load(Ordering::Acquire);
    let user_satp = USER_SATP.load(Ordering::Acquire);
    if kernel_satp == 0 || user_satp == 0 || kernel_satp == user_satp {
        return false;
    }
    let kernel_root = pte_table_root(kernel_satp);
    let user_root = pte_table_root(user_satp);

    let kernel_text = has_permissions_range(kernel_root, text_start, text_end, R | X, W | U);
    let user_text = has_permissions_range_with_user_override(
        user_root, text_start, text_end, user_code_start, user_code_end,
        R | X, W | U, U | R | X, W,
    );
    let kernel_rodata = has_permissions_range(kernel_root, rodata_start, rodata_end, R, W | X | U);
    let user_rodata = has_permissions_range(user_root, rodata_start, rodata_end, R, W | X | U);
    let kernel_data = has_permissions_range(kernel_root, data_start, data_end, R | W, X | U);
    let user_data = has_permissions_range(user_root, data_start, data_end, R | W, X | U);
    let kernel_bss = has_permissions_range(kernel_root, bss_start, bss_end, R | W, X | U);
    let user_bss = has_permissions_range_with_user_override(
        user_root, bss_start, bss_end, user_stack_start, user_stack_end,
        R | W, X | U, U | R | W, X,
    );
    let user_code = has_permissions_range(user_root, user_code_start, user_code_end, U | R | X, W);
    let user_stack = has_permissions_range(user_root, user_stack_start, user_stack_end, U | R | W, X);
    let kernel_page_is_rx = has_permissions(user_root, kernel_page, R | X, W | U);
    let supervisor_stack = has_permissions_range(
        user_root, supervisor_stack_start, supervisor_stack_end, R | W, X | U,
    ) && has_permissions_range(
        kernel_root, supervisor_stack_start, supervisor_stack_end, R | W, X | U,
    );

    kernel_text && user_text && kernel_rodata && user_rodata
        && kernel_data && user_data && kernel_bss && user_bss
        && user_code && user_stack && kernel_page_is_rx && supervisor_stack
}
