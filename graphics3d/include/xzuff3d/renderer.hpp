#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace xzuff3d {

struct Vec3 {
    float x{}, y{}, z{};
    Vec3 operator+(Vec3 b) const noexcept { return {x+b.x,y+b.y,z+b.z}; }
    Vec3 operator-(Vec3 b) const noexcept { return {x-b.x,y-b.y,z-b.z}; }
    Vec3 operator*(float s) const noexcept { return {x*s,y*s,z*s}; }
    Vec3 operator/(float s) const noexcept { return {x/s,y/s,z/s}; }
};
struct Color { std::uint8_t r{}, g{}, b{}; };
struct Vertex { Vec3 position; Vec3 normal; };
struct Triangle { std::uint32_t a{}, b{}, c{}; };

struct Transform {
    Vec3 position{0,0,0};
    Vec3 rotation{0,0,0}; // Euler angles in radians: pitch, yaw, roll
    Vec3 scale{1,1,1};
    Vec3 applyPoint(Vec3 p) const noexcept;
    Vec3 applyDirection(Vec3 n) const noexcept;
};

struct Mesh {
    std::vector<Vertex> vertices;
    std::vector<Triangle> triangles;
    static Mesh cube(float size = 1.0F);
};

struct Camera {
    Vec3 position{0,0,-4};
    Vec3 target{0,0,0};
    Vec3 up{0,1,0};
    float verticalFovRadians{1.04719755F};
    float nearPlane{0.1F};
    float farPlane{1000.0F};
};

struct Material {
    Color baseColor{35,170,255};
    float ambient{0.18F};
    float diffuse{0.82F};
};
struct DirectionalLight {
    Vec3 direction{-0.4F,-0.8F,-0.5F}; // direction rays travel
    float intensity{0.9F};
    Color color{255,255,255};
};
struct PointLight {
    Vec3 position{2,3,-2};
    float intensity{8.0F};
    float range{12.0F};
    Color color{255,255,255};
};

class Framebuffer {
public:
    Framebuffer(int width, int height);
    ~Framebuffer();
    Framebuffer(const Framebuffer&) = delete;
    Framebuffer& operator=(const Framebuffer&) = delete;
    Framebuffer(Framebuffer&& other) noexcept;
    Framebuffer& operator=(Framebuffer&& other) noexcept;
    int width() const noexcept { return width_; }
    int height() const noexcept { return height_; }
    const std::uint8_t* pixels() const noexcept { return pixels_; }
    std::uint8_t* pixels() noexcept { return pixels_; }
    void clear(Color color);
    void writePpm(const std::string& path) const;
private:
    int width_{};
    int height_{};
    std::uint8_t* pixels_{};
    friend class Renderer;
};

class Renderer {
public:
    explicit Renderer(Framebuffer& target);
    void clear(Color color = {12,16,28});
    void setAmbientLight(float intensity);
    void setDirectionalLight(const DirectionalLight& light);
    void clearPointLights();
    void addPointLight(const PointLight& light);
    void renderMesh(const Mesh& mesh, const Transform& transform,
                    const Camera& camera, const Material& material);
    void drawCube(Vec3 center, float size, float angleRadians,
                  Color color = {35,170,255});
private:
    Framebuffer& target_;
    std::vector<float> depth_;
    float ambientLight_{0.2F};
    DirectionalLight directional_{};
    std::vector<PointLight> points_;
};

} // namespace xzuff3d
