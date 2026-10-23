#pragma once
#include <string_view>
namespace openxzuff::app {
struct Manifest { std::string_view id; std::string_view name; std::string_view version; };
constexpr bool valid_manifest(const Manifest& manifest) noexcept {
    return !manifest.id.empty() && !manifest.name.empty() && !manifest.version.empty() &&
           manifest.id.find(' ') == std::string_view::npos;
}
} // namespace openxzuff::app
