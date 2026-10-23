#pragma once
#include <cstddef>
#include <cstdint>
#include <string_view>
namespace openxzuff::kernel {
enum class SyscallId : std::uint64_t { Exit = 0, SendMessage = 1, ReceiveMessage = 2, MapMemory = 3, Yield = 4 };
struct IpcMessage { std::uint64_t sender{}; std::uint64_t kind{}; std::uint64_t arg0{}; std::uint64_t arg1{}; };
constexpr bool valid_user_range(std::uint64_t address, std::uint64_t length, std::uint64_t user_limit) noexcept {
    if (length == 0 || address >= user_limit) return false;
    return length <= user_limit - address;
}
constexpr std::string_view target_name = "Gravity Kernel RV64";
std::uint64_t contract_version() noexcept;
} // namespace openxzuff::kernel
