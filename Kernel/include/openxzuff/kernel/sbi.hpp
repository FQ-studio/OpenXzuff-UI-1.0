#pragma once

#include <cstdint>

namespace openxzuff::kernel::sbi {

struct Result {
    std::int64_t error;
    std::uint64_t value;
    constexpr bool ok() const noexcept { return error == 0; }
};

enum class ResetType : std::uint32_t { Shutdown = 0, ColdReboot = 1, WarmReboot = 2 };
enum class ResetReason : std::uint32_t { NoReason = 0, SystemFailure = 1 };

// SBI calls are available only after firmware has entered the kernel in S-mode.
// On non-RISC-V host builds, calls return -2 (not supported) for safe smoke tests.
Result call(std::uint64_t extension, std::uint64_t function,
            std::uint64_t arg0 = 0, std::uint64_t arg1 = 0,
            std::uint64_t arg2 = 0, std::uint64_t arg3 = 0,
            std::uint64_t arg4 = 0, std::uint64_t arg5 = 0) noexcept;

Result get_spec_version() noexcept;
Result probe_extension(std::uint64_t extension) noexcept;
Result set_timer(std::uint64_t absolute_time) noexcept;
Result hart_start(std::uint64_t hart_id, std::uint64_t start_address,
                  std::uint64_t opaque) noexcept;
Result system_reset(ResetType type, ResetReason reason) noexcept;

} // namespace openxzuff::kernel::sbi
