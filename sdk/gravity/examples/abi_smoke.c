#include <gravity/syscall.h>

/*
 * Compile-only SDK example. The live kernel currently has a built-in U-mode
 * probe, not a general application loader, so CI does not execute this object.
 */
int main(void) {
    const int64_t yielded = gravity_yield();
    if (gravity_is_error(yielded)) {
        return (int)gravity_exit((int32_t)yielded);
    }

    const int64_t sent = gravity_send_message(0, UINT64_C(0x47524156));
    if (gravity_is_error(sent)) {
        return (int)gravity_exit((int32_t)sent);
    }

    uint64_t sender = 0;
    const int64_t received = gravity_receive_message(&sender);
    (void)sender;
    if (received == -GRAVITY_EAGAIN) {
        return (int)gravity_exit(0);
    }
    if (gravity_is_error(received)) {
        return (int)gravity_exit((int32_t)received);
    }
    return (int)gravity_exit((int32_t)received);
}
