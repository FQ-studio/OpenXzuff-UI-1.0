# Lua runtime integration

OpenXzuff UI uses Lua for the native GUI and apps. The current Lua code is host-agnostic and runs under a system Lua interpreter for tests; it is not yet embedded in the kernel. Next steps: vendor or reproducibly fetch a pinned Lua 5.4 source release, define allocator/stdio/module-loading boundaries, build freestanding where practical, and implement a platform host for framebuffer drawing and input. Do not claim Lua is running on bare metal until the runtime is linked and executed there.
