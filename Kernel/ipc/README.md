# Inter-Process Communication

## Implemented: bounded scalar mailboxes

The RV64 runtime now has an allocation-free, per-task FIFO mailbox implementation in `kernel/src/ipc.rs`.

- One mailbox per scheduler task ID.
- Eight queued messages per mailbox.
- Each message carries a sender task ID and one machine-word scalar payload.
- `SendMessage` enqueues without dereferencing a user pointer.
- `ReceiveMessage` dequeues the oldest message and does not block; an empty queue returns `-EAGAIN`.
- Queue mutation masks S-mode interrupts so the single-hart timer cannot preempt a task halfway through a mailbox update.

The OpenSBI/QEMU probe exercises a U-mode request to task 0, a receive in task 0, a reply back to the U-mode task, and a U-mode receive. The host test suite covers FIFO ordering, receiver isolation, queue capacity/recovery, and invalid task IDs.

## Explicit limitations

This is a single-hart bootstrap IPC transport, not the final service IPC design. Blocking receive/wakeup, byte buffers, user-pointer validation and copying, endpoint capability checks, message cancellation, SMP locking, and process teardown are not implemented. The queue API must not be treated as SMP-safe until its critical section is replaced with an appropriate multiprocessor synchronization primitive.
