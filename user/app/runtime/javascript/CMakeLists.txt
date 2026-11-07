cmake_minimum_required(VERSION 3.20)
project(OpenXzuffJavaScriptRuntime VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(FetchContent)
include(CTest)

# Pin the upstream engine. This host prototype does not imply RV64 or OS-ABI support.
set(QJS_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(QJS_ENABLE_INSTALL OFF CACHE BOOL "" FORCE)
FetchContent_Declare(
  quickjs
  GIT_REPOSITORY https://github.com/quickjs-ng/quickjs.git
  GIT_TAG v0.18.0
  GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(quickjs)

add_executable(xzuff-js src/main.cpp)
target_link_libraries(xzuff-js PRIVATE qjs)
target_compile_options(xzuff-js PRIVATE
  $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>
)

if(BUILD_TESTING)
  add_test(
    NAME xzuff-js-host-smoke
    COMMAND xzuff-js ${CMAKE_CURRENT_SOURCE_DIR}/tests/smoke.js
  )
  add_test(
    NAME xzuff-js-timeout-guard
    COMMAND xzuff-js --timeout-ms 200 ${CMAKE_CURRENT_SOURCE_DIR}/tests/infinite-loop.js
  )
  set_tests_properties(xzuff-js-timeout-guard PROPERTIES WILL_FAIL TRUE TIMEOUT 5)
endif()
