# Early freestanding C toolchain for Gravity Kernel RV64 applications.
# This config compiles relocatable objects only; there is no application linker yet.
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR riscv64)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(GRAVITY_NDK_ROOT "${CMAKE_CURRENT_LIST_DIR}")
get_filename_component(GRAVITY_REPO_ROOT "${CMAKE_CURRENT_LIST_DIR}/../.." ABSOLUTE)
set(GRAVITY_SYSROOT "${GRAVITY_REPO_ROOT}/sdk/gravity/sysroot")

set(CMAKE_C_COMPILER riscv64-unknown-elf-gcc)
set(CMAKE_ASM_COMPILER riscv64-unknown-elf-gcc)
set(CMAKE_C_FLAGS_INIT
    "-march=rv64imac -mabi=lp64 -ffreestanding -fno-builtin -fno-pic -Wall -Wextra -Werror -I${GRAVITY_SYSROOT}/usr/include")

set(CMAKE_FIND_ROOT_PATH "${GRAVITY_SYSROOT}")
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
