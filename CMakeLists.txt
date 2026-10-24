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

# User-facing Xzuff Go CLI. It dispatches check/run/build/create/delete commands.
add_executable(xgo language/xzuff-go/cli/xgo.cpp)
target_compile_options(xgo PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME xgo-cli-help COMMAND xgo --help)

# Coder Checker is built from the same lexer/parser as the compiler to prevent grammar drift.
add_executable(xzuff-check language/xzuff-go/compiler/xzuff_go.cpp)
target_compile_definitions(xzuff-check PRIVATE XZUFF_CHECKER_MODE=1)
target_compile_options(xzuff-check PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME xzuff-check-valid-source
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/return_42.xgo
)
add_test(NAME xzuff-check-json-output
    COMMAND xzuff-check --json ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/return_42.xgo
)
add_test(NAME xzuff-check-rejects-out-of-range-i32
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/checker_invalid_i32.xgo
)
set_tests_properties(xzuff-check-rejects-out-of-range-i32 PROPERTIES WILL_FAIL TRUE)

add_executable(xexe-loader-test tests/xexe_loader_test.cpp)
target_link_libraries(xexe-loader-test PRIVATE xzuff_os_base)
target_compile_options(xexe-loader-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-xexe-validation COMMAND xexe-loader-test)

# End-to-end check: compile the return_42 fixture, then validate its emitted .xexe.
add_test(NAME xzuff-go-compile-fixture
    COMMAND xzuff-go
        ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/return_42.xgo
        ${CMAKE_CURRENT_BINARY_DIR}/return_42.xexe
)
add_test(NAME xzuff-go-xexe-validation
    COMMAND xexe-loader-test ${CMAKE_CURRENT_BINARY_DIR}/return_42.xexe
)
set_tests_properties(xzuff-go-xexe-validation PROPERTIES DEPENDS xzuff-go-compile-fixture)
