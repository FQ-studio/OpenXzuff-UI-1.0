#pragma once

#include <cstddef>
#include <cstdint>

namespace openxzuff::drivers::serial {

// The caller must validate the MMIO region from the firmware FDT and map it
// uncached/device-memory before constructing a driver. Never use this against
// arbitrary host memory.
class PollingUart {
public:
    virtual ~PollingUart() = default;
    virtual bool put(char value) noexcept = 0;
    bool write(const char* data, std::size_t size) noexcept;
};

// FU740 SiFive UART. Register offsets are byte offsets from the FDT reg base.
class SiFiveUart final : public PollingUart {
public:
    explicit SiFiveUart(std::uintptr_t mmio_base) noexcept : base_(mmio_base) {}
    bool put(char value) noexcept override;
private:
    volatile std::uint32_t* reg(std::uintptr_t offset) const noexcept;
    std::uintptr_t base_;
};

// NS16550-compatible UART, including the DesignWare APB UART used by TH1520.
// reg_shift and register access width must come from a validated FDT node.
// This first version supports 32-bit MMIO access only.
class Ns16550Uart final : public PollingUart {
public:
    Ns16550Uart(std::uintptr_t mmio_base, unsigned reg_shift) noexcept
        : base_(mmio_base), reg_shift_(reg_shift) {}
    bool put(char value) noexcept override;
private:
    volatile std::uint32_t* reg(unsigned index) const noexcept;
    std::uintptr_t base_;
    unsigned reg_shift_;
};

} // namespace openxzuff::drivers::serial
