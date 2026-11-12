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
pub const USER_HEAP_BASE: usize = 0x9000_0000;
pub const USER_HEAP_SIZE: usize = L1_SPAN;

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
static KERNEL_SATP: AtomicUsize = AtomicUsize::new(0);
static USER_SATP: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmError {
    InvalidRange, OutsideBootstrapWindow, UserRegionsOverlap,
    UserRegionTooLarge, KernelImageTooLarge, MisalignedTable, SatpReadback,
    NoMemory, InvalidProtection, AddressNotMapped, OutsideUserHeap,
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

unsafe fn lookup_flags(root_address: usize, virtual_address: usize) -> Option<u64> {
    let root = &*(root_address as *const PageTable);
    let pte2 = root.0[(virtual_address >> 30) & 0x1ff];
    if pte2 & V == 0 { return None; }
    if pte2 & (R | W | X) != 0 { return Some(pte2 & 0x3ff); }
    let level1 = &*(pte_table_address(pte2) as *const PageTable);
    let pte1 = level1.0[(virtual_address >> 21) & 0x1ff];
    if pte1 & V == 0 { return None; }
    if pte1 & (R | W | X) != 0 { return Some(pte1 & 0x3ff); }
    let level0 = &*(pte_table_address(pte1) as *const PageTable);
    let pte0 = level0.0[(virtual_address >> 12) & 0x1ff];
    if pte0 & V == 0 || pte0 & (R | W | X) == 0 { return None; }
    Some(pte0 & 0x3ff)
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
    let heap_end = USER_HEAP_BASE.checked_add(USER_HEAP_SIZE)
        .ok_or(VmError::OutsideBootstrapWindow)?;
    if USER_HEAP_BASE < base || heap_end > end {
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
    for address in [kernel_root as usize, kernel_level1 as usize, kernel_image_l0 as usize, user_root as usize,
        user_level1 as usize, code_l0 as usize, stack_l0 as usize, heap_l0 as usize] {
        if address & (PAGE - 1) != 0 { return Err(VmError::MisalignedTable); }
    }

    (*kernel_root).0.fill(0);
    (*kernel_level1).0.fill(0);
    (*kernel_image_l0).0.fill(0);
    (*user_root).0.fill(0);
    (*user_level1).0.fill(0);
    (*code_l0).0.fill(0);
    (*stack_l0).0.fill(0);

    for i in 0..ENTRIES {
        (*kernel_level1).0[i] = leaf_pte(base + i * L1_SPAN, SUPERVISOR_RW);
    }
    fill_kernel_image_region(
        kernel_image_l0, base + image_region * L1_SPAN,
        text_start, text_end, rodata_start, rodata_end,
    );
    (*kernel_level1).0[image_region] = table_pte(kernel_image_l0 as usize);
    (*user_level1).0.copy_from_slice(&(*kernel_level1).0);
    let heap_region = (USER_HEAP_BASE >> 21) & 0x1ff;
    if heap_region == code_region || heap_region == stack_region {
        return Err(VmError::UserRegionTooLarge);
    }
    (*USER_HEAP_LEVEL0).0.fill(0);
    (*user_level1).0[heap_region] = table_pte(
        core::ptr::addr_of_mut!(USER_HEAP_LEVEL0) as usize,
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

/// Convert Win32 host protection bits to a valid Sv39 user leaf PTE.
fn user_leaf_flags(protection: u32) -> Result<u64, VmError> {
    const READ: u32 = 1;
    const WRITE: u32 = 2;
    const EXECUTE: u32 = 4;
    const USER: u32 = 8;
    if protection & !(READ | WRITE | EXECUTE | USER) != 0
        || protection & USER == 0
        || protection & (READ | EXECUTE) == 0
        || (protection & WRITE != 0 && protection & READ == 0) {
        return Err(VmError::InvalidProtection);
    }
    let mut flags = V | U | A;
    if protection & READ != 0 { flags |= R; }
    if protection & WRITE != 0 { flags |= W | D; }
    if protection & EXECUTE != 0 { flags |= X; }
    Ok(flags)
}

fn user_heap_page_range(address: usize, length: usize) -> Result<(usize, usize), VmError> {
    if length == 0 || address & (PAGE - 1) != 0 {
        return Err(VmError::InvalidRange);
    }
    let rounded = length.checked_add(PAGE - 1)
        .ok_or(VmError::InvalidRange)? & !(PAGE - 1);
    let end = address.checked_add(rounded).ok_or(VmError::InvalidRange)?;
    let heap_end = USER_HEAP_BASE.checked_add(USER_HEAP_SIZE)
        .ok_or(VmError::OutsideUserHeap)?;
    if address < USER_HEAP_BASE || end > heap_end {
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
        let first = (start - USER_HEAP_BASE) / PAGE;
        let count = (end - start) / PAGE;
        if count != pages || !table.0[first..first + pages].iter().all(|pte| pte & V == 0) {
            return Err(VmError::AddressNotMapped);
        }
        first
    };

    for offset in 0..pages {
        let physical = match allocator.alloc_in_range(0x8000_0000, 0xc000_0000) {
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
    Ok(USER_HEAP_BASE + first * PAGE)
}

/// Unmap only pages owned by the Win32 user heap arena and return them to PPA.
pub fn unmap_user_memory(
    allocator: &mut crate::page_alloc::PageAllocator,
    address: usize,
    length: usize,
) -> Result<(), VmError> {
    if !active() || !current_is_user() {
        return Err(VmError::OutsideUserHeap);
    }
    let (start, end) = user_heap_page_range(address, length)?;
    let first = (start - USER_HEAP_BASE) / PAGE;
    let pages = (end - start) / PAGE;
    let table = unsafe { &mut *core::ptr::addr_of_mut!(USER_HEAP_LEVEL0) };
    if !table.0[first..first + pages].iter().all(|pte| pte & (V | U) == (V | U)) {
        return Err(VmError::AddressNotMapped);
    }
    for index in first..first + pages {
        let physical = pte_table_address(table.0[index]);
        allocator.free(physical).map_err(|_| VmError::AddressNotMapped)?;
        table.0[index] = 0;
    }
    unsafe { asm!("sfence.vma zero, zero", options(nostack)); }
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
    let first = (start - USER_HEAP_BASE) / PAGE;
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
