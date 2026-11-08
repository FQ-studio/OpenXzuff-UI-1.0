#pragma once

#include <cstddef>
#include <cstdint>
#include <type_traits>

namespace openxzuff::kernel::riscv64 {

inline constexpr std::size_t register_zero = 0;
inline constexpr std::size_t register_sp = 2;
inline constexpr std::size_t register_a0 = 10;
inline constexpr std::size_t register_a5 = 15;
inline constexpr std::size_t register_a7 = 17;

// Memory layout contract for a future S-mode trap entry. x[n] stores the
// interrupted value of integer register xn; x0 is always represented as zero.
// The assembly entry must preserve this exact order and 16-byte stack alignment.
struct alignas(16) TrapFrame {
    std::uint64_t x[32]{};
    std::uint64_t sepc{};
    std::uint64_t sstatus{};
    std::uint64_t scause{};
    std::uint64_t stval{};
};

static_assert(std::is_standard_layout_v<TrapFrame>);
static_assert(offsetof(TrapFrame, x) == 0);
static_assert(offsetof(TrapFrame, sepc) == 256);
static_assert(offsetof(TrapFrame, sstatus) == 264);
static_assert(offsetof(TrapFrame, scause) == 272);
static_assert(offsetof(TrapFrame, stval) == 280);
static_assert(sizeof(TrapFrame) == 288);
static_assert(alignof(TrapFrame) == 16);

} // namespace openxzuff::kernel::riscv64
