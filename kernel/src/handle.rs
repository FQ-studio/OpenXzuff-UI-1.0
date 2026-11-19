//! Generation-safe, per-process kernel handle table.
//! Handles are capabilities: every lookup checks owner, object type and rights.
use crate::process::ProcessId;

pub const MAX_HANDLES: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleId { pub slot: u16, pub generation: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectKind { File, Directory, Event, Channel, Device, Process, Thread, Memory }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights(pub u16);
impl Rights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const WAIT: Self = Self(1 << 3);
    pub const CONTROL: Self = Self(1 << 4);
    pub const DUPLICATE: Self = Self(1 << 5);
    pub const ALL: Self = Self((1 << 6) - 1);
    pub const fn contains(self, required: Self) -> bool { self.0 & required.0 == required.0 }
    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleInfo {
    pub id: HandleId,
    pub owner: ProcessId,
    pub kind: ObjectKind,
    pub object_id: u64,
    pub rights: Rights,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleError { Full, InvalidHandle, WrongOwner, WrongType, AccessDenied }

#[derive(Clone, Copy)]
struct Slot { generation: u32, info: Option<HandleInfo> }
impl Slot { const EMPTY: Self = Self { generation: 1, info: None }; }

pub struct HandleTable { slots: [Slot; MAX_HANDLES] }
impl HandleTable {
    pub const fn new() -> Self { Self { slots: [Slot::EMPTY; MAX_HANDLES] } }

    pub fn insert(&mut self, owner: ProcessId, kind: ObjectKind, object_id: u64, rights: Rights) -> Result<HandleId, HandleError> {
        let index = self.slots.iter().position(|s| s.info.is_none()).ok_or(HandleError::Full)?;
        let slot = &mut self.slots[index];
        let id = HandleId { slot: index as u16, generation: slot.generation };
        slot.info = Some(HandleInfo { id, owner, kind, object_id, rights });
        Ok(id)
    }

    pub fn get(&self, caller: ProcessId, id: HandleId, expected: Option<ObjectKind>, required: Rights) -> Result<HandleInfo, HandleError> {
        let slot = self.slots.get(id.slot as usize).ok_or(HandleError::InvalidHandle)?;
        if slot.generation != id.generation { return Err(HandleError::InvalidHandle); }
        let info = slot.info.ok_or(HandleError::InvalidHandle)?;
        if info.owner != caller { return Err(HandleError::WrongOwner); }
        if expected.map_or(false, |kind| kind != info.kind) { return Err(HandleError::WrongType); }
        if !info.rights.contains(required) { return Err(HandleError::AccessDenied); }
        Ok(info)
    }

    pub fn close(&mut self, caller: ProcessId, id: HandleId) -> Result<(), HandleError> {
        let info = self.get(caller, id, None, Rights::NONE)?;
        let slot = &mut self.slots[info.id.slot as usize];
        slot.info = None;
        slot.generation = slot.generation.wrapping_add(1).max(1);
        Ok(())
    }

    pub fn duplicate(&mut self, caller: ProcessId, id: HandleId, target: ProcessId, rights: Rights) -> Result<HandleId, HandleError> {
        let source = self.get(caller, id, None, Rights::DUPLICATE)?;
        if !source.rights.contains(rights) { return Err(HandleError::AccessDenied); }
        self.insert(target, source.kind, source.object_id, rights)
    }

    pub fn open_count(&self, owner: ProcessId) -> usize {
        self.slots.iter().filter(|slot| slot.info.map_or(false, |info| info.owner == owner)).count()
    }
}
impl Default for HandleTable { fn default() -> Self { Self::new() } }
