#pragma once

#include <cstdint>
#include <string>

namespace xzuff3d {

struct Vec3 { float x{}, y{}, z{}; };
struct Color { std::uint8_t r{}, g{}, b{}; };

class Framebuffer {
public:
    Framebuffer(int width, int height);
    ~Framebuffer();
    Framebuffer(const Framebuffer&) = delete;
    Framebuffer& operator=(const Framebuffer&) = delete;
    int width() const noexcept { return width_; }
    int height() const noexcept { return height_; }
    const std::uint8_t* pixels() const noexcept { return pixels_; }
    std::uint8_t* pixels() noexcept { return pixels_; }
    void clear(Color color);
    void writePpm(const std::string& path) const;
private:
    int width_;
    int height_;
    std::uint8_t* pixels_;
    friend class Renderer;
};

class Renderer {
public:
    explicit Renderer(Framebuffer& target);
    void clear(Color color = {12, 16, 28});
    void drawCube(Vec3 center, float size, float angleRadians,
                  Color color = {35, 170, 255});
private:
    Framebuffer& target_;
};

} // namespace xzuff3d
