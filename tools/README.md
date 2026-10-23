# OpenXzuff native C++ package installer

The LAP/LUAP installer is written in C++17 and uses the standard library and
filesystem APIs; no Python, Linux, Flutter, or Lua runtime is needed to build
or run this installer utility on a supported host.

Build and test with `make check-installer`. Install with
`build/oxz-install install MyApp.lap ./installed-apps`.

The initial ZIP reader supports ZIP_STORED only. Deflated entries, ZIP64,
encrypted entries and multi-disk archives are rejected. It validates basic
manifest fields, the declared entry script, archive paths, size limits and CRC32,
stages files, and refuses to overwrite an existing app. It does not execute
scripts, verify signatures, provide an app sandbox, or integrate with Gravity
Kernel yet. This is a native C++ host utility foundation, not a completed OS GUI installer.
