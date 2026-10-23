#include "openxzuff/app/manifest.hpp"
#include <iostream>
int main() {
    constexpr openxzuff::app::Manifest manifest{"org.fqstudio.hello", "Hello OpenXzuff", "0.1.0"};
    if (!openxzuff::app::valid_manifest(manifest)) return 1;
    std::cout << manifest.name << " " << manifest.version << '\n';
    return 0;
}
