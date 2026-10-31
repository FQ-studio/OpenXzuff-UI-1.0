//! Minimal fixed-capacity task lifecycle and round-robin scheduler core.
//! This module deliberately does not perform a CPU context switch; that requires
//! a saved register context and a dedicated assembly switch path.
pub const MAX_TASKS: usize = 16;
pub type TaskId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState { Empty, Ready, Running, Exited { status: isize } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
}
impl Task {
    const EMPTY: Self = Self { id: 0, state: TaskState::Empty };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskError { Full, InvalidId, NotRunning }

pub struct TaskManager {
    tasks: [Task; MAX_TASKS],
    current: Option<TaskId>,
    next_id: TaskId,
}
impl Default for TaskManager {
    fn default() -> Self { Self { tasks: [Task::EMPTY; MAX_TASKS], current: None, next_id: 1 } }
}
impl TaskManager {
    pub const fn new() -> Self { Self { tasks: [Task::EMPTY; MAX_TASKS], current: None, next_id: 1 } }

    pub fn create(&mut self) -> Result<TaskId, TaskError> {
        let slot = self.tasks.iter_mut().find(|t| t.state == TaskState::Empty).ok_or(TaskError::Full)?;
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        *slot = Task { id, state: TaskState::Ready };
        Ok(id)
    }

    pub fn current(&self) -> Option<TaskId> { self.current }

    pub fn state(&self, id: TaskId) -> Result<TaskState, TaskError> {
        self.tasks.iter().find(|t| t.id == id && t.state != TaskState::Empty)
            .map(|t| t.state).ok_or(TaskError::InvalidId)
    }

    /// Selects the next Ready task in round-robin order. This is scheduler
    /// bookkeeping only; the caller must switch CPU contexts before returning.
    pub fn schedule_next(&mut self) -> Option<TaskId> {
        if let Some(id) = self.current {
            if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id && t.state == TaskState::Running) {
                t.state = TaskState::Ready;
            }
        }
        let start = self.current.and_then(|id| self.tasks.iter().position(|t| t.id == id)).unwrap_or(usize::MAX);
        let chosen = (1..=MAX_TASKS).map(|step| if start == usize::MAX { step - 1 } else { (start + step) % MAX_TASKS })
            .find_map(|i| if self.tasks[i].state == TaskState::Ready { Some(i) } else { None });
        self.current = chosen.map(|i| self.tasks[i].id);
        if let Some(i) = chosen { self.tasks[i].state = TaskState::Running; }
        self.current
    }

    pub fn exit_current(&mut self, status: isize) -> Result<TaskId, TaskError> {
        let id = self.current.ok_or(TaskError::NotRunning)?;
        let task = self.tasks.iter_mut().find(|t| t.id == id && t.state == TaskState::Running)
            .ok_or(TaskError::NotRunning)?;
        task.state = TaskState::Exited { status };
        self.current = None;
        Ok(id)
    }

    pub fn ready_count(&self) -> usize { self.tasks.iter().filter(|t| t.state == TaskState::Ready).count() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_tasks_and_schedules_round_robin() {
        let mut m = TaskManager::new();
        let a = m.create().unwrap(); let b = m.create().unwrap();
        assert_eq!(m.schedule_next(), Some(a));
        assert_eq!(m.schedule_next(), Some(b));
        assert_eq!(m.schedule_next(), Some(a));
    }
    #[test]
    fn exit_marks_only_current_task_and_schedules_another() {
        let mut m = TaskManager::new();
        let a = m.create().unwrap(); let b = m.create().unwrap();
        assert_eq!(m.schedule_next(), Some(a));
        assert_eq!(m.exit_current(23), Ok(a));
        assert_eq!(m.state(a), Ok(TaskState::Exited { status: 23 }));
        assert_eq!(m.schedule_next(), Some(b));
    }
    #[test]
    fn full_table_fails_cleanly() {
        let mut m = TaskManager::new();
        for _ in 0..MAX_TASKS { m.create().unwrap(); }
        assert_eq!(m.create(), Err(TaskError::Full));
    }
    #[test]
    fn exit_without_running_task_is_rejected() {
        let mut m = TaskManager::new();
        assert_eq!(m.exit_current(0), Err(TaskError::NotRunning));
    }
}
