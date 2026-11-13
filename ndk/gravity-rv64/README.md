# Gravity NDK — RV64 freestanding C starter

The initial NDK is a CMake cross-toolchain for `riscv64-unknown-elf-gcc`. It uses the Gravity SDK project headers from `sdk/gravity/sysroot/usr/include` and applies the RV64IMAC / LP64 freestanding compilation contract.

## Configure and build

```sh
cmake -S ndk/gravity-rv64/examples -B build/gravity-ndk \
  -DCMAKE_TOOLCHAIN_FILE="$PWD/ndk/gravity-rv64/toolchain.cmake"
cmake --build build/gravity-ndk
```

The sample is an object library by design. CI disassembles the resulting RV64 object and verifies that the syscall wrappers emit `ecall` instructions.

## Current boundary

This NDK currently compiles freestanding C objects only. It does not yet provide C++, libc, CRT/startup objects, a production linker, relocation processing, or an application runner. A linkable application pipeline must wait for a compatible `.xexe` segment/memory contract and a general Gravity Kernel user-process loader. The toolchain must not be described as a complete NDK yet.
