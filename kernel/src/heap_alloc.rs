//! Fixed-capacity first-fit heap allocator for the Gravity Kernel.
//! The allocator owns metadata only; the caller supplies a stable, writable arena.
//! Allocations are 16-byte aligned. At most 128 blocks are tracked.
const ALIGN: usize = 16;
const MAX_BLOCKS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapError { InvalidArena, Capacity, InvalidPointer, DoubleFree }

#[derive(Clone, Copy)]
struct Block { offset: usize, size: usize, used: bool }
impl Block { const EMPTY: Self = Self { offset: 0, size: 0, used: false }; }

/// Fixed-metadata heap allocator. The arena must remain valid and exclusively
/// owned by this allocator until it is reinitialized. External locking is
/// required before using it concurrently from multiple harts.
pub struct HeapAllocator {
    base: *mut u8,
    len: usize,
    blocks: [Block; MAX_BLOCKS],
    count: usize,
    initialized: bool,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self { base: core::ptr::null_mut(), len: 0,
            blocks: [Block::EMPTY; MAX_BLOCKS], count: 0, initialized: false }
    }

    /// # Safety
    /// base must point to a writable, 16-byte-aligned arena of len bytes that
    /// remains alive and is not accessed by another allocator for this lifetime.
    pub unsafe fn init(&mut self, base: *mut u8, len: usize) -> Result<(), HeapError> {
        if base.is_null() || (base as usize) % ALIGN != 0 || len < ALIGN {
            return Err(HeapError::InvalidArena);
        }
        let usable = len & !(ALIGN - 1);
        if usable < ALIGN { return Err(HeapError::InvalidArena); }
        self.base = base;
        self.len = usable;
        self.blocks = [Block::EMPTY; MAX_BLOCKS];
        self.blocks[0] = Block { offset: 0, size: usable, used: false };
        self.count = 1;
        self.initialized = true;
        Ok(())
    }

    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        if !self.initialized || size == 0 { return None; }
        let rounded = size.checked_add(ALIGN - 1)? & !(ALIGN - 1);
        for i in 0..self.count {
            if self.blocks[i].used || self.blocks[i].size < rounded { continue; }
            let old_size = self.blocks[i].size;
            if old_size > rounded {
                if self.count == MAX_BLOCKS { return None; }
                // Copy the split offset before mutably indexing the block array.
                // This keeps the borrow unambiguous for the Rust compiler.
                let split_offset = self.blocks[i].offset + rounded;
                self.blocks[self.count] = Block {
                    offset: split_offset,
                    size: old_size - rounded,
                    used: false,
                };
                self.count += 1;
            }
            self.blocks[i].size = rounded;
            self.blocks[i].used = true;
            return Some(unsafe { self.base.add(self.blocks[i].offset) });
        }
        None
    }

    pub fn free(&mut self, ptr: *mut u8) -> Result<(), HeapError> {
        if !self.initialized || ptr.is_null() { return Err(HeapError::InvalidPointer); }
        let address = ptr as usize;
        let base = self.base as usize;
        let offset = address.checked_sub(base).ok_or(HeapError::InvalidPointer)?;
        if offset >= self.len || offset % ALIGN != 0 {
            return Err(HeapError::InvalidPointer);
        }
        let index = self.blocks[..self.count].iter()
            .position(|b| b.offset == offset)
            .ok_or(HeapError::InvalidPointer)?;
        if !self.blocks[index].used { return Err(HeapError::DoubleFree); }
        self.blocks[index].used = false;
        self.coalesce();
        Ok(())
    }

    fn coalesce(&mut self) {
        let mut i = 0;
        while i < self.count {
            let mut j = i + 1;
            while j < self.count {
                if self.blocks[i].offset + self.blocks[i].size == self.blocks[j].offset
                    && !self.blocks[i].used && !self.blocks[j].used
                {
                    self.blocks[i].size += self.blocks[j].size;
                    for k in j..self.count - 1 { self.blocks[k] = self.blocks[k + 1]; }
                    self.count -= 1;
                } else if self.blocks[j].offset + self.blocks[j].size == self.blocks[i].offset
                    && !self.blocks[i].used && !self.blocks[j].used
                {
                    self.blocks[j].size += self.blocks[i].size;
                    self.blocks[j].offset = self.blocks[i].offset;
                    for k in i..self.count - 1 { self.blocks[k] = self.blocks[k + 1]; }
                    self.count -= 1;
                    i = 0;
                    break;
                } else { j += 1; }
            }
            if i < self.count { i += 1; }
        }
    }

    pub fn capacity(&self) -> usize { if self.initialized { self.len } else { 0 } }
    pub fn used_bytes(&self) -> usize {
        self.blocks[..self.count].iter().filter(|b| b.used).map(|b| b.size).sum()
    }
    pub fn available_bytes(&self) -> usize { self.capacity().saturating_sub(self.used_bytes()) }
}
