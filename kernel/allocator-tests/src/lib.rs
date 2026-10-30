//! Host-side tests for the exact allocator modules used by the RV64 kernel.
#[path = "../../src/memory.rs"]
pub mod memory;
#[path = "../../src/page_alloc.rs"]
pub mod page_alloc;
#[path = "../../src/heap_alloc.rs"]
pub mod heap_alloc;

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


#[cfg(test)]
mod heap_tests {
    use super::heap_alloc::{HeapAllocator, HeapError};

    #[repr(align(16))]
    struct Arena([u8; 1024]);

    #[test]
    fn heap_alloc_free_and_reuse() {
        let mut arena = Arena([0; 1024]);
        let mut heap = HeapAllocator::new();
        unsafe { heap.init(arena.0.as_mut_ptr(), arena.0.len()).unwrap(); }
        let before = heap.available_bytes();
        let a = heap.alloc(100).unwrap();
        assert_eq!((a as usize) % 16, 0);
        assert!(heap.available_bytes() < before);
        heap.free(a).unwrap();
        assert_eq!(heap.available_bytes(), before);
    }

    #[test]
    fn heap_coalesces_adjacent_freed_blocks() {
        let mut arena = Arena([0; 1024]);
        let mut heap = HeapAllocator::new();
        unsafe { heap.init(arena.0.as_mut_ptr(), arena.0.len()).unwrap(); }
        let a = heap.alloc(128).unwrap();
        let b = heap.alloc(128).unwrap();
        heap.free(a).unwrap();
        heap.free(b).unwrap();
        assert_eq!(heap.available_bytes(), 1024);
        assert!(heap.alloc(900).is_some());
    }

    #[test]
    fn heap_rejects_invalid_and_double_free() {
        let mut arena = Arena([0; 1024]);
        let mut heap = HeapAllocator::new();
        unsafe { heap.init(arena.0.as_mut_ptr(), arena.0.len()).unwrap(); }
        let a = heap.alloc(32).unwrap();
        assert_eq!(heap.free(unsafe { a.add(1) }), Err(HeapError::InvalidPointer));
        heap.free(a).unwrap();
        assert_eq!(heap.free(a), Err(HeapError::DoubleFree));
    }

    #[test]
    fn heap_exhaustion_returns_none() {
        let mut arena = Arena([0; 1024]);
        let mut heap = HeapAllocator::new();
        unsafe { heap.init(arena.0.as_mut_ptr(), arena.0.len()).unwrap(); }
        assert!(heap.alloc(2048).is_none());
        assert!(heap.alloc(0).is_none());
    }
}
