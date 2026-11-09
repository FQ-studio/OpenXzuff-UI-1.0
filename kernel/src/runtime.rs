//! RV64 S-mode task runtime: real trap-frame context switching and SBI timer preemption.
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::scheduler::{Scheduler, TaskId};
use crate::trap::TrapFrame;

const TASK_COUNT: usize = 2;
const STACK_SIZE: usize = 16 * 1024;
const SBI_EXT_TIME: usize = 0x5449_4d45;
const SBI_FID_SET_TIMER: usize = 0;
const TIMER_QUANTUM: usize = 100_000; // QEMU virt timebase is 10 MHz: 10 ms.
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

fn arm_next_timer() {
    if set_timer(read_time().wrapping_add(TIMER_QUANTUM)).is_err() {
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
    frame.write(TrapFrame::new(entry as usize, top, gp));
    frame
}

extern "C" fn task_a() -> ! {
    loop {
        TASK_A_TICKS.fetch_add(1, Ordering::Relaxed);
        if TASK_B_TICKS.load(Ordering::Relaxed) != 0 {
            let ticks = TIMER_INTERRUPTS.load(Ordering::Relaxed);
            crate::console_write("RV64 timer preemption + stack context switch: PASS; interrupts=");
            crate::print_hex(ticks);
            crate::console_write("\n");
            crate::trap::umode_smoke_test();
        }
        core::hint::spin_loop();
    }
}

extern "C" fn task_b() -> ! {
    loop {
        TASK_B_TICKS.fetch_add(1, Ordering::Relaxed);
        core::hint::spin_loop();
    }
}

/// Start two independent S-mode tasks. This function transfers control to the
/// first task and never returns; timer interrupts switch between saved frames.
pub fn start() -> ! {
    let (a, b) = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        let a = (*scheduler).create_task().unwrap_or_else(|_| crate::halt());
        let b = (*scheduler).create_task().unwrap_or_else(|_| crate::halt());
        let stack0 = core::ptr::addr_of_mut!(TASK0_STACK);
        let stack1 = core::ptr::addr_of_mut!(TASK1_STACK);
        let frame0 = make_initial_frame(stack0, task_a);
        let frame1 = make_initial_frame(stack1, task_b);
        TASK_FRAMES[a.0] = frame0 as usize;
        TASK_FRAMES[b.0] = frame1 as usize;
        (a, b)
    };

    let first = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        if (*scheduler).schedule_next() != Some(a) { crate::halt(); }
        if (*scheduler).state(b).is_err() { crate::halt(); }
        TASK_FRAMES[a.0] as *mut TrapFrame
    };

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

/// Called only from the S-mode trap handler with supervisor interrupts masked.
/// Save the interrupted frame, rotate the run queue, and return the next frame.
pub fn on_timer(frame: *mut TrapFrame) -> *mut TrapFrame {
    let current = unsafe { (*core::ptr::addr_of!(SCHEDULER)).current() };
    let current = match current {
        Some(id) => id,
        None => { crate::console_write("Scheduler timer: no current task\n"); crate::halt(); }
    };

    unsafe { TASK_FRAMES[current.0] = frame as usize; }
    TIMER_INTERRUPTS.fetch_add(1, Ordering::Relaxed);
    arm_next_timer();

    let next: TaskId = unsafe {
        let scheduler = core::ptr::addr_of_mut!(SCHEDULER);
        match (*scheduler).schedule_next() {
            Some(id) => id,
            None => { crate::console_write("Scheduler timer: no runnable task\n"); crate::halt(); }
        }
    };
    let next_frame = unsafe { TASK_FRAMES[next.0] as *mut TrapFrame };
    if next_frame.is_null() {
        crate::console_write("Scheduler timer: missing task context\n");
        crate::halt();
    }
    next_frame
}
