# Kernel Scheduler

## Implemented RV64 runtime

The executable scheduler is in kernel/src/scheduler.rs; the RV64 trap/timer integration is in kernel/src/runtime.rs.

- Fixed-capacity, allocation-free round-robin run queue with Ready, Running, Blocked, and Exited states.
- Three boot-time test contexts: two independent S-mode tasks with separate stacks, plus one U-mode probe task with a dedicated user stack and supervisor trap stack.
- Full 288-byte RV64 trap frames are saved and restored through the common sret path, including mixed S-mode/U-mode returns.
- SBI TIME deadlines are derived from the firmware FDT timebase-frequency; the current quantum is 10 ms.
- Supervisor timer interrupts preempt both S-mode and U-mode tasks.
- ABI v1 Yield = 4 saves the U-mode frame, rotates the run queue, and resumes the user task after other tasks have run.

The OpenSBI/QEMU workflow requires the timer/context-switch and U-mode Yield assertions to appear in the serial log.

## Explicit limitations

This is a single-hart bring-up scheduler, not a production task manager. There is no priority scheduling, SMP locking, idle task, blocking wait queue integration, process lifecycle/reaping, per-task resource table, or general task creation API yet. The timer test tasks are kernel probes, not applications.
