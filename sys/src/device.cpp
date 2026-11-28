#include "openxzuff/hal/device.hpp"

#include <algorithm>
#include <utility>

namespace openxzuff::hal {

bool DeviceCatalog::add(DeviceDescriptor device) {
    if (device.path.empty() || device.compatible.empty() || !device.registers.valid()) {
        return false;
    }
    if (find_path(device.path) != nullptr) return false;
    devices_.push_back(std::move(device));
    return true;
}

const DeviceDescriptor* DeviceCatalog::find_path(std::string_view path) const noexcept {
    const auto it = std::find_if(devices_.begin(), devices_.end(),
        [path](const DeviceDescriptor& device) { return device.path == path; });
    return it == devices_.end() ? nullptr : &*it;
}

const DeviceDescriptor* DeviceCatalog::find_compatible(std::string_view compatible) const noexcept {
    const auto it = std::find_if(devices_.begin(), devices_.end(),
        [compatible](const DeviceDescriptor& device) { return device.compatible == compatible; });
    return it == devices_.end() ? nullptr : &*it;
}

std::size_t DeviceCatalog::size() const noexcept { return devices_.size(); }

} // namespace openxzuff::hal
