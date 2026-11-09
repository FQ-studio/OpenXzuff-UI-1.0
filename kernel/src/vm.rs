//! Bootstrap Sv39 page tables for the RV64 kernel.
//! A 1 GiB identity window is split into 2 MiB supervisor leaves.
//! Dedicated 4 KiB leaves expose only the U-mode probe text and its stack.
use core::arch::asm;

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
const SUPERVISOR_RWX: u64 = V | R | W | X | A | D;
const SV39_MODE: usize = 8usize << 60;
const SATP_PPN_MASK: usize = (1usize << 44) - 1;

#[repr(C, align(4096))]
struct PageTable([u64; ENTRIES]);

static mut ROOT: PageTable = PageTable([0; ENTRIES]);
static mut LEVEL1: PageTable = PageTable([0; ENTRIES]);
static mut CODE_LEVEL0: PageTable = PageTable([0; ENTRIES]);
static mut STACK_LEVEL0: PageTable = PageTable([0; ENTRIES]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmError {
    InvalidRange,
    OutsideBootstrapWindow,
    UserRegionsOverlap,
    UserRegionTooLarge,
    MisalignedTable,
    SatpReadback,
}

fn table_pte(address: usize) -> u64 {
    (((address as u64) >> 12) << 10) | V
}

fn leaf_pte(address: usize, flags: u64) -> u64 {
    (((address as u64) >> 12) << 10) | flags
}

fn in_window(address: usize, base: usize, end: usize) -> bool {
    address >= base && address < end
}

unsafe fn fill_level0(
    table: *mut PageTable,
    region_base: usize,
    code_start: usize,
    code_end: usize,
    stack_start: usize,
    stack_end: usize,
) {
    for i in 0..ENTRIES {
        let address = region_base + i * PAGE;
        let page_end = address + PAGE;
        let code_page = address < code_end && code_start < page_end;
        let stack_page = address < stack_end && stack_start < page_end;
        let flags = if code_page {
            V | R | X | U | A
        } else if stack_page {
            V | R | W | U | A | D
        } else {
            SUPERVISOR_RWX
        };
        (*table).0[i] = leaf_pte(address, flags);
    }
}

/// Enable one 1 GiB Sv39 identity window. Kernel/FDT pages remain supervisor-only.
/// The dedicated U-mode probe text and stack are the only U=1 mappings.
/// This is bootstrap translation, not yet a per-process address-space manager.
pub unsafe fn enable_bootstrap(
    kernel_start: usize,
    kernel_end: usize,
    fdt_start: usize,
    fdt_end: usize,
    user_code_start: usize,
    user_code_end: usize,
    user_stack_start: usize,
    user_stack_end: usize,
) -> Result<(), VmError> {
    if kernel_start >= kernel_end || fdt_start >= fdt_end
        || user_code_start >= user_code_end || user_stack_start >= user_stack_end {
        return Err(VmError::InvalidRange);
    }
    if user_code_start < user_stack_end && user_stack_start < user_code_end {
        return Err(VmError::UserRegionsOverlap);
    }

    let base = kernel_start & !(L2_SPAN - 1);
    let end = base.checked_add(L2_SPAN).ok_or(VmError::InvalidRange)?;
    let range_in_window = |start: usize, finish: usize| {
        finish.checked_sub(1).map(|last| {
            in_window(start, base, end) && in_window(last, base, end)
        }).unwrap_or(false)
    };
    if !range_in_window(kernel_start, kernel_end)
        || !range_in_window(fdt_start, fdt_end)
        || !range_in_window(user_code_start, user_code_end)
        || !range_in_window(user_stack_start, user_stack_end) {
        return Err(VmError::OutsideBootstrapWindow);
    }

    let code_region = (user_code_start - base) / L1_SPAN;
    let code_region_end = (user_code_end - 1 - base) / L1_SPAN;
    let stack_region = (user_stack_start - base) / L1_SPAN;
    let stack_region_end = (user_stack_end - 1 - base) / L1_SPAN;
    if code_region != code_region_end || stack_region != stack_region_end {
        return Err(VmError::UserRegionTooLarge);
    }

    let root = core::ptr::addr_of_mut!(ROOT);
    let level1 = core::ptr::addr_of_mut!(LEVEL1);
    let code_l0 = core::ptr::addr_of_mut!(CODE_LEVEL0);
    let stack_l0 = core::ptr::addr_of_mut!(STACK_LEVEL0);
    let root_addr = root as usize;
    let level1_addr = level1 as usize;
    if root_addr & (PAGE - 1) != 0 || level1_addr & (PAGE - 1) != 0
        || (code_l0 as usize) & (PAGE - 1) != 0
        || (stack_l0 as usize) & (PAGE - 1) != 0 {
        return Err(VmError::MisalignedTable);
    }

    (*root).0.fill(0);
    (*level1).0.fill(0);
    (*code_l0).0.fill(0);
    (*stack_l0).0.fill(0);

    for i in 0..ENTRIES {
        let physical = base + i * L1_SPAN;
        (*level1).0[i] = leaf_pte(physical, SUPERVISOR_RWX);
    }

    let code_region_base = base + code_region * L1_SPAN;
    fill_level0(code_l0, code_region_base, user_code_start, user_code_end,
                user_stack_start, user_stack_end);
    (*level1).0[code_region] = table_pte(code_l0 as usize);

    if stack_region != code_region {
        let stack_region_base = base + stack_region * L1_SPAN;
        fill_level0(stack_l0, stack_region_base, user_code_start, user_code_end,
                    user_stack_start, user_stack_end);
        (*level1).0[stack_region] = table_pte(stack_l0 as usize);
    }

    let root_index = (base >> 30) & 0x1ff;
    (*root).0[root_index] = table_pte(level1_addr);

    let satp = SV39_MODE | ((root_addr >> 12) & SATP_PPN_MASK);
    asm!(
        "sfence.vma zero, zero",
        "csrw satp, {satp}",
        "sfence.vma zero, zero",
        "fence.i",
        satp = in(reg) satp,
        options(nostack)
    );
    let readback: usize;
    asm!("csrr {value}, satp", value = out(reg) readback, options(nomem, nostack));
    if readback != satp {
        return Err(VmError::SatpReadback);
    }
    Ok(())
}

pub fn active() -> bool {
    let satp: usize;
    unsafe { asm!("csrr {value}, satp", value = out(reg) satp, options(nomem, nostack)); }
    (satp >> 60) == 8
}
