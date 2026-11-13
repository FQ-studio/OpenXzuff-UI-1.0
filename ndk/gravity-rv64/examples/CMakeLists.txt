cmake_minimum_required(VERSION 3.20)
project(GravityRv64NdkSmoke LANGUAGES C)

get_filename_component(GRAVITY_REPO_ROOT "${CMAKE_CURRENT_LIST_DIR}/../../.." ABSOLUTE)

# Object-only target: the current ABI starter has no CRT or application linker.
add_library(gravity-abi-smoke OBJECT
    "${GRAVITY_REPO_ROOT}/sdk/gravity/examples/abi_smoke.c"
)
set_target_properties(gravity-abi-smoke PROPERTIES C_STANDARD 11 C_STANDARD_REQUIRED ON)
