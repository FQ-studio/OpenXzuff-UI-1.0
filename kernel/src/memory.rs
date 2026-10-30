//! Fixed-capacity physical memory map used during early boot.
#![allow(dead_code)]

const MAX_RANGES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range { pub start: usize, pub end: usize }

#[derive(Debug)]
pub struct MemoryMap {
    ram: [Option<Range>; MAX_RANGES],
    reserved: [Option<Range>; MAX_RANGES],
    ram_len: usize,
    reserved_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError { InvalidRange, Capacity }

impl MemoryMap {
    pub const fn new() -> Self {
        Self {
            ram: [None; MAX_RANGES],
            reserved: [None; MAX_RANGES],
            ram_len: 0,
            reserved_len: 0,
        }
    }

    /// Add a half-open RAM range [start, end).
    pub fn add(&mut self, start: usize, end: usize) -> Result<(), MapError> {
        if start >= end { return Err(MapError::InvalidRange); }
        if self.ram_len == MAX_RANGES { return Err(MapError::Capacity); }
        self.ram[self.ram_len] = Some(Range { start, end });
        self.ram_len += 1;
        Ok(())
    }

    /// Mark a half-open range [start, end) unavailable to the allocator.
    pub fn reserve(&mut self, start: usize, end: usize) -> Result<(), MapError> {
        if start >= end { return Err(MapError::InvalidRange); }
        if self.reserved_len == MAX_RANGES { return Err(MapError::Capacity); }
        self.reserved[self.reserved_len] = Some(Range { start, end });
        self.reserved_len += 1;
        Ok(())
    }

    pub fn ram_ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.ram[..self.ram_len].iter().filter_map(|r| *r)
    }

    pub fn reserved_ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.reserved[..self.reserved_len].iter().filter_map(|r| *r)
    }

    pub fn ram_count(&self) -> usize { self.ram_len }
}
