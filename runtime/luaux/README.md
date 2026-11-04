# LuauX Runtime — RV64

LuauX is the OpenXzuff UI runtime integration based on upstream Luau: https://github.com/luau-lang/luau

## Target policy
- OpenXzuff target: RV64 / riscv64gc only.
- Reuse the upstream Luau compiler and bytecode VM/interpreter.
- Exclude x86_64 and AArch64 JIT backends from the OpenXzuff RV64 build.
- Develop a custom FQ Solo Studio RV64 JIT backend.
- Never fall back to another architecture's native-code backend.
- Keep the bytecode interpreter as fallback when RV64 JIT is unavailable.

## Component status
- Upstream compiler: planned integration; not vendored or built here yet.
- Upstream VM/interpreter: planned integration; not vendored or built here yet.
- Custom RV64 JIT backend: design decision only; implementation not present yet.
- x86_64 and AArch64 JIT backends: excluded from the OpenXzuff RV64 build.
- Gravity Kernel bindings: not implemented yet.

## Size policy
Only RV64 runtime components should be compiled and linked into the OpenXzuff UI image. Leaving unused backend source files in an upstream checkout does not itself increase the final image; compiling and linking their objects and dependencies would.

## RV64 JIT requirements
Before enabling the JIT, implement and test RV64 instruction emission, register allocation, the selected calling convention and Gravity Kernel ABI boundary, relocations, executable-memory permissions, instruction-cache synchronization, VM entry/exit handling, and interpreter-versus-JIT correctness under RV64 QEMU. Apply memory quotas and sandbox restrictions to untrusted .lap scripts. Do not claim JIT support until these tests pass.

## Current status
This file records the RV64-only LuauX architecture and separates it from the existing standard-Lua .luap documentation. It does not vendor Luau, compile the upstream VM, implement the RV64 JIT, or claim runtime tests have passed. Those require separate source integration and CI work.
