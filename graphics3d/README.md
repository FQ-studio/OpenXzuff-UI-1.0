# OpenXzuff3D

OpenXzuff3D is a dependency-free C++20 CPU software-rendering module for OpenXzuff UI.

## Implemented milestones (hosted renderer)

### Phase 1 — Geometry and transforms
- Vec3 operations and Euler rotation / scale / translation transforms
- Indexed triangle meshes and a cube mesh generator
- Vertex normals and inverse-scale normal transformation
- Bounds-checked mesh indices

### Phase 2 — Camera and render pipeline
- Look-at camera basis and perspective projection
- Near/far view-space polygon clipping
- Triangle rasterization with a persistent per-frame depth buffer
- Framebuffer clear, move support, and binary PPM output

### Phase 3 — Basic materials and lighting
- Base material color with ambient and diffuse coefficients
- Ambient light, directional light and range-attenuated point lights
- Interpolated vertex normals for diffuse shading

## Build and run

```sh
cmake -S graphics3d -B graphics3d/build -DCMAKE_BUILD_TYPE=Release
cmake --build graphics3d/build --parallel 2
graphics3d/build/xzuff3d-cube graphics3d/build/scene.ppm
```

## Scope and limitations

This is a more capable **CPU renderer**, not a finished commercial-grade engine. It does not yet provide texture mapping, physically based materials, shadow maps, skeletal animation, a GPU backend, scene graph, or Luau bindings. The framebuffer is currently a host-memory buffer; a Gravity Kernel RV64 display adapter and bare-metal integration remain separate work. CI compilation and smoke tests validate the hosted target, not booting on hardware.
