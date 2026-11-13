//! Coordinated process/task lifecycle layer.
//!
//! This module keeps the process table and scheduler state transitions in sync.
//! It is a kernel policy layer; it does not yet allocate independent page-table
//! roots or construct executable trap frames for dynamically spawned processes.
use crate::process::{ProcessError, ProcessId, ProcessInfo, ProcessState, ProcessTable};
use crate::scheduler::{Scheduler, SchedulerError, TaskId, MAX_TASKS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskManagerError {
    Process(ProcessError),
    Scheduler(SchedulerError),
    MissingTask,
    InvalidTransition,
}

pub struct TaskManager {
    processes: ProcessTable,
    scheduler: Scheduler,
    task_process: [Option<ProcessId>; MAX_TASKS],
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            processes: ProcessTable::new(),
            scheduler: Scheduler::new(),
            task_process: [None; MAX_TASKS],
        }
    }

    /// Create matching process metadata and a scheduler task.
    /// The current scheduler is fixed-capacity and does not yet recycle exited task slots.
    pub fn spawn(
        &mut self,
        parent: Option<ProcessId>,
        entry_pc: usize,
        address_space_root: usize,
    ) -> Result<ProcessId, TaskManagerError> {
        let task = self.scheduler.create_task().map_err(TaskManagerError::Scheduler)?;
        let process = match self.processes.create(parent, entry_pc, address_space_root) {
            Ok(id) => id,
            Err(error) => return Err(TaskManagerError::Process(error)),
        };
        self.task_process[task.0] = Some(process);
        Ok(process)
    }

    pub fn process_for_task(&self, task: TaskId) -> Result<ProcessId, TaskManagerError> {
        self.task_process.get(task.0).copied().flatten().ok_or(TaskManagerError::MissingTask)
    }

    pub fn get(&self, process: ProcessId) -> Result<ProcessInfo, TaskManagerError> {
        self.processes.get(process).map_err(TaskManagerError::Process)
    }

    /// Select the next runnable task and mirror Running/Ready into process metadata.
    pub fn schedule_next_process(&mut self) -> Result<Option<ProcessId>, TaskManagerError> {
        let previous = self.scheduler.current();
        let next = self.scheduler.schedule_next();
        if let Some(old) = previous {
            if let Some(process) = self.task_process[old.0] {
                if self.processes.get(process).map_err(TaskManagerError::Process)?.state == ProcessState::Running {
                    self.processes.transition(process, ProcessState::Ready)
                        .map_err(TaskManagerError::Process)?;
                }
            }
        }
        if let Some(task) = next {
            let process = self.process_for_task(task)?;
            self.processes.transition(process, ProcessState::Running)
                .map_err(TaskManagerError::Process)?;
            Ok(Some(process))
        } else {
            Ok(None)
        }
    }

    pub fn yield_current_process(&mut self) -> Result<Option<ProcessId>, TaskManagerError> {
        self.schedule_next_process()
    }

    pub fn block_current(&mut self) -> Result<Option<ProcessId>, TaskManagerError> {
        let task = self.scheduler.current().ok_or(TaskManagerError::Scheduler(SchedulerError::NoCurrentTask))?;
        let process = self.process_for_task(task)?;
        self.processes.transition(process, ProcessState::Blocked)
            .map_err(TaskManagerError::Process)?;
        let next = self.scheduler.block_current().map_err(TaskManagerError::Scheduler)?;
        if let Some(task) = next {
            let next_process = self.process_for_task(task)?;
            self.processes.transition(next_process, ProcessState::Running)
                .map_err(TaskManagerError::Process)?;
            Ok(Some(next_process))
        } else {
            Ok(None)
        }
    }

    pub fn wake(&mut self, process: ProcessId) -> Result<(), TaskManagerError> {
        let task = self.task_process.iter().position(|entry| *entry == Some(process))
            .ok_or(TaskManagerError::MissingTask)?;
        self.scheduler.wake(TaskId(task)).map_err(TaskManagerError::Scheduler)?;
        self.processes.transition(process, ProcessState::Ready)
            .map_err(TaskManagerError::Process)
    }

    pub fn exit_current_with_code(&mut self, code: i32) -> Result<Option<ProcessId>, TaskManagerError> {
        let task = self.scheduler.current().ok_or(TaskManagerError::Scheduler(SchedulerError::NoCurrentTask))?;
        let process = self.process_for_task(task)?;
        self.processes.exit(process, code).map_err(TaskManagerError::Process)?;
        let next = self.scheduler.exit_current().map_err(TaskManagerError::Scheduler)?;
        if let Some(task) = next {
            let next_process = self.process_for_task(task)?;
            self.processes.transition(next_process, ProcessState::Running)
                .map_err(TaskManagerError::Process)?;
            Ok(Some(next_process))
        } else {
            Ok(None)
        }
    }

    /// Wait for an exited child and reap its process-table slot.
    pub fn wait(
        &mut self,
        parent: ProcessId,
        child: Option<ProcessId>,
    ) -> Result<(ProcessId, i32), TaskManagerError> {
        let (reaped, code) = self.processes.wait(parent, child).map_err(TaskManagerError::Process)?;
        for entry in &mut self.task_process {
            if *entry == Some(reaped) { *entry = None; }
        }
        Ok((reaped, code))
    }


    /// Runtime-compatible task creation: creates both scheduler state and process metadata.
    pub fn create_runtime_task(&mut self) -> Result<TaskId, TaskManagerError> {
        let process = self.spawn(None, 0, 0)?;
        self.task_process.iter().position(|entry| *entry == Some(process))
            .map(TaskId)
            .ok_or(TaskManagerError::MissingTask)
    }

    pub fn runtime_current(&self) -> Option<TaskId> { self.scheduler.current() }

    pub fn runtime_state(&self, task: TaskId) -> Result<crate::scheduler::TaskState, SchedulerError> {
        self.scheduler.state(task)
    }

    pub fn runtime_schedule_next(&mut self) -> Option<TaskId> {
        let process = self.schedule_next_process().ok().flatten()?;
        self.task_process.iter().position(|entry| *entry == Some(process)).map(TaskId)
    }

    pub fn runtime_yield_current(&mut self) -> Result<Option<TaskId>, SchedulerError> {
        let process = self.schedule_next_process().map_err(|_| SchedulerError::InvalidTask)?;
        Ok(process.and_then(|id| self.task_process.iter().position(|entry| *entry == Some(id)).map(TaskId)))
    }

    pub fn runtime_exit_current(&mut self) -> Result<Option<TaskId>, SchedulerError> {
        self.runtime_exit_current_with_code(0)
    }

    pub fn runtime_exit_current_with_code(&mut self, code: i32) -> Result<Option<TaskId>, SchedulerError> {
        let process = self.exit_current_with_code(code).map_err(|_| SchedulerError::InvalidTask)?;
        Ok(process.and_then(|id| self.task_process.iter().position(|entry| *entry == Some(id)).map(TaskId)))
    }

    pub fn active_processes(&self) -> usize { self.processes.active_count() }
}
impl Default for TaskManager {
    fn default() -> Self { Self::new() }
}
