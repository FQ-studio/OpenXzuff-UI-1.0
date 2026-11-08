#include "openxzuff/kernel/riscv64/trap_dispatch.hpp"

#include <cstdint>
#include <iostream>

namespace {
struct State {
    std::uint64_t exit_status{};
    unsigned yields{};
};

void yield_task(void* context) noexcept {
    ++static_cast<State*>(context)->yields;
}

void exit_task(void* context, std::uint64_t status) noexcept {
    static_cast<State*>(context)->exit_status = status;
}
}

int main() {
    using namespace openxzuff::kernel;
    using namespace openxzuff::kernel::riscv64;
    State state{};
    const SyscallHooks hooks{&state, yield_task, exit_task};
    TrapFrame frame{};

    frame.scause = user_environment_call_cause;
    frame.sepc = 0x1000;
    frame.x[register_a7] = 4;
    const auto yielded = dispatch_user_trap(frame, hooks);
    if (!yielded.handled || yielded.terminated || frame.x[register_a0] != 0 ||
        frame.sepc != 0x1004 || state.yields != 1) {
        std::cerr << "FAIL: user Yield ecall dispatch\n";
        return 1;
    }

    frame.sepc = 0x2000;
    frame.x[register_a7] = 0;
    frame.x[register_a0] = 73;
    const auto exited = dispatch_user_trap(frame, hooks);
    if (!exited.handled || !exited.terminated || state.exit_status != 73 ||
        frame.sepc != 0x2000) {
        std::cerr << "FAIL: user Exit ecall dispatch\n";
        return 1;
    }

    frame.scause = 9;
    const auto supervisor_call = dispatch_user_trap(frame, hooks);
    if (supervisor_call.handled || supervisor_call.terminated) {
        std::cerr << "FAIL: S-mode ecall must not be handled as U-mode syscall\n";
        return 1;
    }

    std::cout << "RV64 user trap dispatch tests: PASS\n";
    return 0;
}
