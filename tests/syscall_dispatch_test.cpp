#include "openxzuff/kernel/syscall.hpp"

#include <cstdint>
#include <iostream>

namespace {
struct HookState {
    std::uint64_t exit_status{};
    unsigned yield_count{};
    unsigned exit_count{};
};

void on_yield(void* context) noexcept {
    ++static_cast<HookState*>(context)->yield_count;
}

void on_exit(void* context, std::uint64_t status) noexcept {
    auto& state = *static_cast<HookState*>(context);
    state.exit_status = status;
    ++state.exit_count;
}

bool check(bool condition, const char* message) {
    if (!condition) std::cerr << "FAIL: " << message << '\n';
    return condition;
}
}

int main() {
    using namespace openxzuff::kernel;
    bool ok = true;
    HookState state{};
    const SyscallHooks hooks{&state, on_yield, on_exit};
    SyscallArguments args{};

    args.values[0] = 42;
    const auto exited = dispatch_syscall(0, args, hooks);
    ok &= check(exited.terminated && exited.value == 0, "Exit terminates task");
    ok &= check(state.exit_count == 1 && state.exit_status == 42, "Exit forwards status");

    const auto yielded = dispatch_syscall(4, args, hooks);
    ok &= check(!yielded.terminated && yielded.value == 0, "Yield returns normally");
    ok &= check(state.yield_count == 1, "Yield invokes scheduler hook");

    for (const std::uint64_t id : {1ULL, 2ULL, 3ULL, 99ULL}) {
        const auto unsupported = dispatch_syscall(id, args, hooks);
        ok &= check(unsupported.value == syscall_error_not_implemented &&
                    !unsupported.terminated, "unimplemented and unknown calls return -ENOSYS");
    }

    const SyscallHooks no_hooks{};
    const auto no_scheduler = dispatch_syscall(4, args, no_hooks);
    const auto no_exit = dispatch_syscall(0, args, no_hooks);
    ok &= check(no_scheduler.value == syscall_error_not_implemented,
                "Yield without scheduler hook is not implemented");
    ok &= check(no_exit.value == syscall_error_not_implemented && !no_exit.terminated,
                "Exit without process hook does not claim termination");

    if (ok) std::cout << "Gravity syscall dispatch tests: PASS\n";
    return ok ? 0 : 1;
}
