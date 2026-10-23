#include "xzuff3d/renderer.hpp"

#include <cmath>
#include <cstdlib>
#include <iostream>
#include <stdexcept>

namespace {
void require(bool condition,const char* message) {
    if(!condition) throw std::runtime_error(message);
}
}
int main() {
    try {
        const auto cube=xzuff3d::Mesh::cube(2.0F);
        require(cube.vertices.size()==8,"cube should have eight shared vertices");
        require(cube.triangles.size()==12,"cube should have twelve triangles");

        xzuff3d::Transform transform;
        transform.position={1,2,3};
        transform.scale={2,2,2};
        const auto p=transform.applyPoint({1,0,0});
        require(std::abs(p.x-3.0F)<0.001F&&std::abs(p.y-2.0F)<0.001F&&std::abs(p.z-3.0F)<0.001F,
                "transform must apply scale and translation");

        xzuff3d::Framebuffer fb(160,120);
        xzuff3d::Renderer renderer(fb);
        renderer.clear({2,3,4});
        renderer.setAmbientLight(0.3F);
        renderer.setDirectionalLight({{-0.2F,-0.8F,-0.4F},0.8F,{255,255,255}});
        renderer.addPointLight({{2,2,-2},4.0F,8.0F,{255,255,255}});
        xzuff3d::Camera camera;
        camera.position={0,0,-4}; camera.target={0,0,0};
        xzuff3d::Material material; material.baseColor={30,160,240};
        renderer.renderMesh(cube,{},camera,material);

        const auto* pixels=fb.pixels();
        std::size_t changed=0;
        for(std::size_t i=0;i<static_cast<std::size_t>(fb.width())*fb.height();++i) {
            if(pixels[i*3]!=2||pixels[i*3+1]!=3||pixels[i*3+2]!=4) ++changed;
        }
        require(changed>100,"render should modify a visible region of the framebuffer");

        renderer.clear({7,8,9});
        require(fb.pixels()[0]==7&&fb.pixels()[1]==8&&fb.pixels()[2]==9,"clear must reset framebuffer pixels");
        bool rejected=false;
        try { (void)xzuff3d::Mesh::cube(0.0F); } catch(const std::invalid_argument&) { rejected=true; }
        require(rejected,"zero-sized cube must be rejected");

        std::cout<<"OpenXzuff3D tests passed\n";
        return EXIT_SUCCESS;
    } catch(const std::exception& e) {
        std::cerr<<"OpenXzuff3D test failure: "<<e.what()<<'\n';
        return EXIT_FAILURE;
    }
}
