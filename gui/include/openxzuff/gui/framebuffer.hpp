#pragma once
#include <cstddef>
#include <cstdint>
#include <vector>
namespace openxzuff::gui {
struct Pixel { std::uint8_t r{}; std::uint8_t g{}; std::uint8_t b{}; std::uint8_t a{255}; };
class Framebuffer {
public:
    Framebuffer(std::size_t width, std::size_t height);
    std::size_t width() const noexcept;
    std::size_t height() const noexcept;
    bool set_pixel(std::size_t x, std::size_t y, Pixel pixel) noexcept;
    Pixel pixel_at(std::size_t x, std::size_t y) const noexcept;
    void clear(Pixel pixel) noexcept;
private:
    std::size_t width_;
    std::size_t height_;
    std::vector<Pixel> pixels_;
};
} // namespace openxzuff::gui
