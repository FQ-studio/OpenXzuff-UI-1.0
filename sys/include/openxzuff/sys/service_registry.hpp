#pragma once
#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>
#include <vector>
namespace openxzuff::sys {
struct ServiceDescriptor { std::string name; std::uint32_t endpoint; };
class ServiceRegistry {
public:
    bool add(std::string_view name, std::uint32_t endpoint);
    const ServiceDescriptor* find(std::string_view name) const noexcept;
    std::size_t size() const noexcept;
private:
    std::vector<ServiceDescriptor> services_;
};
} // namespace openxzuff::sys
