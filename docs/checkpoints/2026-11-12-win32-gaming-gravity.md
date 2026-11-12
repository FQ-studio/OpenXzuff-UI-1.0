# Win32 Gaming / Gravity port checkpoint

Date: 2026-11-12
Branch: `feature/win32-gaming-gravity-port`
Base: `main`

## Completed in this checkpoint

- Created the isolated design specification at `docs/design/win32-gaming-gravity-port.md`.
- Defined a Linux-free runtime boundary: Win32 compatibility code must call a Gravity/OpenXzuff host adapter.
- Recorded the Linux-to-Gravity replacement map and implementation phases.
- Defined CI acceptance gates and prohibited unsupported compatibility claims.
- Kept OpenXzuff's native target RV64; x86/x64 Windows execution requires an explicit translation subsystem.

## Not implemented / not verified

- No Wine or Proton source has been imported or ported.
- No Gravity host ABI adapter has been implemented.
- No PE loader or x86/x64-to-RV64 translator exists from this checkpoint.
- No graphics/audio/input backend has been connected.
- No build, unit test, QEMU test or game compatibility test has been run for Win32 Gaming.
- No package size has been measured. The previously discussed 100–300 MB figure remains a planning target only.

## Next engineering action

Audit the actual Gravity syscall and OpenXzuff service interfaces, then implement a versioned host ABI contract and host mock tests before importing any Wine/Proton code.
