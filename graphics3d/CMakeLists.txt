cmake_minimum_required(VERSION 3.24)
project(OpenXzuff3D LANGUAGES CXX)

add_library(xzuff3d src/renderer.cpp)
target_include_directories(xzuff3d PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_compile_features(xzuff3d PUBLIC cxx_std_20)
if(MSVC)
  target_compile_options(xzuff3d PRIVATE /W4 /permissive-)
else()
  target_compile_options(xzuff3d PRIVATE -Wall -Wextra -Wpedantic)
endif()

add_executable(xzuff3d-cube examples/cube_demo.cpp)
target_link_libraries(xzuff3d-cube PRIVATE xzuff3d)
