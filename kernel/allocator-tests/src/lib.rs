//! Host-side tests for the exact allocator modules used by the RV64 kernel.
pub mod memory {
    #[path = "../../src/memory.rs"]
    mod source;
    pub use source::*;
}
pub mod page_alloc {
    #[path = "../../src/page_alloc.rs"]
    mod source;
    pub use source::*;
}

#[cfg(test)]
mod tests {
    use super::{memory::MemoryMap, page_alloc::{AllocError, PageAllocator, PAGE_SIZE}};

    #[test]
    fn reserved_pages_are_never_allocated() {
        let mut map = MemoryMap::new();
        map.add(0x8000_0000, 0x8000_8000).unwrap();
        map.reserve(0x8000_2000, 0x8000_4000).unwrap();
        let mut p = PageAllocator::new();
        assert_eq!(p.init(&map).unwrap(), 6);
        while let Some(addr) = p.alloc() {
            assert!(addr < 0x8000_2000 || addr >= 0x8000_4000);
        }
        assert_eq!(p.available_pages(), 0);
        assert_eq!(p.alloc(), None);
    }

    #[test]
    fn alloc_free_restores_capacity() {
        let mut map = MemoryMap::new();
        map.add(0x1000, 0x5000).unwrap();
        let mut p = PageAllocator::new();
        assert_eq!(p.init(&map).unwrap(), 4);
        let before = p.available_pages();
        let addr = p.alloc().unwrap();
        assert_eq!(addr % PAGE_SIZE, 0);
        assert_eq!(p.available_pages(), before - 1);
        p.free(addr).unwrap();
        assert_eq!(p.available_pages(), before);
    }

    #[test]
    fn double_free_is_rejected() {
        let mut map = MemoryMap::new();
        map.add(0x1000, 0x3000).unwrap();
        let mut p = PageAllocator::new();
        p.init(&map).unwrap();
        let addr = p.alloc().unwrap();
        p.free(addr).unwrap();
        assert_eq!(p.free(addr), Err(AllocError::DoubleFree));
    }

    #[test]
    fn unaligned_and_unmanaged_addresses_are_rejected() {
        let mut map = MemoryMap::new();
        map.add(0x1000, 0x3000).unwrap();
        let mut p = PageAllocator::new();
        p.init(&map).unwrap();
        assert_eq!(p.free(0x1001), Err(AllocError::InvalidAddress));
        assert_eq!(p.free(0x9000), Err(AllocError::InvalidAddress));
    }

    #[test]
    fn overlapping_ram_ranges_do_not_duplicate_pages() {
        let mut map = MemoryMap::new();
        map.add(0x1000, 0x5000).unwrap();
        map.add(0x3000, 0x7000).unwrap();
        let mut p = PageAllocator::new();
        assert_eq!(p.init(&map).unwrap(), 6);
        assert_eq!(p.managed_pages(), 6);
    }
}
