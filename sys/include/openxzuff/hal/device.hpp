#pragma once

#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace openxzuff::hal {

// An MMIO window copied from a validated firmware resource. This type checks
// arithmetic and sub-ranges; it does not map memory or grant access by itself.
struct MmioResource {
    std::uint64_t base{};
    std::uint64_t size{};

    [[nodiscard]] constexpr bool valid() const noexcept {
        return size != 0 && base <= UINT64_MAX - size;
    }

    [[nodiscard]] constexpr bool contains(std::uint64_t address,
                                          std::uint64_t length) const noexcept {
        if (!valid() || length == 0 || address < base) return false;
        if (address > UINT64_MAX - length) return false;
        return address + length <= base + size;
    }
};

struct DeviceDescriptor {
    std::string path;
    std::string compatible;
    MmioResource registers;
    std::uint32_t interrupt{};
    bool interrupt_valid{};
};

// Firmware discovery produces descriptors; the catalog rejects malformed or
// duplicate identities and offers exact compatible matching to driver binders.
class DeviceCatalog {
public:
    [[nodiscard]] bool add(DeviceDescriptor device);
    [[nodiscard]] const DeviceDescriptor* find_path(std::string_view path) const noexcept;
    [[nodiscard]] const DeviceDescriptor* find_compatible(std::string_view compatible) const noexcept;
    [[nodiscard]] std::size_t size() const noexcept;

private:
    std::vector<DeviceDescriptor> devices_;
};

} // namespace openxzuff::hal
