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
    fn alloc_in_range_only_returns_direct_mapped_pages() {
        let mut map = MemoryMap::new();
        map.add(0x8000_0000, 0x8000_8000).unwrap();
        let mut pages = PageAllocator::new();
        assert_eq!(pages.init(&map).unwrap(), 8);

        for _ in 0..4 {
            let page = pages.alloc_in_range(0x8000_2000, 0x8000_6000).unwrap();
            assert!((0x8000_2000..0x8000_6000).contains(&page));
        }
        assert_eq!(pages.alloc_in_range(0x8000_2000, 0x8000_6000), None);
        assert_eq!(pages.available_pages(), 4);
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
    fn allocation_state_tracks_alloc_and_free() {
        let mut map = MemoryMap::new(); map.add(0x1000, 0x3000).unwrap();
        let mut p = PageAllocator::new(); p.init(&map).unwrap();
        let addr = p.alloc().unwrap(); assert!(p.is_allocated(addr));
        p.free(addr).unwrap(); assert!(!p.is_allocated(addr));
    }
    #[test]
    fn ranged_allocation_never_returns_a_page_outside_the_window() {
        let mut map = MemoryMap::new(); map.add(0x1000, 0x9000).unwrap();
        let mut p = PageAllocator::new(); p.init(&map).unwrap();
        let page = p.alloc_in_range(0x4000, 0x8000).unwrap();
        assert!((0x4000..0x8000).contains(&page)); assert!(p.is_allocated(page));
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
        let strings = b"#address-cells\0#size-cells\0device_type\0reg\0timebase-frequency\0compatible\0";
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
        property(&mut structure, string_offset(strings, b"compatible"), b"test,first-uart\0ns16550a\0");

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

    pub(super) fn valid_fdt() -> Vec<u8> {
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
#[path = "../../src/syscall_abi.rs"]
pub mod syscall_abi;
#[path = "../../src/vm_policy.rs"]
pub mod vm_policy;

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

#[cfg(test)]
mod syscall_abi_tests {
    use super::syscall_abi;

    #[test]
    fn public_syscall_numbers_match_win32_gravity_abi_v1() {
        assert_eq!(
            [
                syscall_abi::EXIT,
                syscall_abi::SEND_MESSAGE,
                syscall_abi::RECEIVE_MESSAGE,
                syscall_abi::MAP_MEMORY,
                syscall_abi::YIELD,
                syscall_abi::UNMAP_MEMORY,
                syscall_abi::PROTECT_MEMORY,
            ],
            [0, 1, 2, 3, 4, 7, 8]
        );
    }

    #[test]
    fn errno_values_match_the_c_host_abi() {
        assert_eq!(
            [
                syscall_abi::EAGAIN,
                syscall_abi::EFAULT,
                syscall_abi::ENOMEM,
                syscall_abi::ENOSYS,
                syscall_abi::EINVAL,
            ],
            [-11, -14, -12, -38, -22]
        );
    }
}


#[cfg(test)]
mod fdt_compatible_tests {
    use super::fdt;
    #[test]
    fn compatible_matches_exact_nul_delimited_entries() {
        let blob = super::fdt_tests::valid_fdt();
        assert_eq!(fdt::has_compatible(&blob, b"test,first-uart"), Ok(true));
        assert_eq!(fdt::has_compatible(&blob, b"ns16550a"), Ok(true));
        assert_eq!(fdt::has_compatible(&blob, b"16550"), Ok(false));
        assert_eq!(fdt::has_compatible(&blob, b"missing,device"), Ok(false));
    }
    #[test]
    fn compatible_rejects_invalid_query_and_bad_header() {
        let blob = super::fdt_tests::valid_fdt();
        assert_eq!(fdt::has_compatible(&blob, b""), Err(fdt::FdtError::Structure));
        assert_eq!(fdt::has_compatible(&blob, b"bad\0query"), Err(fdt::FdtError::Structure));
        let mut corrupt = blob;
        corrupt[0..4].copy_from_slice(&0u32.to_be_bytes());
        assert_eq!(fdt::has_compatible(&corrupt, b"ns16550a"), Err(fdt::FdtError::Header));
    }
}

#[path = "../../src/process.rs"]
pub mod process;

#[cfg(test)]
mod process_tests {
    use super::process::{ProcessError, ProcessState, ProcessTable, MAX_PROCESSES};
    #[test]
    fn creates_process_with_parent_and_address_space_metadata() {
        let mut t=ProcessTable::new(); let p=t.create(None,0x1000,0x2000).unwrap();
        let c=t.create(Some(p),0x3000,0x4000).unwrap(); let i=t.get(c).unwrap();
        assert_eq!(i.parent,Some(p)); assert_eq!(i.state,ProcessState::Ready);
        assert_eq!(i.entry_pc,0x3000); assert_eq!(i.address_space_root,0x4000);
    }
    #[test]
    fn lifecycle_transitions_reject_invalid_edges() {
        let mut t=ProcessTable::new(); let id=t.create(None,1,2).unwrap();
        assert_eq!(t.transition(id,ProcessState::Running),Ok(()));
        assert_eq!(t.transition(id,ProcessState::Blocked),Ok(()));
        assert_eq!(t.transition(id,ProcessState::Ready),Ok(()));
        assert_eq!(t.transition(id,ProcessState::Exited),Err(ProcessError::InvalidTransition));
    }
    #[test]
    fn wait_reports_running_child_then_reaps_exit_status() {
        let mut t=ProcessTable::new(); let p=t.create(None,1,2).unwrap();
        let c=t.create(Some(p),3,4).unwrap();
        assert_eq!(t.wait(p,Some(c)),Err(ProcessError::WouldBlock));
        t.exit(c,37).unwrap(); assert_eq!(t.wait(p,Some(c)),Ok((c,37)));
        assert_eq!(t.get(c),Err(ProcessError::InvalidProcess));
    }
    #[test]
    fn stale_generation_cannot_access_reused_slot() {
        let mut t=ProcessTable::new(); let old=t.create(None,1,2).unwrap();
        t.exit(old,0).unwrap(); t.reap(old).unwrap(); let new=t.create(None,3,4).unwrap();
        assert_eq!(old.slot,new.slot); assert_ne!(old.generation,new.generation);
        assert_eq!(t.get(old),Err(ProcessError::InvalidProcess)); assert!(t.get(new).is_ok());
    }
    #[test]
    fn exit_orphans_children_and_capacity_is_bounded() {
        let mut t=ProcessTable::new(); let p=t.create(None,1,2).unwrap();
        let c=t.create(Some(p),3,4).unwrap(); t.exit(p,0).unwrap();
        assert_eq!(t.get(c).unwrap().parent,None);
        for _ in 2..MAX_PROCESSES { t.create(None,1,2).unwrap(); }
        assert_eq!(t.create(None,1,2),Err(ProcessError::Full));
    }
}

#[path = "../../src/handle.rs"]
pub mod handle;

#[cfg(test)]
mod handle_tests {
    use super::handle::{HandleError, HandleTable, ObjectKind, Rights, MAX_HANDLES};
    use super::process::ProcessId;

    fn process(slot: u16) -> ProcessId { ProcessId { slot, generation: 1 } }

    #[test]
    fn handle_lookup_enforces_owner_type_and_rights() {
        let mut table = HandleTable::new();
        let owner = process(0);
        let id = table.insert(owner, ObjectKind::File, 0x55, Rights::READ.union(Rights::WRITE)).unwrap();
        assert_eq!(table.get(owner, id, Some(ObjectKind::File), Rights::READ).unwrap().object_id, 0x55);
        assert_eq!(table.get(process(1), id, None, Rights::READ), Err(HandleError::WrongOwner));
        assert_eq!(table.get(owner, id, Some(ObjectKind::Device), Rights::READ), Err(HandleError::WrongType));
        assert_eq!(table.get(owner, id, None, Rights::EXECUTE), Err(HandleError::AccessDenied));
    }

    #[test]
    fn duplicate_can_only_reduce_rights_and_changes_owner() {
        let mut table = HandleTable::new();
        let a = process(0); let b = process(1);
        let source = table.insert(a, ObjectKind::Channel, 9, Rights::READ.union(Rights::WRITE).union(Rights::DUPLICATE)).unwrap();
        let copy = table.duplicate(a, source, b, Rights::READ).unwrap();
        assert_eq!(table.get(b, copy, Some(ObjectKind::Channel), Rights::READ).unwrap().object_id, 9);
        assert_eq!(table.get(b, copy, None, Rights::WRITE), Err(HandleError::AccessDenied));
        assert_eq!(table.get(a, copy, None, Rights::NONE), Err(HandleError::WrongOwner));
        assert_eq!(table.duplicate(a, source, b, Rights::ALL), Err(HandleError::AccessDenied));
    }

    #[test]
    fn close_invalidates_stale_generation_and_reuses_slot() {
        let mut table = HandleTable::new(); let owner = process(2);
        let old = table.insert(owner, ObjectKind::Event, 1, Rights::WAIT).unwrap();
        table.close(owner, old).unwrap();
        let new = table.insert(owner, ObjectKind::Event, 2, Rights::WAIT).unwrap();
        assert_eq!(old.slot, new.slot); assert_ne!(old.generation, new.generation);
        assert_eq!(table.get(owner, old, None, Rights::NONE), Err(HandleError::InvalidHandle));
    }

    #[test]
    fn handle_capacity_and_per_owner_count_are_bounded() {
        let mut table = HandleTable::new(); let owner = process(3);
        for i in 0..MAX_HANDLES { table.insert(owner, ObjectKind::Memory, i as u64, Rights::READ).unwrap(); }
        assert_eq!(table.open_count(owner), MAX_HANDLES);
        assert_eq!(table.insert(owner, ObjectKind::Memory, 0, Rights::READ), Err(HandleError::Full));
    }
}

#[path = "../../src/vfs.rs"]
pub mod vfs;

#[cfg(test)]
mod vfs_tests {
    use super::vfs::{NodeKind, RamFs, VfsError, MAX_FILE_BYTES, MAX_NODES};

    #[test]
    fn creates_and_resolves_nested_directories_and_files() {
        let mut fs=RamFs::new();
        fs.create_dir(b"/system").unwrap();
        fs.create_dir(b"/system/bin").unwrap();
        let (id,kind)=fs.lookup(b"/system/bin").unwrap();
        assert_eq!(kind,NodeKind::Directory);
        assert_eq!(fs.lookup(b"/system").unwrap().1,NodeKind::Directory);
        assert_eq!(fs.node_count(),3);
        assert!(id.generation>0);
    }

    #[test]
    fn file_reads_and_writes_support_offsets_and_eof() {
        let mut fs=RamFs::new(); fs.create_file(b"/hello").unwrap();
        assert_eq!(fs.write(b"/hello",2,b"world"),Ok(5));
        let mut data=[0u8;8];
        assert_eq!(fs.read(b"/hello",0,&mut data),Ok(7));
        assert_eq!(&data[..7],b"\0\0world");
        assert_eq!(fs.read(b"/hello",7,&mut data),Ok(0));
    }

    #[test]
    fn rejects_traversal_invalid_names_and_non_absolute_paths() {
        let mut fs=RamFs::new();
        assert_eq!(fs.lookup(b"relative"),Err(VfsError::InvalidPath));
        assert_eq!(fs.lookup(b"/../escape"),Err(VfsError::InvalidPath));
        assert_eq!(fs.lookup(b"/a\0b"),Err(VfsError::InvalidPath));
        assert_eq!(fs.create_file(b"/this-name-is-far-too-long-for-a-node"),Err(VfsError::NameTooLong));
        assert_eq!(fs.create_file(b"/trailing/"),Err(VfsError::InvalidPath));
        assert_eq!(fs.create_dir(b"/"),Err(VfsError::InvalidPath));
    }

    #[test]
    fn enforces_directory_file_type_and_duplicate_rules() {
        let mut fs=RamFs::new(); fs.create_dir(b"/dir").unwrap();
        assert_eq!(fs.create_file(b"/dir"),Err(VfsError::AlreadyExists));
        assert_eq!(fs.create_file(b"/missing/file"),Err(VfsError::NotFound));
        assert_eq!(fs.create_file(b"/dir/file").unwrap().generation,1);
        let mut out=[0u8;4];
        assert_eq!(fs.read(b"/dir",0,&mut out),Err(VfsError::IsDirectory));
    }

    #[test]
    fn remove_requires_empty_directory_and_invalidates_stale_inode() {
        let mut fs=RamFs::new();
        let dir=fs.create_dir(b"/d").unwrap();
        let file=fs.create_file(b"/d/f").unwrap();
        assert_eq!(fs.remove(b"/d"),Err(VfsError::NotEmpty));
        fs.remove(b"/d/f").unwrap();
        assert_eq!(fs.kind_by_id(file),Err(VfsError::InvalidNode));
        assert_eq!(fs.lookup(b"/d/f"),Err(VfsError::NotFound));
        fs.remove(b"/d").unwrap();
        let replacement=fs.create_file(b"/new").unwrap();
        assert_eq!(replacement.slot,dir.slot);
        assert_ne!(replacement.generation,dir.generation);
        assert_eq!(fs.kind_by_id(dir),Err(VfsError::InvalidNode));
        assert_eq!(fs.kind_by_id(replacement),Ok(NodeKind::File));
    }

    #[test]
    fn file_size_and_filesystem_capacity_are_bounded() {
        let mut fs=RamFs::new(); fs.create_file(b"/f").unwrap();
        let full=[7u8;MAX_FILE_BYTES];
        assert_eq!(fs.write(b"/f",0,&full),Ok(MAX_FILE_BYTES));
        assert_eq!(fs.write(b"/f",MAX_FILE_BYTES,&[1]),Err(VfsError::FileTooLarge));
        for i in 1..(MAX_NODES-1) {
            let name=[b'/', b'x', b'0'+(i%10) as u8, b'a'+(i/10) as u8];
            let _=fs.create_file(&name);
        }
        assert!(fs.node_count()<=MAX_NODES);
    }
}


#[cfg(test)]
mod vfs_inode_tests {
    use super::vfs::{RamFs, VfsError};

    #[test]
    fn generation_safe_inode_io_and_removal_reject_stale_handles() {
        let mut fs = RamFs::new();
        let inode = fs.create_file(b"/capability-file").unwrap();
        assert_eq!(fs.write_node(inode, 0, b"gravity"), Ok(7));
        let mut bytes = [0u8; 7];
        assert_eq!(fs.read_node(inode, 0, &mut bytes), Ok(7));
        assert_eq!(&bytes, b"gravity");
        fs.remove_node(inode).unwrap();
        assert_eq!(fs.read_node(inode, 0, &mut bytes), Err(VfsError::InvalidNode));
        assert_eq!(fs.write_node(inode, 0, b"stale"), Err(VfsError::InvalidNode));
        assert_eq!(fs.remove_node(inode), Err(VfsError::InvalidNode));
    }
}
