# Graphical LAP/LUAP installer

runtime/lua/apps/package_installer.lua provides a graphical installer screen using the OpenXzuff Lua drawing and pointer-event contract. It renders supported package types, selected package, Choose package and Install app controls, status feedback, and touch/pointer hit testing.

## Host integration required

The drawing host supplies fill_rect and draw_text. To enable actual installation, the host must also provide:

- choose_package(extensions): open a trusted native file picker and return a selected path or nil.
- install_package(path): invoke the native C++ installer/filesystem service and return true on success, or false plus a reason on failure.

These callbacks are host-provided so Lua UI code does not get unrestricted filesystem access. The current C++ CLI installer is not automatically callable from this screen, and Gravity Kernel has not yet implemented a filesystem, GUI event loop, or Lua VM integration. Until those native services are connected, the screen displays an unavailable status rather than pretending installation succeeded.

The native C++ installer currently supports ZIP_STORED .lap and .luap packages only. It does not execute scripts, verify signatures, or sandbox apps. Treat packages as untrusted.

## Test

Run lua tests/package_installer_spec.lua and make check-lua (with Lua and luac installed).
