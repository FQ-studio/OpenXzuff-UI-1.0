//! Host-side tests for the exact allocator and FDT modules used by the RV64 kernel.
#[path = "../../src/fdt.rs"]
pub mod fdt;
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

#[cfg(test)]
mod fdt_tests {
    use super::{fdt::{self, FdtError}, memory::{MemoryMap, Range}};

    const BEGIN_NODE: u32 = 1;
    const END_NODE: u32 = 2;
    const PROP: u32 = 3;
    const END: u32 = 9;

    fn push_be32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_be_bytes());
    }

    fn push_be64(out: &mut Vec<u8>, value: u64) {
        out.extend_from_slice(&value.to_be_bytes());
    }

    fn pad4(out: &mut Vec<u8>) {
        while out.len() % 4 != 0 { out.push(0); }
    }

    fn begin_node(out: &mut Vec<u8>, name: &[u8]) {
        push_be32(out, BEGIN_NODE);
        out.extend_from_slice(name);
        out.push(0);
        pad4(out);
    }

    fn property(out: &mut Vec<u8>, name_offset: u32, value: &[u8]) {
        push_be32(out, PROP);
        push_be32(out, value.len() as u32);
        push_be32(out, name_offset);
        out.extend_from_slice(value);
        pad4(out);
    }

    fn string_offset(strings: &[u8], name: &[u8]) -> u32 {
        strings.windows(name.len() + 1)
            .position(|window| window == [name, &[0]].concat())
            .expect("property name exists in FDT strings block") as u32
    }

    fn valid_fdt() -> Vec<u8> {
        let strings = b"#address-cells\0#size-cells\0device_type\0reg\0";
        let address_cells = string_offset(strings, b"#address-cells");
        let size_cells = string_offset(strings, b"#size-cells");
        let device_type = string_offset(strings, b"device_type");
        let reg = string_offset(strings, b"reg");

        let mut structure = Vec::new();
        begin_node(&mut structure, b"");
        property(&mut structure, address_cells, &2u32.to_be_bytes());
        property(&mut structure, size_cells, &2u32.to_be_bytes());
        begin_node(&mut structure, b"memory@80000000");
        property(&mut structure, device_type, b"memory\0");

        let mut memory_reg = Vec::new();
        for cell in [1u32, 0x8000_0000, 1, 0x0002_0000] {
            push_be32(&mut memory_reg, cell);
        }
        property(&mut structure, reg, &memory_reg);
        push_be32(&mut structure, END_NODE);
        push_be32(&mut structure, END_NODE);
        push_be32(&mut structure, END);

        const RESERVE_OFFSET: u32 = 40;
        const STRUCT_OFFSET: u32 = 72;
        let strings_offset = STRUCT_OFFSET + structure.len() as u32;
        let total_size = strings_offset + strings.len() as u32;

        let mut blob = Vec::with_capacity(total_size as usize);
        for value in [
            0xd00d_feed,
            total_size,
            STRUCT_OFFSET,
            strings_offset,
            RESERVE_OFFSET,
            17,
            16,
            0,
            strings.len() as u32,
            structure.len() as u32,
        ] {
            push_be32(&mut blob, value);
        }

        // One memreserve entry plus the required zero/zero terminator.
        push_be64(&mut blob, 0x1_8000_2000);
        push_be64(&mut blob, 0x1000);
        push_be64(&mut blob, 0);
        push_be64(&mut blob, 0);
        blob.extend_from_slice(&structure);
        blob.extend_from_slice(strings);
        assert_eq!(blob.len(), total_size as usize);
        blob
    }

    #[test]
    fn fdt_parser_extracts_ram_and_memreserve_ranges() {
        let blob = valid_fdt();
        let mut map = MemoryMap::new();
        assert_eq!(fdt::parse_memory(&blob, &mut map), Ok(blob.len()));
        assert_eq!(
            map.ram_ranges().collect::<Vec<_>>(),
            vec![Range { start: 0x1_8000_0000, end: 0x2_8002_0000 }]
        );
        assert_eq!(
            map.reserved_ranges().collect::<Vec<_>>(),
            vec![Range { start: 0x1_8000_2000, end: 0x1_8000_3000 }]
        );
    }

    #[test]
    fn fdt_parser_rejects_bad_magic_and_out_of_bounds_structure() {
        let mut bad_magic = valid_fdt();
        bad_magic[0..4].copy_from_slice(&0u32.to_be_bytes());
        assert_eq!(fdt::parse_memory(&bad_magic, &mut MemoryMap::new()), Err(FdtError::Header));

        let mut bad_structure = valid_fdt();
        bad_structure[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(fdt::parse_memory(&bad_structure, &mut MemoryMap::new()), Err(FdtError::Bounds));
    }

    #[test]
    fn fdt_parser_rejects_truncated_reservation_map() {
        let mut truncated = valid_fdt();
        let reserve_offset = (truncated.len() - 8) as u32;
        truncated[16..20].copy_from_slice(&reserve_offset.to_be_bytes());
        assert_eq!(
            fdt::parse_memory(&truncated, &mut MemoryMap::new()),
            Err(FdtError::Reservation)
        );
    }
}
