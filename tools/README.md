# OpenXzuff native package installer

C11 utility with no Python, Lua, Linux, Flutter, or third-party library dependency at build time. Build/test with make check-installer; install with build/oxz-install install MyApp.lap ./installed-apps.

The initial ZIP reader deliberately supports ZIP_STORED only. Deflated ZIP entries, ZIP64, encrypted entries and multi-disk archives are rejected. Package creators must use uncompressed ZIP entries. The utility checks basic manifest fields, paths, size limits and CRC32, stages files, and refuses to overwrite an existing app. It does not execute scripts, verify signatures, provide a sandbox, or integrate with Gravity Kernel yet. This is a native C host utility foundation, not a completed OS GUI installer.
