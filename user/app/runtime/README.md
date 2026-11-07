# Application Runtimes

Reserved for the Lua and Luau runtime adapters used by user applications. Keep Standard Lua `.luap` and Luau `.lap` formats distinct.

Status: scaffold only; connect to the existing runtime code only after its API and dependencies are verified.

The `runtime/` directory currently hosts the Luau runtime. The separate standard Lua runtime belongs under `user/app/runtime/lua/` and must own `.luap`; do not route `.luap` files through the Luau VM.
