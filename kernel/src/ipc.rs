//! Fixed-capacity, allocation-free per-task message queues.
//!
//! The initial implementation is deliberately single-hart: callers mask S-mode
//! interrupts while mutating the queues. It transports one machine word per
//! message and never dereferences a user-supplied pointer.
use crate::scheduler::{TaskId, MAX_TASKS};

pub const IPC_QUEUE_CAPACITY: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub sender: TaskId,
    pub payload: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    InvalidTask,
    QueueFull,
}

#[derive(Clone, Copy)]
struct Mailbox {
    entries: [Option<Message>; IPC_QUEUE_CAPACITY],
    head: usize,
    len: usize,
}

impl Mailbox {
    const EMPTY: Self = Self {
        entries: [None; IPC_QUEUE_CAPACITY],
        head: 0,
        len: 0,
    };

    fn push(&mut self, message: Message) -> Result<(), IpcError> {
        if self.len == IPC_QUEUE_CAPACITY {
            return Err(IpcError::QueueFull);
        }
        let tail = (self.head + self.len) % IPC_QUEUE_CAPACITY;
        self.entries[tail] = Some(message);
        self.len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Option<Message> {
        if self.len == 0 {
            return None;
        }
        let message = self.entries[self.head].take();
        self.head = (self.head + 1) % IPC_QUEUE_CAPACITY;
        self.len -= 1;
        message
    }
}

/// Mailboxes are indexed by kernel task ID. This storage is also host-testable.
pub struct MailboxSet {
    queues: [Mailbox; MAX_TASKS],
}

impl MailboxSet {
    pub const fn new() -> Self {
        Self { queues: [Mailbox::EMPTY; MAX_TASKS] }
    }

    pub fn send(&mut self, sender: TaskId, receiver: TaskId, payload: usize) -> Result<(), IpcError> {
        if sender.0 >= MAX_TASKS || receiver.0 >= MAX_TASKS {
            return Err(IpcError::InvalidTask);
        }
        self.queues[receiver.0].push(Message { sender, payload })
    }

    pub fn receive(&mut self, receiver: TaskId) -> Result<Option<Message>, IpcError> {
        if receiver.0 >= MAX_TASKS {
            return Err(IpcError::InvalidTask);
        }
        Ok(self.queues[receiver.0].pop())
    }

    pub fn pending(&self, receiver: TaskId) -> Result<usize, IpcError> {
        if receiver.0 >= MAX_TASKS {
            return Err(IpcError::InvalidTask);
        }
        Ok(self.queues[receiver.0].len)
    }
}

impl Default for MailboxSet {
    fn default() -> Self { Self::new() }
}

static mut MAILBOXES: MailboxSet = MailboxSet::new();
const SSTATUS_SIE: usize = 1 << 1;

struct InterruptGuard {
    previous_sstatus: usize,
}

impl InterruptGuard {
    #[inline]
    fn enter() -> Self {
        #[cfg(target_arch = "riscv64")]
        {
            let previous: usize;
            unsafe {
                core::arch::asm!(
                    "csrrc {previous}, sstatus, {mask}",
                    previous = out(reg) previous,
                    mask = in(reg) SSTATUS_SIE,
                    options(nostack)
                );
            }
            Self { previous_sstatus: previous }
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            Self { previous_sstatus: 0 }
        }
    }
}

impl Drop for InterruptGuard {
    #[inline]
    fn drop(&mut self) {
        #[cfg(target_arch = "riscv64")]
        if self.previous_sstatus & SSTATUS_SIE != 0 {
            unsafe {
                core::arch::asm!(
                    "csrs sstatus, {mask}",
                    mask = in(reg) SSTATUS_SIE,
                    options(nostack)
                );
            }
        }
    }
}

/// Send one scalar word to a task's FIFO mailbox. No user pointers are accepted.
pub fn send(sender: TaskId, receiver: TaskId, payload: usize) -> Result<(), IpcError> {
    let _interrupt_guard = InterruptGuard::enter();
    unsafe { (*core::ptr::addr_of_mut!(MAILBOXES)).send(sender, receiver, payload) }
}

/// Receive the oldest queued message for a task; this operation never blocks.
pub fn receive(receiver: TaskId) -> Result<Option<Message>, IpcError> {
    let _interrupt_guard = InterruptGuard::enter();
    unsafe { (*core::ptr::addr_of_mut!(MAILBOXES)).receive(receiver) }
}
