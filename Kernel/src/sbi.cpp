#include "openxzuff/kernel/sbi.hpp"

namespace openxzuff::kernel::sbi {
namespace {
constexpr std::uint64_t ext_base = 0x10;
constexpr std::uint64_t ext_time = 0x54494D45;
constexpr std::uint64_t ext_hsm = 0x48534D;
constexpr std::uint64_t ext_srst = 0x53525354;
[[maybe_unused]] constexpr std::int64_t not_supported = -2;
}

Result call(std::uint64_t extension, std::uint64_t function,
            std::uint64_t arg0, std::uint64_t arg1,
            std::uint64_t arg2, std::uint64_t arg3,
            std::uint64_t arg4, std::uint64_t arg5) noexcept {
#if defined(__riscv) && __riscv_xlen == 64
    register std::uint64_t a0 __asm__("a0") = arg0;
    register std::uint64_t a1 __asm__("a1") = arg1;
    register std::uint64_t a2 __asm__("a2") = arg2;
    register std::uint64_t a3 __asm__("a3") = arg3;
    register std::uint64_t a4 __asm__("a4") = arg4;
    register std::uint64_t a5 __asm__("a5") = arg5;
    register std::uint64_t a6 __asm__("a6") = function;
    register std::uint64_t a7 __asm__("a7") = extension;
    __asm__ volatile("ecall"
                 : "+r"(a0), "+r"(a1)
                 : "r"(a2), "r"(a3), "r"(a4), "r"(a5), "r"(a6), "r"(a7)
                 : "memory");
    return {static_cast<std::int64_t>(a0), a1};
#else
    (void)extension; (void)function; (void)arg0; (void)arg1; (void)arg2;
    (void)arg3; (void)arg4; (void)arg5;
    return {not_supported, 0};
#endif
}

Result get_spec_version() noexcept { return call(ext_base, 0); }
Result probe_extension(std::uint64_t extension) noexcept { return call(ext_base, 3, extension); }
Result set_timer(std::uint64_t absolute_time) noexcept { return call(ext_time, 0, absolute_time); }
Result hart_start(std::uint64_t hart_id, std::uint64_t start_address,
                  std::uint64_t opaque) noexcept {
    return call(ext_hsm, 0, hart_id, start_address, opaque);
}
Result system_reset(ResetType type, ResetReason reason) noexcept {
    return call(ext_srst, 0, static_cast<std::uint32_t>(type),
                static_cast<std::uint32_t>(reason));
}

} // namespace openxzuff::kernel::sbi
