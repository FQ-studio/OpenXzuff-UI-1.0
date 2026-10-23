#include "xzuff3d/renderer.hpp"

#include <algorithm>
#include <cmath>
#include <fstream>
#include <limits>
#include <stdexcept>
#include <vector>

namespace xzuff3d {
Framebuffer::Framebuffer(int width, int height) : width_(width), height_(height), pixels_(nullptr) {
    if (width <= 0 || height <= 0 || width > 8192 || height > 8192)
        throw std::invalid_argument("invalid framebuffer dimensions");
    pixels_ = new std::uint8_t[static_cast<std::size_t>(width) * static_cast<std::size_t>(height) * 3U]{};
}
Framebuffer::~Framebuffer() { delete[] pixels_; }

void Framebuffer::clear(Color c) {
    const auto count = static_cast<std::size_t>(width_) * static_cast<std::size_t>(height_);
    for (std::size_t i = 0; i < count; ++i) {
        pixels_[i * 3] = c.r;
        pixels_[i * 3 + 1] = c.g;
        pixels_[i * 3 + 2] = c.b;
    }
}
void Framebuffer::writePpm(const std::string& path) const {
    std::ofstream out(path, std::ios::binary);
    if (!out) throw std::runtime_error("cannot open output image");
    out << "P6\n" << width_ << " " << height_ << "\n255\n";
    out.write(reinterpret_cast<const char*>(pixels_),
              static_cast<std::streamsize>(static_cast<std::size_t>(width_) * height_ * 3U));
    if (!out) throw std::runtime_error("failed writing output image");
}
Renderer::Renderer(Framebuffer& target) : target_(target) {}
void Renderer::clear(Color c) { target_.clear(c); }

namespace {
struct Vertex { float x, y, z; };
float edge(const Vertex& a, const Vertex& b, float x, float y) {
    return (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x);
}
}
void Renderer::drawCube(Vec3 center, float size, float angle, Color color) {
    if (!(size > 0.0F) || !std::isfinite(size) || !std::isfinite(angle))
        throw std::invalid_argument("cube size and angle must be finite; size must be positive");
    const float h = size * 0.5F;
    const Vertex local[8] = {
        {-h,-h,-h},{h,-h,-h},{h,h,-h},{-h,h,-h},
        {-h,-h,h},{h,-h,h},{h,h,h},{-h,h,h}
    };
    const int faces[12][3] = {
        {0,1,2},{0,2,3},{4,6,5},{4,7,6},
        {0,4,5},{0,5,1},{3,2,6},{3,6,7},
        {0,3,7},{0,7,4},{1,5,6},{1,6,2}
    };
    const float cs = std::cos(angle), sn = std::sin(angle);
    Vertex p[8];
    for (int i = 0; i < 8; ++i) {
        const float rx = local[i].x * cs - local[i].z * sn + center.x;
        const float rz = local[i].x * sn + local[i].z * cs + center.z;
        const float ry = local[i].y + center.y;
        const float depth = rz + 4.0F;
        const float safe = std::max(depth, 0.1F);
        p[i] = {target_.width() * 0.5F + rx * (target_.height() * 0.9F) / safe,
                target_.height() * 0.5F - ry * (target_.height() * 0.9F) / safe, depth};
    }
    const auto count = static_cast<std::size_t>(target_.width()) * static_cast<std::size_t>(target_.height());
    std::vector<float> zbuffer(count, std::numeric_limits<float>::infinity());
    for (const auto& f : faces) {
        const Vertex a = p[f[0]], b = p[f[1]], c = p[f[2]];
        const float area = edge(a, b, c.x, c.y);
        if (std::abs(area) < 0.0001F) continue;
        const int minX = std::max(0, static_cast<int>(std::floor(std::min({a.x,b.x,c.x}))));
        const int maxX = std::min(target_.width()-1, static_cast<int>(std::ceil(std::max({a.x,b.x,c.x}))));
        const int minY = std::max(0, static_cast<int>(std::floor(std::min({a.y,b.y,c.y}))));
        const int maxY = std::min(target_.height()-1, static_cast<int>(std::ceil(std::max({a.y,b.y,c.y}))));
        for (int y = minY; y <= maxY; ++y) for (int x = minX; x <= maxX; ++x) {
            const float px = x + 0.5F, py = y + 0.5F;
            const float wa = edge(b,c,px,py)/area;
            const float wb = edge(c,a,px,py)/area;
            const float wc = edge(a,b,px,py)/area;
            if (wa < 0 || wb < 0 || wc < 0) continue;
            const float z = wa*a.z + wb*b.z + wc*c.z;
            const auto index = static_cast<std::size_t>(y) * target_.width() + x;
            if (z >= zbuffer[index]) continue;
            zbuffer[index] = z;
            const float shade = 0.55F + 0.45F * std::max(0.0F, 1.0F - (z-3.0F)*0.12F);
            target_.pixels_[index*3] = static_cast<std::uint8_t>(std::min(255.0F, color.r*shade));
            target_.pixels_[index*3+1] = static_cast<std::uint8_t>(std::min(255.0F, color.g*shade));
            target_.pixels_[index*3+2] = static_cast<std::uint8_t>(std::min(255.0F, color.b*shade));
        }
    }
}
} // namespace xzuff3d
