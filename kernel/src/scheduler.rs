//! Allocation-free round-robin task-state scheduler.
//! The RV64 runtime uses this policy from its timer trap to select saved task contexts.
pub const MAX_TASKS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState { Unused, Ready, Running, Blocked, Exited }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerError { Full, InvalidTask, NotBlocked, NoCurrentTask }

#[derive(Clone, Copy)]
struct Slot { state: TaskState }
impl Slot { const EMPTY: Self = Self { state: TaskState::Unused }; }

pub struct Scheduler {
    slots: [Slot; MAX_TASKS],
    current: Option<TaskId>,
    count: usize,
}
impl Scheduler {
    pub const fn new() -> Self {
        Self { slots: [Slot::EMPTY; MAX_TASKS], current: None, count: 0 }
    }
    pub fn create_task(&mut self) -> Result<TaskId, SchedulerError> {
        if self.count == MAX_TASKS { return Err(SchedulerError::Full); }
        let id = TaskId(self.count);
        self.slots[id.0].state = TaskState::Ready;
        self.count += 1;
        Ok(id)
    }
    pub fn state(&self, id: TaskId) -> Result<TaskState, SchedulerError> {
        self.slots.get(id.0).filter(|s| s.state != TaskState::Unused)
            .map(|s| s.state).ok_or(SchedulerError::InvalidTask)
    }
    pub const fn current(&self) -> Option<TaskId> { self.current }
    pub fn schedule_next(&mut self) -> Option<TaskId> {
        if let Some(id) = self.current {
            if self.slots[id.0].state == TaskState::Running {
                self.slots[id.0].state = TaskState::Ready;
            }
        }
        let start = self.current.map_or(0, |id| (id.0 + 1) % MAX_TASKS);
        for n in 0..MAX_TASKS {
            let i = (start + n) % MAX_TASKS;
            if self.slots[i].state == TaskState::Ready {
                self.slots[i].state = TaskState::Running;
                self.current = Some(TaskId(i));
                return self.current;
            }
        }
        self.current = None;
        None
    }
    pub fn yield_current(&mut self) -> Result<Option<TaskId>, SchedulerError> {
        self.require_running()?;
        Ok(self.schedule_next())
    }
    pub fn block_current(&mut self) -> Result<Option<TaskId>, SchedulerError> {
        let id = self.require_running()?;
        self.slots[id.0].state = TaskState::Blocked;
        Ok(self.schedule_next())
    }
    pub fn exit_current(&mut self) -> Result<Option<TaskId>, SchedulerError> {
        let id = self.require_running()?;
        self.slots[id.0].state = TaskState::Exited;
        Ok(self.schedule_next())
    }
    pub fn wake(&mut self, id: TaskId) -> Result<(), SchedulerError> {
        let slot = self.slots.get_mut(id.0)
            .filter(|s| s.state != TaskState::Unused)
            .ok_or(SchedulerError::InvalidTask)?;
        if slot.state != TaskState::Blocked { return Err(SchedulerError::NotBlocked); }
        slot.state = TaskState::Ready;
        Ok(())
    }
    fn require_running(&self) -> Result<TaskId, SchedulerError> {
        let id = self.current.ok_or(SchedulerError::NoCurrentTask)?;
        if self.slots[id.0].state != TaskState::Running {
            return Err(SchedulerError::NoCurrentTask);
        }
        Ok(id)
    }
}
impl Default for Scheduler { fn default() -> Self { Self::new() } }

pub fn smoke_test() -> bool {
    let mut s = Scheduler::new();
    let a = match s.create_task() { Ok(v) => v, Err(_) => return false };
    let b = match s.create_task() { Ok(v) => v, Err(_) => return false };
    let c = match s.create_task() { Ok(v) => v, Err(_) => return false };
    s.schedule_next() == Some(a)
        && s.yield_current() == Ok(Some(b))
        && s.block_current() == Ok(Some(c))
        && s.exit_current() == Ok(Some(a))
        && s.wake(b).is_ok()
        && s.yield_current() == Ok(Some(b))
        && s.exit_current() == Ok(Some(a))
        && s.state(a) == Ok(TaskState::Running)
        && s.state(b) == Ok(TaskState::Exited)
        && s.state(c) == Ok(TaskState::Exited)
}
