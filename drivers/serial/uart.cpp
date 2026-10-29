#include "uart.hpp"

namespace openxzuff::drivers::serial {

bool PollingUart::write(const char* data, std::size_t size) noexcept {
    if (data == nullptr && size != 0U) {
        return false;
    }
    for (std::size_t i = 0; i < size; ++i) {
        if (!put(data[i])) {
            return false;
        }
    }
    return true;
}

volatile std::uint32_t* SiFiveUart::reg(std::uintptr_t offset) const noexcept {
    return reinterpret_cast<volatile std::uint32_t*>(base_ + offset);
}

bool SiFiveUart::put(char value) noexcept {
    // SiFive UART TXDATA bit 31 indicates FIFO full; low 8 bits hold data.
    constexpr std::uintptr_t txdata = 0x00;
    constexpr std::uint32_t fifo_full = 1U << 31U;
    // Bound polling so a missing clock or invalid FDT mapping cannot hang boot.
    constexpr std::uint32_t poll_limit = 1'000'000U;
    for (std::uint32_t i = 0; i < poll_limit; ++i) {
        if ((*reg(txdata) & fifo_full) == 0U) {
            *reg(txdata) = static_cast<std::uint8_t>(value);
            return true;
        }
    }
    return false;
}

volatile std::uint32_t* Ns16550Uart::reg(unsigned index) const noexcept {
    const std::uintptr_t offset = static_cast<std::uintptr_t>(index) << reg_shift_;
    return reinterpret_cast<volatile std::uint32_t*>(base_ + offset);
}

bool Ns16550Uart::put(char value) noexcept {
    // 16550 LSR bit 5 (THRE) means the transmit holding register is empty.
    constexpr unsigned lsr = 5U;
    constexpr unsigned thr = 0U;
    constexpr std::uint32_t thre = 1U << 5U;
    constexpr std::uint32_t poll_limit = 1'000'000U;
    for (std::uint32_t i = 0; i < poll_limit; ++i) {
        if ((*reg(lsr) & thre) != 0U) {
            *reg(thr) = static_cast<std::uint8_t>(value);
            return true;
        }
    }
    return false;
}

} // namespace openxzuff::drivers::serial
