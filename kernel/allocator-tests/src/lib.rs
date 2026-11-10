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

    fn valid_fdt_with_timebase(timebase: Option<&[u8]>) -> Vec<u8> {
        let strings = b"#address-cells\0#size-cells\0device_type\0reg\0timebase-frequency\0";
        let address_cells = string_offset(strings, b"#address-cells");
        let size_cells = string_offset(strings, b"#size-cells");
        let device_type = string_offset(strings, b"device_type");
        let reg = string_offset(strings, b"reg");
        let timebase_frequency = string_offset(strings, b"timebase-frequency");

        let mut structure = Vec::new();
        begin_node(&mut structure, b"");
        property(&mut structure, address_cells, &2u32.to_be_bytes());
        property(&mut structure, size_cells, &2u32.to_be_bytes());
        begin_node(&mut structure, b"cpus");
        if let Some(value) = timebase {
            property(&mut structure, timebase_frequency, value);
        }
        push_be32(&mut structure, END_NODE);
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

    fn valid_fdt() -> Vec<u8> {
        valid_fdt_with_timebase(Some(&10_000_000u32.to_be_bytes()))
    }

    #[test]
    fn fdt_timebase_frequency_reads_32_and_64_bit_values() {
        let narrow = valid_fdt();
        assert_eq!(fdt::timebase_frequency(&narrow), Ok(10_000_000));

        let wide_value = 0x1_0000_0000u64;
        let wide = valid_fdt_with_timebase(Some(&wide_value.to_be_bytes()));
        assert_eq!(fdt::timebase_frequency(&wide), Ok(wide_value as usize));
    }

    #[test]
    fn fdt_timebase_frequency_rejects_zero_bad_width_and_missing_property() {
        let zero = valid_fdt_with_timebase(Some(&0u32.to_be_bytes()));
        assert_eq!(fdt::timebase_frequency(&zero), Err(FdtError::Timebase));

        let bad_width = valid_fdt_with_timebase(Some(&[0, 1, 2]));
        assert_eq!(fdt::timebase_frequency(&bad_width), Err(FdtError::Timebase));

        let missing = valid_fdt_with_timebase(None);
        assert_eq!(fdt::timebase_frequency(&missing), Err(FdtError::Timebase));
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

#[path = "../../src/scheduler.rs"]
pub mod scheduler;

#[cfg(test)]
mod scheduler_tests {
    use super::scheduler::{Scheduler, SchedulerError, TaskId, TaskState, MAX_TASKS};

    #[test]
    fn round_robin_yield_rotates_ready_tasks() {
        let mut s = Scheduler::new();
        let a=s.create_task().unwrap(); let b=s.create_task().unwrap(); let c=s.create_task().unwrap();
        assert_eq!(s.schedule_next(),Some(a));
        assert_eq!(s.yield_current(),Ok(Some(b)));
        assert_eq!(s.yield_current(),Ok(Some(c)));
        assert_eq!(s.yield_current(),Ok(Some(a)));
        assert_eq!(s.state(a),Ok(TaskState::Running));
        assert_eq!(s.state(b),Ok(TaskState::Ready));
        assert_eq!(s.state(c),Ok(TaskState::Ready));
    }
    #[test]
    fn blocked_task_is_skipped_until_woken() {
        let mut s=Scheduler::new(); let a=s.create_task().unwrap(); let b=s.create_task().unwrap();
        assert_eq!(s.schedule_next(),Some(a));
        assert_eq!(s.block_current(),Ok(Some(b)));
        assert_eq!(s.state(a),Ok(TaskState::Blocked));
        assert_eq!(s.wake(a),Ok(()));
        assert_eq!(s.yield_current(),Ok(Some(a)));
    }
    #[test]
    fn exited_task_is_not_scheduled_again() {
        let mut s=Scheduler::new(); let a=s.create_task().unwrap(); let b=s.create_task().unwrap();
        assert_eq!(s.schedule_next(),Some(a));
        assert_eq!(s.exit_current(),Ok(Some(b)));
        assert_eq!(s.exit_current(),Ok(None));
        assert_eq!(s.state(a),Ok(TaskState::Exited));
        assert_eq!(s.schedule_next(),None);
    }
    #[test]
    fn sole_ready_task_can_yield_to_itself() {
        let mut s=Scheduler::new(); let a=s.create_task().unwrap();
        assert_eq!(s.schedule_next(),Some(a));
        assert_eq!(s.yield_current(),Ok(Some(a)));
    }
    #[test]
    fn rejects_invalid_operations_and_capacity_overflow() {
        let mut s=Scheduler::new();
        assert_eq!(s.yield_current(),Err(SchedulerError::NoCurrentTask));
        assert_eq!(s.state(TaskId(MAX_TASKS)),Err(SchedulerError::InvalidTask));
        let a=s.create_task().unwrap();
        assert_eq!(s.wake(a),Err(SchedulerError::NotBlocked));
        for _ in 1..MAX_TASKS { s.create_task().unwrap(); }
        assert_eq!(s.create_task(),Err(SchedulerError::Full));
    }
}


#[path = "../../src/ipc.rs"]
pub mod ipc;

#[cfg(test)]
mod ipc_tests {
    use super::ipc::{IpcError, MailboxSet, IPC_QUEUE_CAPACITY};
    use super::scheduler::{TaskId, MAX_TASKS};

    #[test]
    fn mailboxes_deliver_fifo_messages_with_sender_identity() {
        let mut mailboxes = MailboxSet::new();
        let sender = TaskId(2);
        let receiver = TaskId(0);
        mailboxes.send(sender, receiver, 0x1111).unwrap();
        mailboxes.send(TaskId(1), receiver, 0x2222).unwrap();
        assert_eq!(mailboxes.pending(receiver), Ok(2));
        assert_eq!(mailboxes.receive(receiver).unwrap(), Some(super::ipc::Message { sender, payload: 0x1111 }));
        assert_eq!(mailboxes.receive(receiver).unwrap(), Some(super::ipc::Message { sender: TaskId(1), payload: 0x2222 }));
        assert_eq!(mailboxes.receive(receiver).unwrap(), None);
    }

    #[test]
    fn mailboxes_are_isolated_per_receiver() {
        let mut mailboxes = MailboxSet::new();
        mailboxes.send(TaskId(0), TaskId(1), 0xAAAA).unwrap();
        mailboxes.send(TaskId(0), TaskId(2), 0xBBBB).unwrap();
        assert_eq!(mailboxes.receive(TaskId(1)).unwrap().unwrap().payload, 0xAAAA);
        assert_eq!(mailboxes.receive(TaskId(2)).unwrap().unwrap().payload, 0xBBBB);
        assert_eq!(mailboxes.pending(TaskId(0)), Ok(0));
    }

    #[test]
    fn bounded_mailbox_reports_full_and_recovers_after_receive() {
        let mut mailboxes = MailboxSet::new();
        for value in 0..IPC_QUEUE_CAPACITY {
            mailboxes.send(TaskId(1), TaskId(0), value).unwrap();
        }
        assert_eq!(mailboxes.send(TaskId(1), TaskId(0), 99), Err(IpcError::QueueFull));
        assert_eq!(mailboxes.receive(TaskId(0)).unwrap().unwrap().payload, 0);
        mailboxes.send(TaskId(1), TaskId(0), 99).unwrap();
        assert_eq!(mailboxes.pending(TaskId(0)), Ok(IPC_QUEUE_CAPACITY));
    }

    #[test]
    fn mailbox_rejects_task_ids_outside_scheduler_capacity() {
        let mut mailboxes = MailboxSet::new();
        let invalid = TaskId(MAX_TASKS);
        assert_eq!(mailboxes.send(TaskId(0), invalid, 1), Err(IpcError::InvalidTask));
        assert_eq!(mailboxes.receive(invalid), Err(IpcError::InvalidTask));
        assert_eq!(mailboxes.pending(invalid), Err(IpcError::InvalidTask));
    }
}
