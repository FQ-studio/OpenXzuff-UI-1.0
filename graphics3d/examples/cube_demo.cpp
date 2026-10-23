#include "xzuff3d/renderer.hpp"
#include <exception>
#include <iostream>

int main(int argc, char** argv) {
    try {
        const char* output = argc > 1 ? argv[1] : "xzuff3d-cube.ppm";
        xzuff3d::Framebuffer framebuffer(320, 240);
        xzuff3d::Renderer renderer(framebuffer);
        renderer.clear({10, 14, 26});
        renderer.drawCube({0.0F, 0.0F, 0.0F}, 1.6F, 0.65F);
        framebuffer.writePpm(output);
        std::cout << "Rendered 3D cube to " << output << '\n';
        return 0;
    } catch (const std::exception& e) {
        std::cerr << "OpenXzuff3D error: " << e.what() << '\n';
        return 1;
    }
}
