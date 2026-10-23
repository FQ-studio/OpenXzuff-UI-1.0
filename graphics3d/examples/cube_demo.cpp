#include "xzuff3d/renderer.hpp"
#include <exception>
#include <iostream>

int main(int argc,char** argv) {
    try {
        const char* output=argc>1?argv[1]:"xzuff3d-scene.ppm";
        xzuff3d::Framebuffer fb(640,480);
        xzuff3d::Renderer renderer(fb);
        renderer.clear({10,14,26});
        renderer.setAmbientLight(0.25F);
        renderer.setDirectionalLight({{-0.5F,-1.0F,-0.4F},0.85F,{220,235,255}});
        renderer.addPointLight({{2.0F,2.0F,-1.0F},7.0F,10.0F,{100,180,255}});
        auto cube=xzuff3d::Mesh::cube(1.5F);
        xzuff3d::Transform transform;
        transform.position={0,0,0};
        transform.rotation={0.45F,0.65F,0.1F};
        xzuff3d::Material material;
        material.baseColor={35,170,255};
        xzuff3d::Camera camera;
        camera.position={0,0,-4.5F}; camera.target={0,0,0};
        renderer.renderMesh(cube,transform,camera,material);
        fb.writePpm(output);
        std::cout<<"Rendered mesh scene to "<<output<<'\n';
        return 0;
    } catch(const std::exception& e) {
        std::cerr<<"OpenXzuff3D error: "<<e.what()<<'\n';
        return 1;
    }
}
