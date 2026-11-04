//! Fixed-capacity cooperative RV64 kernel-thread scheduler.
/// Kernel threads only: entries must be trusted kernel functions, not user pointers.
/// User-mode TaskOpen must wait for validated user address spaces and trap-frame switching.
use crate::context::{gravity_context_switch, SavedContext};

pub const MAX_KERNEL_TASKS: usize = 16;
const STACK_SIZE: usize = 4096;
pub type KernelTaskEntry = extern "C" fn() -> !;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenError { Full, SchedulerNotStarted }

#[repr(align(16))]
struct TaskStack([u8; STACK_SIZE]);

static mut STACKS: [TaskStack; MAX_KERNEL_TASKS] = [
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]), TaskStack([0; STACK_SIZE]),
];
static mut CONTEXTS: [SavedContext; MAX_KERNEL_TASKS] = [SavedContext {
    ra: 0, sp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
    s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
}; MAX_KERNEL_TASKS];
static mut ACTIVE: [bool; MAX_KERNEL_TASKS] = [false; MAX_KERNEL_TASKS];
static mut SCHEDULER_CONTEXT: SavedContext = SavedContext {
    ra: 0, sp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
    s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
};
static mut CURRENT: usize = MAX_KERNEL_TASKS - 1;
static mut INITIALIZED: bool = false;

extern "C" fn task_a() -> ! {
    loop { crate::scheduler_log(b'A'); yield_now(); }
}
extern "C" fn task_b() -> ! {
    loop { crate::scheduler_log(b'B'); yield_now(); }
}

unsafe fn install_task(slot: usize, entry: KernelTaskEntry) {
    let base = core::ptr::addr_of_mut!(STACKS[slot].0) as *mut u8 as usize;
    let top = (base + STACK_SIZE) & !15usize;
    CONTEXTS[slot] = SavedContext { ra: entry as usize, sp: top, ..SavedContext::default() };
    ACTIVE[slot] = true;
}

/// Open a trusted kernel thread in a free slot; returns its zero-based slot ID.
/// This is not the user-mode syscall implementation: passing an untrusted
/// userspace address here would be unsafe.
pub unsafe fn task_open(entry: KernelTaskEntry) -> Result<usize, OpenError> {
    if !INITIALIZED { return Err(OpenError::SchedulerNotStarted); }
    for slot in 0..MAX_KERNEL_TASKS {
        if !ACTIVE[slot] {
            install_task(slot, entry);
            return Ok(slot);
        }
    }
    Err(OpenError::Full)
}

/// Start the scheduler with two built-in demonstration kernel threads.
pub unsafe fn start() -> ! {
    if !INITIALIZED {
        install_task(0, task_a);
        install_task(1, task_b);
        INITIALIZED = true;
    }
    CURRENT = MAX_KERNEL_TASKS - 1;
    yield_from_scheduler()
}

unsafe fn next_active(after: usize) -> usize {
    for step in 1..=MAX_KERNEL_TASKS {
        let slot = (after + step) % MAX_KERNEL_TASKS;
        if ACTIVE[slot] { return slot; }
    }
    // The built-in tasks are permanent, so this indicates corrupted state.
    loop { core::hint::spin_loop(); }
}

unsafe fn yield_from_scheduler() -> ! {
    loop {
        let next = next_active(CURRENT);
        CURRENT = next;
        gravity_context_switch(core::ptr::addr_of_mut!(SCHEDULER_CONTEXT),
            core::ptr::addr_of!(CONTEXTS[next]));
    }
}

/// Yield cooperatively to the next active kernel thread.
pub fn yield_now() {
    unsafe {
        let current = CURRENT;
        let next = next_active(current);
        CURRENT = next;
        gravity_context_switch(core::ptr::addr_of_mut!(CONTEXTS[current]),
            core::ptr::addr_of!(CONTEXTS[next]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_and_stack_alignment_are_fixed() {
        assert_eq!(MAX_KERNEL_TASKS, 16);
        assert_eq!(STACK_SIZE, 4096);
        assert_eq!(core::mem::align_of::<TaskStack>(), 16);
        assert_eq!(core::mem::size_of::<SavedContext>(), 14 * core::mem::size_of::<usize>());
    }
}
