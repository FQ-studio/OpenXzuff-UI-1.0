# OpenXzuff 3D

A portable C++20 software 3D renderer foundation for OpenXzuff UI.

## First milestone
- RGB framebuffer abstraction
- Perspective-projected cube demo
- Triangle rasterization and depth buffer
- PPM output for hosted smoke tests

## Build

```sh
cmake -S graphics3d -B build/graphics3d -DCMAKE_BUILD_TYPE=Release
cmake --build build/graphics3d --parallel 2
./build/graphics3d/xzuff3d-cube cube.ppm
```

This is a CPU renderer foundation, not yet a GPU driver or complete 3D engine. Follow-up work is mesh/material/camera APIs, Luau bindings, and a framebuffer adapter for Gravity Kernel RV64.