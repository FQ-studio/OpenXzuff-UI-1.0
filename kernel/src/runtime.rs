//! Mixed S/U-mode runtime with SBI timer preemption and trap-frame context switching.
use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::scheduler::{Scheduler, TaskId};
use crate::trap::TrapFrame;

const TASK_COUNT: usize = 3;
const IPC_REQUEST: usize = 0x4752_4156;
const IPC_REPLY: usize = 0x5245_504c;
const STACK_SIZE: usize = 16 * 1024;
const SBI_EXT_TIME: usize = 0x5449_4d45;
const SBI_FID_SET_TIMER: usize = 0;
static TIMER_QUANTUM: AtomicUsize = AtomicUsize::new(100_000);
const SIE_STIE: usize = 1 << 5;
const SSTATUS_SIE: usize = 1 << 1;

#[repr(align(16))]
struct TaskStack([u8; STACK_SIZE]);

static mut TASK0_STACK: TaskStack = TaskStack([0; STACK_SIZE]);
static mut TASK1_STACK: TaskStack = TaskStack([0; STACK_SIZE]);
static mut SCHEDULER: Scheduler = Scheduler::new();
static mut TASK_FRAMES: [usize; TASK_COUNT] = [0; TASK_COUNT];
static TIMER_INTERRUPTS: AtomicUsize = AtomicUsize::new(0);
static TASK_A_TICKS: AtomicUsize = AtomicUsize::new(0);
static TASK_B_TICKS: AtomicUsize = AtomicUsize::new(0);
static TIMER_TEST_REPORTED: AtomicBool = AtomicBool::new(false);
static TASK_A_CONTEXT_OK: AtomicBool = AtomicBool::new(false);
static TASK_B_CONTEXT_OK: AtomicBool = AtomicBool::new(false);
static USER_TASK_EXITED: AtomicBool = AtomicBool::new(false);
static USER_EXIT_STATUS: AtomicUsize = AtomicUsize::new(usize::MAX);
static IPC_REQUEST_RECEIVED: AtomicBool = AtomicBool::new(false);
static USER_IPC_REPLY_RECEIVED: AtomicBool = AtomicBool::new(false);
static USER_IPC_EMPTY_OBSERVED: AtomicBool = AtomicBool::new(false);
static USER_TIMER_PREEMPTED: AtomicBool = AtomicBool::new(false);

extern "C" {
    fn __gravity_s_trap_resume(frame: *mut TrapFrame) -> !;
}

fn read_time() -> usize {
    let value: usize;
    unsafe { asm!("rdtime {value}", value = out(reg) value, options(nomem, nostack)); }
    value
}

fn set_timer(deadline: usize) -> Result<(), usize> {
    let error: usize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") deadline => error,
            lateout("a1") _,
            in("a6") SBI_FID_SET_TIMER,
            in("a7") SBI_EXT_TIME,
            options(nostack)
        );
    }
    if error == 0 { Ok(()) } else { Err(error) }
}

pub fn configure_timebase(frequency_hz: usize) -> Result<usize, ()> {
    let quantum = frequency_hz / 100;
    if quantum == 0 { return Err(()); }
    TIMER_QUANTUM.store(quantum, Ordering::Release);
    Ok(quantum)
}

fn activate_address_space(task: TaskId) {
    let expects_user = task.0 == 2;
    let switched = if expects_user {
        crate::vm::switch_to_user()
    } else {
        crate::vm::switch_to_kernel()
    };
    if !switched || crate::vm::current_is_user() != expects_user {
        crate::console_write("Task address-space switch: FAIL\n");
        crate::halt();
    }
}

fn arm_next_timer() {
    let quantum = TIMER_QUANTUM.load(Ordering::Acquire);
    if quantum == 0 || set_timer(read_time().wrapping_add(quantum)).is_err() {
        crate::console_write("SBI TIME set_timer: FAIL\n");
        crate::halt();
    }
}

unsafe fn make_initial_frame(stack: *mut TaskStack, entry: extern "C" fn() -> !) -> *mut TrapFrame {
    let base = core::ptr::addr_of_mut!((*stack).0).cast::<u8>();
    let top = base.add(STACK_SIZE) as usize;
    let frame = (top - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
    let gp: usize;
    asm!("mv {gp}, gp", gp = out(reg) gp, options(nomem, nostack));
    frame.write(TrapFrame::new(entry as *const () as usize, top, gp));
    frame
}

extern "C" fn task_a() -> ! {
    let mut register_marker = 0xA11C_E000usize;
    let mut stack_canary = 0xA11C_E55Ausize;
    let stack_canary_ptr = &mut stack_canary as *mut usize;
    loop {
        unsafe { asm!("addi s2, s2, 0", inout("s2") register_marker, options(nomem, nostack)); }
        if register_marker != 0xA11C_E000
            || unsafe { core::ptr::read_volatile(stack_canary_ptr) } != 0xA11C_E55A {
            crate::console_write("Task A register/stack context: FAIL\n");
            crate::halt();
        }
        if USER_TASK_EXITED.load(Ordering::Acquire) {
            let status = USER_EXIT_STATUS.load(Ordering::Acquire);
            if status == 0 {
                crate::console_write("U-mode Exit syscall + scheduler task exit: PASS\n");
                crate::halt();
            }
            crate::console_write("U-mode Exit syscall status: FAIL\n");
            crate::halt();
        }
        match crate::ipc::receive(TaskId(0)) {
            Ok(Some(message)) => {
                if message.sender != TaskId(2) || message.payload != IPC_REQUEST {
                    crate::console_write("IPC request validation: FAIL\n");
                    crate::halt();
                }
                if crate::ipc::send(TaskId(0), TaskId(2), IPC_REPLY).is_err() {
                    crate::console_write("IPC reply enqueue: FAIL\n");
                    crate::halt();
                }
                IPC_REQUEST_RECEIVED.store(true, Ordering::Release);
            }
            Ok(None) => {}
            Err(_) => {
                crate::console_write("IPC mailbox receive: FAIL\n");
                crate::halt();
            }
        }
        TASK_A_TICKS.fetch_add(1, Ordering::Relaxed);
        if TIMER_INTERRUPTS.load(Ordering::Relaxed) >= 2 {
            TASK_A_CONTEXT_OK.store(true, Ordering::Release);
        }
        if TASK_B_TICKS.load(Ordering::Relaxed) != 0
            && TIMER_INTERRUPTS.load(Ordering::Relaxed) >= 2
            && !TIMER_TEST_REPORTED.swap(true, Ordering::AcqRel)
        {
            let ticks = TIMER_INTERRUPTS.load(Ordering::Relaxed);
            crate::console_write("RV64 timer preemption + stack context switch: PASS; interrupts=");
            crate::print_hex(ticks);
            crate::console_write("\n");
        }
        core::hint::spin_loop();
    }
}

extern "C" fn task_b() -> ! {
    let mut register_marker = 0xB11C_E000usize;
    let mut stack_canary = 0xB11C_E55Ausize;
    let stack_canary_ptr = &mut stack_canary as *mut usize;
    loop {
        unsafe { asm!("addi s2, s2, 0", inout("s2") register_marker, options(nomem, nostack)); }
        if register_marker != 0xB11C_E000
            || unsafe { core::ptr::read_volatile(stack_canary_ptr) } != 0xB11C_E55A {
            crate::console_write("Task B register/stack context: FAIL\n");
            crate::halt();
        }
        TASK_B_TICKS.fetch_add(1, Ordering::Relaxed);
        if TIMER_INTERRUPTS.load(Ordering::Relaxed) >= 3 {
            TASK_B_CONTEXT_OK.store(true, Ordering::Release);
        }
        core::hint::spin_loop();
    }
}

/// Create two S-mode tasks and one isolated U-mode task, then enter task A.
pub fn start() -> ! {
    let (a, b, user) = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        let a = (*scheduler).create_task().unwrap_or_else(|_| crate::halt());
        let b = (*scheduler).create_task().unwrap_or_else(|_| crate::halt());
        let user = (*scheduler).create_task().unwrap_or_else(|_| crate::halt());
        let stack0 = core::ptr::addr_of_mut!(TASK0_STACK);
        let stack1 = core::ptr::addr_of_mut!(TASK1_STACK);
        let frame0 = make_initial_frame(stack0, task_a);
        let frame1 = make_initial_frame(stack1, task_b);
        let user_frame = crate::trap::create_user_task_frame(user.0);
        if user_frame.is_null() { crate::halt(); }
        TASK_FRAMES[a.0] = frame0 as usize;
        TASK_FRAMES[b.0] = frame1 as usize;
        TASK_FRAMES[user.0] = user_frame as usize;
        (a, b, user)
    };

    let first = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        if (*scheduler).schedule_next() != Some(a) { crate::halt(); }
        if (*scheduler).state(b).is_err() || (*scheduler).state(user).is_err() {
            crate::halt();
        }
        TASK_FRAMES[a.0] as *mut TrapFrame
    };

    activate_address_space(a);
    arm_next_timer();
    unsafe {
        asm!(
            "csrs sie, {sie}",
            "csrs sstatus, {sstatus}",
            sie = in(reg) SIE_STIE,
            sstatus = in(reg) SSTATUS_SIE,
            options(nostack)
        );
        __gravity_s_trap_resume(first)
    }
}

/// Current scheduler task ID, used to validate ownership of U-mode trap storage.
pub(crate) fn current_task_id() -> Option<usize> {
    unsafe { (*core::ptr::addr_of!(SCHEDULER)).current().map(|id| id.0) }
}

/// Called by both S-origin and U-origin supervisor timer traps.
pub fn on_timer(frame: *mut TrapFrame) -> *mut TrapFrame {
    let current = unsafe { (*core::ptr::addr_of!(SCHEDULER)).current() };
    let current = match current {
        Some(id) => id,
        None => { crate::console_write("Scheduler timer: no current task\n"); crate::halt(); }
    };

    unsafe { TASK_FRAMES[current.0] = frame as usize; }
    if current.0 == 2 && unsafe { (*frame).sstatus & (1 << 8) == 0 } {
        USER_TIMER_PREEMPTED.store(true, Ordering::Release);
    }
    TIMER_INTERRUPTS.fetch_add(1, Ordering::Relaxed);
    arm_next_timer();

    let next: TaskId = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        match (*scheduler).schedule_next() {
            Some(id) => id,
            None => { crate::console_write("Scheduler timer: no runnable task\n"); crate::halt(); }
        }
    };
    activate_address_space(next);
    let next_frame = unsafe { TASK_FRAMES[next.0] as *mut TrapFrame };
    if next_frame.is_null() {
        crate::console_write("Scheduler timer: missing task context\n");
        crate::halt();
    }
    next_frame
}

/// Implement ABI v1 SendMessage=1: a0=destination task ID, a1=scalar payload.
pub fn on_user_send_message(frame: *mut TrapFrame) -> *mut TrapFrame {
    let current = unsafe { (*core::ptr::addr_of!(SCHEDULER)).current() };
    let sender = match current {
        Some(id) if id.0 == 2 => id,
        _ => { crate::console_write("SendMessage: invalid sender task\n"); crate::halt(); }
    };
    let destination = unsafe { (*frame).x[10] };
    let payload = unsafe { (*frame).x[11] };
    let result: isize = if destination >= crate::scheduler::MAX_TASKS {
        -22
    } else {
        let receiver = TaskId(destination);
        let active = unsafe {
            (*core::ptr::addr_of!(SCHEDULER)).state(receiver)
                .map(|state| state != crate::scheduler::TaskState::Exited)
                .unwrap_or(false)
        };
        if !active {
            -22
        } else {
            match crate::ipc::send(sender, receiver, payload) {
                Ok(()) => 0,
                Err(crate::ipc::IpcError::QueueFull) => -11,
                Err(crate::ipc::IpcError::InvalidTask) => -22,
            }
        }
    };
    unsafe {
        (*frame).x[10] = result as usize;
        (*frame).sepc = match (*frame).sepc.checked_add(4) {
            Some(next) => next,
            None => crate::halt(),
        };
    }
    frame
}

/// Implement non-blocking ABI v1 ReceiveMessage=2.
/// Success returns payload in a0 and sender task ID in a1; empty returns -EAGAIN.
pub fn on_user_receive_message(frame: *mut TrapFrame) -> *mut TrapFrame {
    let current = unsafe { (*core::ptr::addr_of!(SCHEDULER)).current() };
    let receiver = match current {
        Some(id) if id.0 == 2 => id,
        _ => { crate::console_write("ReceiveMessage: invalid receiver task\n"); crate::halt(); }
    };
    match crate::ipc::receive(receiver) {
        Ok(Some(message)) => unsafe {
            (*frame).x[10] = message.payload;
            (*frame).x[11] = message.sender.0;
            if message.sender == TaskId(0) && message.payload == IPC_REPLY {
                USER_IPC_REPLY_RECEIVED.store(true, Ordering::Release);
            }
        },
        Ok(None) => {
            USER_IPC_EMPTY_OBSERVED.store(true, Ordering::Release);
            unsafe { (*frame).x[10] = (-11isize) as usize; }
        },
        Err(_) => {
            crate::console_write("ReceiveMessage: invalid mailbox\n");
            crate::halt();
        }
    }
    unsafe {
        (*frame).sepc = match (*frame).sepc.checked_add(4) {
            Some(next) => next,
            None => crate::halt(),
        };
    }
    frame
}

pub fn ipc_empty_receive_observed() -> bool {
    USER_IPC_EMPTY_OBSERVED.load(Ordering::Acquire)
}

pub fn ipc_roundtrip_observed() -> bool {
    IPC_REQUEST_RECEIVED.load(Ordering::Acquire)
        && USER_IPC_REPLY_RECEIVED.load(Ordering::Acquire)
}

/// Implement ABI v1 Yield=4 for the current U-mode task. Its saved frame is
/// retained on the dedicated supervisor trap stack and resumed after rotation.
pub fn on_user_yield(frame: *mut TrapFrame) -> *mut TrapFrame {
    let current = unsafe { (*core::ptr::addr_of!(SCHEDULER)).current() };
    let current = match current {
        Some(id) if id.0 < TASK_COUNT => id,
        _ => { crate::console_write("Yield syscall: no current task\n"); crate::halt(); }
    };
    unsafe {
        if (*core::ptr::addr_of!(SCHEDULER)).state(current) != Ok(crate::scheduler::TaskState::Running) {
            crate::console_write("Yield syscall: task is not running\n");
            crate::halt();
        }
        (*frame).x[10] = 0;
        (*frame).sepc = match (*frame).sepc.checked_add(4) {
            Some(next) => next,
            None => crate::halt(),
        };
        TASK_FRAMES[current.0] = frame as usize;
    }

    let next = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        match (*scheduler).yield_current() {
            Ok(Some(id)) => id,
            _ => { crate::console_write("Yield syscall: scheduler has no next task\n"); crate::halt(); }
        }
    };
    activate_address_space(next);
    let next_frame = unsafe { TASK_FRAMES[next.0] as *mut TrapFrame };
    if next_frame.is_null() {
        crate::console_write("Yield syscall: missing next context\n");
        crate::halt();
    }
    next_frame
}

pub fn register_context_observed() -> bool {
    TASK_A_CONTEXT_OK.load(Ordering::Acquire) && TASK_B_CONTEXT_OK.load(Ordering::Acquire)
}

/// Exit the current U-mode test task and schedule another runnable task.
pub fn on_user_exit(frame: *mut TrapFrame) -> *mut TrapFrame {
    let current = unsafe { (*core::ptr::addr_of!(SCHEDULER)).current() };
    let current = match current {
        Some(id) if id.0 == 2 => id,
        _ => { crate::console_write("Exit syscall: invalid current task\n"); crate::halt(); }
    };
    let status = unsafe { (*frame).x[10] };
    USER_EXIT_STATUS.store(status, Ordering::Release);
    USER_TASK_EXITED.store(true, Ordering::Release);

    let next = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        match (*scheduler).exit_current() {
            Ok(Some(id)) => id,
            _ => { crate::console_write("Exit syscall: scheduler has no next task\n"); crate::halt(); }
        }
    };
    activate_address_space(next);
    let next_frame = unsafe { TASK_FRAMES[next.0] as *mut TrapFrame };
    if next_frame.is_null() {
        crate::console_write("Exit syscall: missing next context\n");
        crate::halt();
    }
    let _ = current;
    next_frame
}

pub fn user_timer_preemption_observed() -> bool {
    USER_TIMER_PREEMPTED.load(Ordering::Acquire)
}

pub fn timer_preemption_observed() -> bool {
    TIMER_INTERRUPTS.load(Ordering::Acquire) >= 2
        && TASK_A_TICKS.load(Ordering::Acquire) != 0
        && TASK_B_TICKS.load(Ordering::Acquire) != 0
}
