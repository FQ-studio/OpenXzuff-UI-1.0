//! Sv39 bootstrap plus a distinct address space for the built-in U-mode task.
//! The kernel root is supervisor-only; the probe root grants U access only to
//! its RX text and RW stack. This is not yet a dynamic VM manager.
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

#[repr(C, align(4096))]
struct PageTable([u64; ENTRIES]);

static mut KERNEL_ROOT: PageTable = PageTable([0; ENTRIES]);
static mut KERNEL_LEVEL1: PageTable = PageTable([0; ENTRIES]);
static mut KERNEL_IMAGE_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static mut USER_ROOT: PageTable = PageTable([0; ENTRIES]);
static mut USER_LEVEL1: PageTable = PageTable([0; ENTRIES]);
static mut USER_CODE_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static mut USER_STACK_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static KERNEL_SATP: AtomicUsize = AtomicUsize::new(0);
static USER_SATP: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmError {
    InvalidRange, OutsideBootstrapWindow, UserRegionsOverlap,
    UserRegionTooLarge, KernelImageTooLarge, MisalignedTable, SatpReadback,
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
    if image_region != image_region_end
        || code_region != code_region_end || stack_region != stack_region_end
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
    for address in [kernel_root as usize, kernel_level1 as usize, kernel_image_l0 as usize, user_root as usize,
        user_level1 as usize, code_l0 as usize, stack_l0 as usize] {
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

/// Verify W^X kernel sections plus RX/RW U-mode mappings in both address spaces.
pub unsafe fn verify_isolation(
    user_code: usize, user_stack: usize, kernel_page: usize, supervisor_stack: usize,
    text: usize, rodata: Option<usize>, data: usize, bss: usize,
) -> bool {
    let kernel_satp = KERNEL_SATP.load(Ordering::Acquire);
    let user_satp = USER_SATP.load(Ordering::Acquire);
    if kernel_satp == 0 || user_satp == 0 || kernel_satp == user_satp { return false; }
    let kernel_root = pte_table_root(kernel_satp);
    let user_root = pte_table_root(user_satp);
    let kernel_text = has_permissions(kernel_root, text, R | X, W | U)
        && has_permissions(user_root, text, R | X, W | U);
    let kernel_rodata = rodata.map(|address| {
        has_permissions(kernel_root, address, R, W | X | U)
            && has_permissions(user_root, address, R, W | X | U)
    }).unwrap_or(true);
    let kernel_data = has_permissions(kernel_root, data, R | W, X | U)
        && has_permissions(user_root, data, R | W, X | U);
    let kernel_bss = has_permissions(kernel_root, bss, R | W, X | U)
        && has_permissions(user_root, bss, R | W, X | U);
    let user_text = has_permissions(user_root, user_code, U | R | X, W);
    let user_stack_flags = has_permissions(user_root, user_stack, U | R | W, X);
    let supervisor_only = has_permissions(user_root, kernel_page, 0, U)
        && has_permissions(user_root, supervisor_stack, 0, U);
    kernel_text && kernel_rodata && kernel_data && kernel_bss
        && user_text && user_stack_flags && supervisor_only
}
