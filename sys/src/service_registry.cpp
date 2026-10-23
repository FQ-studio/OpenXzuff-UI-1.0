#include "openxzuff/sys/service_registry.hpp"
#include <algorithm>
namespace openxzuff::sys {
bool ServiceRegistry::add(std::string_view name, std::uint32_t endpoint) {
    if (name.empty() || endpoint == 0 || find(name) != nullptr) return false;
    services_.push_back(ServiceDescriptor{std::string{name}, endpoint});
    return true;
}
const ServiceDescriptor* ServiceRegistry::find(std::string_view name) const noexcept {
    const auto it = std::find_if(services_.begin(), services_.end(), [name](const ServiceDescriptor& s) { return s.name == name; });
    return it == services_.end() ? nullptr : &*it;
}
std::size_t ServiceRegistry::size() const noexcept { return services_.size(); }
} // namespace openxzuff::sys
