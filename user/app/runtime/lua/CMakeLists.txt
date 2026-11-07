cmake_minimum_required(VERSION 3.24)
project(XzuffLuaRuntime VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(FetchContent)
FetchContent_Declare(
  LuaSource
  URL https://www.lua.org/ftp/lua-5.5.1.tar.gz
  URL_HASH SHA256=1c4b4068d67061f2a2231ad2b5422e77acea1487ea9890f6320af614f4373dce
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
)
FetchContent_GetProperties(LuaSource)
if(NOT luasource_POPULATED)
  FetchContent_Populate(LuaSource)
endif()
set(LUA_SRC ${luasource_SOURCE_DIR}/src)
add_library(xzuff-lua-core STATIC
  ${LUA_SRC}/lapi.c ${LUA_SRC}/lcode.c ${LUA_SRC}/lctype.c
  ${LUA_SRC}/ldebug.c ${LUA_SRC}/ldo.c ${LUA_SRC}/ldump.c
  ${LUA_SRC}/lfunc.c ${LUA_SRC}/lgc.c ${LUA_SRC}/llex.c
  ${LUA_SRC}/lmem.c ${LUA_SRC}/lobject.c ${LUA_SRC}/lopcodes.c
  ${LUA_SRC}/lparser.c ${LUA_SRC}/lstate.c ${LUA_SRC}/lstring.c
  ${LUA_SRC}/ltable.c ${LUA_SRC}/ltm.c ${LUA_SRC}/lundump.c
  ${LUA_SRC}/lvm.c ${LUA_SRC}/lzio.c ${LUA_SRC}/lauxlib.c
  ${LUA_SRC}/lbaselib.c ${LUA_SRC}/lcorolib.c ${LUA_SRC}/lmathlib.c
  ${LUA_SRC}/lstrlib.c ${LUA_SRC}/ltablib.c ${LUA_SRC}/lutf8lib.c
)
target_include_directories(xzuff-lua-core PUBLIC ${LUA_SRC})
add_executable(xzuff-lua src/main.cpp)
target_link_libraries(xzuff-lua PRIVATE xzuff-lua-core)
if(MSVC)
  target_compile_options(xzuff-lua PRIVATE /W4 /permissive-)
else()
  target_compile_options(xzuff-lua PRIVATE -Wall -Wextra -Wpedantic)
endif()

enable_testing()
add_test(NAME lua-host-smoke COMMAND xzuff-lua ${CMAKE_CURRENT_SOURCE_DIR}/tests/smoke.luap)
add_test(NAME lua-instruction-guard COMMAND xzuff-lua ${CMAKE_CURRENT_SOURCE_DIR}/tests/infinite-loop.luap)
set_tests_properties(lua-instruction-guard PROPERTIES WILL_FAIL TRUE TIMEOUT 5)
