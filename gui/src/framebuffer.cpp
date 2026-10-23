#include "openxzuff/gui/framebuffer.hpp"
#include <stdexcept>
namespace openxzuff::gui {
Framebuffer::Framebuffer(std::size_t width, std::size_t height) : width_(width), height_(height) {
    if (width == 0 || height == 0 || width > pixels_.max_size() / height)
        throw std::invalid_argument("framebuffer dimensions are invalid");
    pixels_.resize(width * height);
}
std::size_t Framebuffer::width() const noexcept { return width_; }
std::size_t Framebuffer::height() const noexcept { return height_; }
bool Framebuffer::set_pixel(std::size_t x, std::size_t y, Pixel pixel) noexcept {
    if (x >= width_ || y >= height_) return false;
    pixels_[y * width_ + x] = pixel;
    return true;
}
Pixel Framebuffer::pixel_at(std::size_t x, std::size_t y) const noexcept {
    if (x >= width_ || y >= height_) return Pixel{0, 0, 0, 0};
    return pixels_[y * width_ + x];
}
void Framebuffer::clear(Pixel pixel) noexcept { for (auto& item : pixels_) item = pixel; }
} // namespace openxzuff::gui
