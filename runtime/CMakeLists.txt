cmake_minimum_required(VERSION 3.24)
project(XzuffLuauRuntime VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(FetchContent)
set(LUAU_BUILD_CLI OFF CACHE BOOL "" FORCE)
set(LUAU_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(LUAU_BUILD_WEB OFF CACHE BOOL "" FORCE)
FetchContent_Declare(
  Luau
  GIT_REPOSITORY https://github.com/luau-lang/luau.git
  GIT_TAG 0.728
  GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(Luau)

add_executable(xzuff-luau src/main.cpp)
target_link_libraries(xzuff-luau PRIVATE Luau.VM Luau.Compiler Luau.Ast)
target_include_directories(xzuff-luau PRIVATE ${luau_SOURCE_DIR}/VM/include ${luau_SOURCE_DIR}/VM/src ${luau_SOURCE_DIR}/Compiler/include ${luau_SOURCE_DIR}/Ast/include)
if(MSVC)
  target_compile_options(xzuff-luau PRIVATE /W4 /permissive-)
else()
  target_compile_options(xzuff-luau PRIVATE -Wall -Wextra -Wpedantic)
endif()
