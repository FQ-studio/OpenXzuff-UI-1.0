cmake_minimum_required(VERSION 3.20)
project(OpenXzuffUI VERSION 1.0.0 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
enable_testing()

add_library(xzuff_os_base
    Kernel/src/boot_contract.cpp
    Kernel/src/xexe_loader.cpp
    sys/src/service_registry.cpp
    gui/src/framebuffer.cpp
)
target_include_directories(xzuff_os_base PUBLIC Kernel/include sys/include user/app/include gui/include)
target_compile_options(xzuff_os_base PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)

add_executable(openxzuff-os-smoke tests/os_smoke.cpp)
target_link_libraries(openxzuff-os-smoke PRIVATE xzuff_os_base)
target_compile_options(openxzuff-os-smoke PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME openxzuff-os-smoke COMMAND openxzuff-os-smoke)

# First Xzuff Go compiler milestone: a deliberately small, testable RV64 subset.
add_executable(xzuff-go language/xzuff-go/compiler/xzuff_go.cpp)
target_compile_options(xzuff-go PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME xzuff-go-codegen-self-test COMMAND xzuff-go --self-test)

add_executable(xexe-loader-test tests/xexe_loader_test.cpp)
target_link_libraries(xexe-loader-test PRIVATE xzuff_os_base)
target_compile_options(xexe-loader-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-xexe-validation COMMAND xexe-loader-test)
