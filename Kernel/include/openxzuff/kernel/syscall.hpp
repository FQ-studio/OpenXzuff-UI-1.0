#pragma once

#include <array>
#include <cstdint>

namespace openxzuff::kernel {

inline constexpr std::int64_t syscall_error_invalid = -22;       // -EINVAL
inline constexpr std::int64_t syscall_error_bad_address = -14;   // -EFAULT
inline constexpr std::int64_t syscall_error_not_implemented = -38; // -ENOSYS

struct SyscallArguments {
    std::array<std::uint64_t, 6> values{};
};

struct SyscallOutcome {
    std::int64_t value{};
    bool terminated{};
};

struct SyscallHooks {
    void* context{};
    void (*yield_task)(void*) noexcept{};
    void (*exit_task)(void*, std::uint64_t) noexcept{};
};

// Dispatches the stable ABI v1 syscall numbers. This is a host-testable
// policy boundary, not an RV64 trap entry or a complete kernel implementation.
SyscallOutcome dispatch_syscall(std::uint64_t syscall_id,
                                const SyscallArguments& arguments,
                                const SyscallHooks& hooks) noexcept;

} // namespace openxzuff::kernel
