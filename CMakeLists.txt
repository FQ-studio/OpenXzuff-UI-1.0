cmake_minimum_required(VERSION 3.20)
project(OpenXzuffUI VERSION 1.0.0 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
enable_testing()

set(GRAVITY_SDK_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/sdk/gravity/sysroot/usr/include")

add_library(xzuff_os_base
    Kernel/src/boot_contract.cpp
    Kernel/src/sbi.cpp
    Kernel/src/xexe_loader.cpp
    Kernel/src/syscall.cpp
    Kernel/arch/riscv64/trap_dispatch.cpp
    sys/src/service_registry.cpp
    gui/src/framebuffer.cpp
    drivers/serial/uart.cpp
)
target_include_directories(xzuff_os_base PUBLIC
    Kernel/include sys/include user/app/include gui/include drivers/serial
    "${GRAVITY_SDK_INCLUDE}"
)
target_compile_options(xzuff_os_base PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)

add_executable(openxzuff-os-smoke tests/os_smoke.cpp)
target_link_libraries(openxzuff-os-smoke PRIVATE xzuff_os_base)
target_compile_options(openxzuff-os-smoke PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME openxzuff-os-smoke COMMAND openxzuff-os-smoke)

# Xzuff Go compiles a deliberately limited, testable RV64 constant-expression subset.
add_executable(xzuff-go language/xzuff-go/compiler/xzuff_go.cpp)
target_include_directories(xzuff-go PRIVATE "${GRAVITY_SDK_INCLUDE}")
target_compile_options(xzuff-go PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME xzuff-go-codegen-self-test COMMAND xzuff-go --self-test)

# User-facing Xzuff Go CLI. It dispatches check/run/build/create/delete commands.
add_executable(xgo language/xzuff-go/cli/xgo.cpp)
target_compile_options(xgo PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME xgo-cli-help COMMAND xgo --help)
add_test(NAME xgo-cli-command-smoke
    COMMAND ${CMAKE_COMMAND}
        -DXGO=$<TARGET_FILE:xgo>
        "-DWORK_DIR=${CMAKE_CURRENT_BINARY_DIR}/Cli Smoke Parent/CliSmoke"
        "-DPROJECT_PARENT=${CMAKE_CURRENT_BINARY_DIR}/Cli Smoke Parent"
        -P ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/xgo_cli_smoke.cmake
)
set_tests_properties(xgo-cli-command-smoke PROPERTIES DEPENDS xgo-cli-help)

# Coder Checker shares the compiler's lexer and parser so its grammar cannot drift.
add_executable(xzuff-check language/xzuff-go/compiler/xzuff_go.cpp)
target_include_directories(xzuff-check PRIVATE "${GRAVITY_SDK_INCLUDE}")
target_compile_definitions(xzuff-check PRIVATE XZUFF_CHECKER_MODE=1)
target_compile_options(xzuff-check PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME xzuff-check-valid-source
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/return_42.xgo
)
add_test(NAME xzuff-check-json-output
    COMMAND xzuff-check --json ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/return_42.xgo
)
add_test(NAME xzuff-check-constant-expression
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/examples/rv64_constant_expression.xgo
)
add_test(NAME xzuff-check-rejects-out-of-range-i32
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/checker_invalid_i32.xgo
)
set_tests_properties(xzuff-check-rejects-out-of-range-i32 PROPERTIES WILL_FAIL TRUE)
add_test(NAME xzuff-check-rejects-division-by-zero
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/checker_division_by_zero.xgo
)
set_tests_properties(xzuff-check-rejects-division-by-zero PROPERTIES WILL_FAIL TRUE)
add_test(NAME xzuff-check-rejects-i64-overflow
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/checker_i64_overflow.xgo
)
set_tests_properties(xzuff-check-rejects-i64-overflow PROPERTIES WILL_FAIL TRUE)
add_test(NAME xzuff-check-rejects-malformed-expression
    COMMAND xzuff-check ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/tests/checker_malformed_expression.xgo
)
set_tests_properties(xzuff-check-rejects-malformed-expression PROPERTIES WILL_FAIL TRUE)

add_executable(xexe-loader-test tests/xexe_loader_test.cpp)
target_link_libraries(xexe-loader-test PRIVATE xzuff_os_base)
target_compile_options(xexe-loader-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-xexe-validation COMMAND xexe-loader-test)

# End-to-end compiler output checks validate both the shared .xexe header and RV64 code bytes.
add_test(NAME xzuff-go-compile-fixture
    COMMAND xzuff-go
        ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/examples/rv64_return_42.xgo
        ${CMAKE_CURRENT_BINARY_DIR}/return_42.xexe
)
add_test(NAME xzuff-go-xexe-validation
    COMMAND xexe-loader-test ${CMAKE_CURRENT_BINARY_DIR}/return_42.xexe 42
)
set_tests_properties(xzuff-go-xexe-validation PROPERTIES DEPENDS xzuff-go-compile-fixture)

add_test(NAME xzuff-go-compile-expression-fixture
    COMMAND xzuff-go
        ${CMAKE_CURRENT_SOURCE_DIR}/language/xzuff-go/examples/rv64_constant_expression.xgo
        ${CMAKE_CURRENT_BINARY_DIR}/constant_expression.xexe
)
add_test(NAME xzuff-go-expression-xexe-validation
    COMMAND xexe-loader-test ${CMAKE_CURRENT_BINARY_DIR}/constant_expression.xexe 33
)
set_tests_properties(xzuff-go-expression-xexe-validation PROPERTIES DEPENDS xzuff-go-compile-expression-fixture)

# SBI wrapper can be host-smoke-tested without executing privileged ecalls.
add_executable(sbi-smoke tests/sbi_smoke.cpp)
target_link_libraries(sbi-smoke PRIVATE xzuff_os_base)
target_compile_options(sbi-smoke PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-sbi-wrapper-smoke COMMAND sbi-smoke)

add_executable(syscall-dispatch-test tests/syscall_dispatch_test.cpp)
target_link_libraries(syscall-dispatch-test PRIVATE xzuff_os_base)
target_compile_options(syscall-dispatch-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-syscall-dispatch COMMAND syscall-dispatch-test)

add_executable(trap-frame-test tests/trap_frame_test.cpp)
target_include_directories(trap-frame-test PRIVATE Kernel/include)
target_compile_options(trap-frame-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-rv64-trap-frame-layout COMMAND trap-frame-test)

add_executable(rv64-trap-dispatch-test tests/rv64_trap_dispatch_test.cpp)
target_link_libraries(rv64-trap-dispatch-test PRIVATE xzuff_os_base)
target_compile_options(rv64-trap-dispatch-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-rv64-user-trap-dispatch COMMAND rv64-trap-dispatch-test)
