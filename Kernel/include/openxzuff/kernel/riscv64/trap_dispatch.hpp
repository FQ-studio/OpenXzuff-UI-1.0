#pragma once

#include "openxzuff/kernel/riscv64/trap_frame.hpp"
#include "openxzuff/kernel/syscall.hpp"

#include <cstdint>

namespace openxzuff::kernel::riscv64 {

inline constexpr std::uint64_t user_environment_call_cause = 8;

struct TrapDispatchOutcome {
    bool handled{};
    bool terminated{};
};

// Handles only an environment-call exception originating in U-mode.
// The assembly trap entry/return path is a separate, unimplemented milestone.
TrapDispatchOutcome dispatch_user_trap(TrapFrame& frame,
                                       const SyscallHooks& hooks) noexcept;

} // namespace openxzuff::kernel::riscv64
