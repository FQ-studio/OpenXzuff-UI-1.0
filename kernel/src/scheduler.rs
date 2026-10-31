//! Minimal cooperative round-robin scheduler for the RV64 S-mode kernel.
//! This smoke-test scheduler supports a fixed pool of independent tasks.
//! Context switching is cooperative: a running task yields explicitly.
use core::arch::global_asm;

const TASKS: usize = 16;
const STACK_SIZE: usize = 4096;
const TICKS_PER_TASK: usize = 3;

#[repr(C)]
#[derive(Clone, Copy)]
struct Context {
    ra: usize,
    sp: usize,
    s: [usize; 12], // s0-s11
}
impl Context {
    const fn empty() -> Self { Self { ra: 0, sp: 0, s: [0; 12] } }
}

#[derive(Clone, Copy)]
#[repr(align(16))]
struct TaskStack([u8; STACK_SIZE]);

static mut CONTEXTS: [Context; TASKS] = [Context::empty(); TASKS];
static mut STACKS: [TaskStack; TASKS] = [TaskStack([0; STACK_SIZE]); TASKS];
static mut CURRENT: usize = 0;
static mut FINISHED: [bool; TASKS] = [false; TASKS];
static mut TICKS: [usize; TASKS] = [0; TASKS];
static mut BOOT_CONTEXT: Context = Context::empty();

global_asm!(
    ".section .text", ".align 2",
    ".global gravity_context_switch", ".type gravity_context_switch, @function",
    "gravity_context_switch:",
    "sd ra, 0(a0)", "sd sp, 8(a0)", "sd s0, 16(a0)", "sd s1, 24(a0)",
    "sd s2, 32(a0)", "sd s3, 40(a0)", "sd s4, 48(a0)", "sd s5, 56(a0)",
    "sd s6, 64(a0)", "sd s7, 72(a0)", "sd s8, 80(a0)", "sd s9, 88(a0)",
    "sd s10, 96(a0)", "sd s11, 104(a0)",
    "ld ra, 0(a1)", "ld sp, 8(a1)", "ld s0, 16(a1)", "ld s1, 24(a1)",
    "ld s2, 32(a1)", "ld s3, 40(a1)", "ld s4, 48(a1)", "ld s5, 56(a1)",
    "ld s6, 64(a1)", "ld s7, 72(a1)", "ld s8, 80(a1)", "ld s9, 88(a1)",
    "ld s10, 96(a1)", "ld s11, 104(a1)", "ret",
    ".global gravity_task_trampoline", ".type gravity_task_trampoline, @function",
    "gravity_task_trampoline:", "mv a0, s0", "call gravity_scheduler_task_entry",
    "1: wfi", "j 1b",
);

extern "C" {
    fn gravity_context_switch(old: *mut Context, new: *const Context);
    fn gravity_task_trampoline() -> !;
    fn gravity_scheduler_log_task(task: usize, tick: usize);
    fn gravity_scheduler_log_start(task_count: usize);
}

pub fn run_smoke() -> bool {
    unsafe {
        CURRENT = 0;
        FINISHED = [false; TASKS];
        TICKS = [0; TASKS];
        gravity_scheduler_log_start(TASKS);
        for id in 0..TASKS {
            let stack_top = core::ptr::addr_of_mut!(STACKS[id].0).cast::<u8>()
                .add(STACK_SIZE) as usize;
            CONTEXTS[id] = Context {
                ra: gravity_task_trampoline as usize,
                sp: stack_top & !0xf,
                s: [0; 12],
            };
            CONTEXTS[id].s[0] = id;
        }
        gravity_context_switch(
            core::ptr::addr_of_mut!(BOOT_CONTEXT),
            core::ptr::addr_of!(CONTEXTS[0]).cast::<Context>(),
        );
        (0..TASKS).all(|id| TICKS[id] == TICKS_PER_TASK && FINISHED[id])
    }
}

#[no_mangle]
extern "C" fn gravity_scheduler_task_entry(task_id: usize) -> ! {
    for tick in 1..=TICKS_PER_TASK {
        unsafe {
            TICKS[task_id] = tick;
            gravity_scheduler_log_task(task_id, tick);
        }
        yield_now();
    }
    unsafe {
        FINISHED[task_id] = true;
        if let Some(next) = next_unfinished(task_id) {
            CURRENT = next;
            gravity_context_switch(
                core::ptr::addr_of_mut!(CONTEXTS[task_id]),
                core::ptr::addr_of!(CONTEXTS[next]).cast::<Context>(),
            );
        } else {
            gravity_context_switch(
                core::ptr::addr_of_mut!(CONTEXTS[task_id]),
                core::ptr::addr_of!(BOOT_CONTEXT).cast::<Context>(),
            );
        }
    }
    loop { core::hint::spin_loop(); }
}

fn next_unfinished(current: usize) -> Option<usize> {
    unsafe {
        (1..=TASKS).map(|offset| (current + offset) % TASKS)
            .find(|&candidate| !FINISHED[candidate])
    }
}

fn yield_now() {
    unsafe {
        let current = CURRENT;
        if let Some(next) = next_unfinished(current) {
            CURRENT = next;
            gravity_context_switch(
                core::ptr::addr_of_mut!(CONTEXTS[current]),
                core::ptr::addr_of!(CONTEXTS[next]).cast::<Context>(),
            );
        }
    }
}
