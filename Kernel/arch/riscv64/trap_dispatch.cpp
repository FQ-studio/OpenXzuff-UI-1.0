#include "openxzuff/kernel/riscv64/trap_dispatch.hpp"

namespace openxzuff::kernel::riscv64 {

TrapDispatchOutcome dispatch_user_trap(TrapFrame& frame,
                                       const SyscallHooks& hooks) noexcept {
    if (frame.scause != user_environment_call_cause) {
        return {false, false};
    }

    SyscallArguments arguments{};
    for (std::size_t i = 0; i < arguments.values.size(); ++i) {
        arguments.values[i] = frame.x[register_a0 + i];
    }

    const auto outcome = dispatch_syscall(frame.x[register_a7], arguments, hooks);
    if (!outcome.terminated) {
        frame.x[register_a0] = static_cast<std::uint64_t>(outcome.value);
        frame.sepc += 4;
    }

    return {true, outcome.terminated};
}

} // namespace openxzuff::kernel::riscv64
