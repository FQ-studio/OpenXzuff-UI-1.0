//! Fixed-capacity process lifecycle table for the RV64 Gravity kernel.
//!
//! Kernel-owned metadata only: this is not yet a process loader or VM engine.
//! IDs carry generations so a reaped slot cannot validate a stale process ID.
pub const MAX_PROCESSES: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessId { pub slot: u16, pub generation: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState { Empty, Ready, Running, Blocked, Exited }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessInfo {
    pub id: ProcessId, pub parent: Option<ProcessId>, pub state: ProcessState,
    pub entry_pc: usize, pub address_space_root: usize, pub exit_code: Option<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessError { Full, InvalidProcess, InvalidTransition, NoChild, WouldBlock }
#[derive(Clone, Copy)]
struct Slot { generation: u32, info: Option<ProcessInfo> }
impl Slot { const EMPTY: Self = Self { generation: 1, info: None }; }

pub struct ProcessTable { slots: [Slot; MAX_PROCESSES] }
impl ProcessTable {
    pub const fn new() -> Self { Self { slots: [Slot::EMPTY; MAX_PROCESSES] } }
    fn slot(&self, id: ProcessId) -> Result<&Slot, ProcessError> {
        let slot = self.slots.get(id.slot as usize).ok_or(ProcessError::InvalidProcess)?;
        if slot.generation != id.generation || slot.info.is_none() { return Err(ProcessError::InvalidProcess); }
        Ok(slot)
    }
    fn slot_mut(&mut self, id: ProcessId) -> Result<&mut Slot, ProcessError> {
        let slot = self.slots.get_mut(id.slot as usize).ok_or(ProcessError::InvalidProcess)?;
        if slot.generation != id.generation || slot.info.is_none() { return Err(ProcessError::InvalidProcess); }
        Ok(slot)
    }
    pub fn create(&mut self, parent: Option<ProcessId>, entry_pc: usize, address_space_root: usize) -> Result<ProcessId, ProcessError> {
        if let Some(parent_id) = parent {
            if self.get(parent_id)?.state == ProcessState::Exited { return Err(ProcessError::InvalidTransition); }
        }
        let index = self.slots.iter().position(|slot| slot.info.is_none()).ok_or(ProcessError::Full)?;
        let slot = &mut self.slots[index];
        let id = ProcessId { slot: index as u16, generation: slot.generation };
        slot.info = Some(ProcessInfo { id, parent, state: ProcessState::Ready, entry_pc, address_space_root, exit_code: None });
        Ok(id)
    }
    pub fn get(&self, id: ProcessId) -> Result<ProcessInfo, ProcessError> {
        self.slot(id)?.info.ok_or(ProcessError::InvalidProcess)
    }
    pub fn transition(&mut self, id: ProcessId, next: ProcessState) -> Result<(), ProcessError> {
        let info = self.slot_mut(id)?.info.as_mut().ok_or(ProcessError::InvalidProcess)?;
        let valid = matches!((info.state, next),
            (ProcessState::Ready, ProcessState::Running) |
            (ProcessState::Ready, ProcessState::Blocked) |
            (ProcessState::Running, ProcessState::Ready) |
            (ProcessState::Running, ProcessState::Blocked) |
            (ProcessState::Blocked, ProcessState::Ready));
        if !valid { return Err(ProcessError::InvalidTransition); }
        info.state = next;
        Ok(())
    }
    pub fn exit(&mut self, id: ProcessId, code: i32) -> Result<(), ProcessError> {
        let info = self.slot_mut(id)?.info.as_mut().ok_or(ProcessError::InvalidProcess)?;
        if !matches!(info.state, ProcessState::Ready | ProcessState::Running | ProcessState::Blocked) {
            return Err(ProcessError::InvalidTransition);
        }
        info.state = ProcessState::Exited;
        info.exit_code = Some(code);
        for slot in &mut self.slots {
            if let Some(child) = slot.info.as_mut() { if child.parent == Some(id) { child.parent = None; } }
        }
        Ok(())
    }
    /// Non-blocking wait; None selects the first exited child.
    pub fn wait(&mut self, parent: ProcessId, child: Option<ProcessId>) -> Result<(ProcessId, i32), ProcessError> {
        self.slot(parent)?;
        let mut has_child = false;
        for slot in &self.slots {
            if let Some(info) = slot.info {
                if info.parent == Some(parent) && child.map_or(true, |wanted| wanted == info.id) {
                    has_child = true;
                    if info.state == ProcessState::Exited {
                        let id = info.id;
                        let code = info.exit_code.ok_or(ProcessError::InvalidTransition)?;
                        self.reap(id)?;
                        return Ok((id, code));
                    }
                }
            }
        }
        if has_child { Err(ProcessError::WouldBlock) } else { Err(ProcessError::NoChild) }
    }
    pub fn reap(&mut self, id: ProcessId) -> Result<(), ProcessError> {
        let slot = self.slot_mut(id)?;
        let info = slot.info.ok_or(ProcessError::InvalidProcess)?;
        if info.state != ProcessState::Exited { return Err(ProcessError::InvalidTransition); }
        slot.info = None;
        slot.generation = slot.generation.wrapping_add(1).max(1);
        Ok(())
    }
    pub fn active_count(&self) -> usize { self.slots.iter().filter(|slot| slot.info.is_some()).count() }
}
impl Default for ProcessTable { fn default() -> Self { Self::new() } }
