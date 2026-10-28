//! Cooperative RV64 kernel-thread scheduler using the real assembly context switch.
//! This is kernel-mode cooperative scheduling; it is not yet user-mode preemption.
use crate::context::{gravity_context_switch, SavedContext};

const TASKS: usize = 2;
const STACK_SIZE: usize = 4096;

type Entry = extern "C" fn() -> !;

#[repr(align(16))]
struct TaskStack([u8; STACK_SIZE]);

static mut STACKS: [TaskStack; TASKS] = [
    TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]),
];
static mut CONTEXTS: [SavedContext; TASKS] = [SavedContext {
    ra: 0, sp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
    s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
}; TASKS];
static mut SCHEDULER_CONTEXT: SavedContext = SavedContext {
    ra: 0, sp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
    s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
};
static mut CURRENT: usize = 0;
static mut INITIALIZED: bool = false;

extern "C" fn task_a() -> ! {
    loop {
        crate::scheduler_log(b'A');
        yield_now();
    }
}
extern "C" fn task_b() -> ! {
    loop {
        crate::scheduler_log(b'B');
        yield_now();
    }
}

/// Initialize two kernel tasks and transfer control to the first one.
/// Each task yields cooperatively; the assembly routine saves/restores ra, sp,
/// and s0-s11 according to SavedContext's C ABI layout.
pub unsafe fn start() -> ! {
    if !INITIALIZED {
        let entries: [Entry; TASKS] = [task_a, task_b];
        for i in 0..TASKS {
            let base = core::ptr::addr_of_mut!(STACKS[i].0) as *mut u8 as usize;
            let top = (base + STACK_SIZE) & !15usize;
            CONTEXTS[i] = SavedContext {
                ra: entries[i] as usize, sp: top,
                ..SavedContext::default()
            };
        }
        INITIALIZED = true;
    }
    CURRENT = TASKS - 1;
    yield_from_scheduler()
}

unsafe fn yield_from_scheduler() -> ! {
    loop {
        let next = (CURRENT + 1) % TASKS;
        CURRENT = next;
        gravity_context_switch(
            core::ptr::addr_of_mut!(SCHEDULER_CONTEXT),
            core::ptr::addr_of!(CONTEXTS[next]),
        );
    }
}

/// Cooperative yield from the currently running kernel task.
pub fn yield_now() {
    unsafe {
        let current = CURRENT;
        let next = (current + 1) % TASKS;
        CURRENT = next;
        gravity_context_switch(
            core::ptr::addr_of_mut!(CONTEXTS[current]),
            core::ptr::addr_of!(CONTEXTS[next]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn task_stack_and_context_capacity_are_fixed() {
        assert_eq!(TASKS, 2);
        assert_eq!(STACK_SIZE, 4096);
        assert_eq!(core::mem::align_of::<TaskStack>(), 16);
        assert_eq!(core::mem::size_of::<SavedContext>(), 14 * core::mem::size_of::<usize>());
    }
}
