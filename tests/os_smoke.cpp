#include "openxzuff/app/manifest.hpp"
#include "openxzuff/gui/framebuffer.hpp"
#include "openxzuff/kernel/contracts.hpp"
#include "openxzuff/sys/service_registry.hpp"
#include <cstdlib>
#include <iostream>
#include <string_view>
namespace {
void require(bool condition, std::string_view message) {
    if (!condition) { std::cerr << "FAIL: " << message << '\n'; std::exit(1); }
}
}
int main() {
    using namespace openxzuff;
    require(kernel::valid_user_range(0x1000, 16, 0x8000), "valid user memory range accepted");
    require(!kernel::valid_user_range(0x7ff8, 16, 0x8000), "range crossing user limit rejected");
    require(!kernel::valid_user_range(UINT64_MAX - 2, 8, UINT64_MAX), "overflowing range rejected");
    require(!kernel::valid_user_range(0x1000, 0, 0x8000), "zero-length range rejected");
    sys::ServiceRegistry registry;
    require(registry.add("display", 1), "service registration");
    require(!registry.add("display", 2), "duplicate service rejected");
    require(!registry.add("input", 0), "zero endpoint rejected");
    require(registry.find("display") != nullptr, "service lookup");
    require(registry.find("input") == nullptr, "missing service lookup");
    require(registry.size() == 1, "service count");
    constexpr app::Manifest good{"org.fqstudio.demo", "Demo", "0.1.0"};
    constexpr app::Manifest bad{"bad app id", "Bad", "0.1.0"};
    static_assert(app::valid_manifest(good));
    static_assert(!app::valid_manifest(bad));
    gui::Framebuffer fb(4, 3);
    const gui::Pixel blue{20, 80, 220, 255};
    require(fb.width() == 4 && fb.height() == 3, "framebuffer dimensions");
    require(fb.set_pixel(2, 1, blue), "in-bounds pixel write");
    const auto written = fb.pixel_at(2, 1);
    require(written.r == 20 && written.g == 80 && written.b == 220, "pixel readback");
    require(!fb.set_pixel(4, 1, blue), "out-of-bounds pixel write rejected");
    fb.clear(gui::Pixel{1, 2, 3, 255});
    require(fb.pixel_at(2, 1).g == 2, "framebuffer clear");
    std::cout << "OpenXzuff OS smoke tests: PASS\n";
    return 0;
}
