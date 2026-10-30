//! Fixed-capacity physical page allocator for early RV64 bring-up.
//! 4 KiB pages; at most 16,384 pages (64 MiB) are managed.
use crate::memory::{MemoryMap, Range};

pub const PAGE_SIZE: usize = 4096;
const MAX_PAGES: usize = 16384;
const MAX_RANGES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocError { InvalidAddress, DoubleFree, Capacity }

pub struct PageAllocator {
    free_stack: [usize; MAX_PAGES],
    free_len: usize,
    ledger: [usize; MAX_PAGES],
    ledger_len: usize,
}

impl PageAllocator {
    pub const fn new() -> Self {
        Self { free_stack: [0; MAX_PAGES], free_len: 0,
            ledger: [0; MAX_PAGES], ledger_len: 0 }
    }

    /// Build the free-page pool, excluding every reserved-range overlap.
    /// The fixed pool intentionally caps managed RAM at 64 MiB.
    pub fn init(&mut self, map: &MemoryMap) -> Result<usize, AllocError> {
        self.free_len = 0;
        self.ledger_len = 0;
        let mut ram: [Option<Range>; MAX_RANGES] = [None; MAX_RANGES];
        let mut reserved: [Option<Range>; MAX_RANGES] = [None; MAX_RANGES];
        let mut nr = 0usize;
        let mut ns = 0usize;
        for r in map.ram_ranges() {
            if nr == MAX_RANGES { return Err(AllocError::Capacity); }
            ram[nr] = Some(r); nr += 1;
        }
        for r in map.reserved_ranges() {
            if ns == MAX_RANGES { return Err(AllocError::Capacity); }
            reserved[ns] = Some(r); ns += 1;
        }
        for r in ram[..nr].iter().flatten() {
            let start = r.start.checked_add(PAGE_SIZE - 1)
                .ok_or(AllocError::InvalidAddress)? & !(PAGE_SIZE - 1);
            let end = r.end & !(PAGE_SIZE - 1);
            let mut addr = start;
            while addr < end {
                let next = addr.checked_add(PAGE_SIZE).ok_or(AllocError::InvalidAddress)?;
                let blocked = reserved[..ns].iter().flatten()
                    .any(|x| addr < x.end && x.start < next);
                if !blocked {
                    // Avoid duplicate entries if firmware describes overlapping RAM.
                    if !self.ledger[..self.ledger_len].contains(&addr) {
                        if self.ledger_len == MAX_PAGES { return Ok(self.free_len); }
                        self.ledger[self.ledger_len] = addr;
                        self.ledger_len += 1;
                        self.free_stack[self.free_len] = addr;
                        self.free_len += 1;
                    }
                }
                addr = next;
            }
        }
        Ok(self.free_len)
    }

    pub fn alloc(&mut self) -> Option<usize> {
        if self.free_len == 0 { return None; }
        self.free_len -= 1;
        Some(self.free_stack[self.free_len])
    }

    pub fn free(&mut self, addr: usize) -> Result<(), AllocError> {
        if addr & (PAGE_SIZE - 1) != 0 || !self.ledger[..self.ledger_len].contains(&addr) {
            return Err(AllocError::InvalidAddress);
        }
        if self.free_stack[..self.free_len].contains(&addr) {
            return Err(AllocError::DoubleFree);
        }
        if self.free_len == MAX_PAGES { return Err(AllocError::Capacity); }
        self.free_stack[self.free_len] = addr;
        self.free_len += 1;
        Ok(())
    }

    pub fn available_pages(&self) -> usize { self.free_len }
    pub fn managed_pages(&self) -> usize { self.ledger_len }
}
