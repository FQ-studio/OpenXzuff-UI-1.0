# Gravity Application Package (GAPP) v1

GAPP is the initial metadata format for native OpenXzuff/Gravity applications.
The current implementation validates a package header only; it does **not**
load, map, link, or execute applications.

All multibyte fields are unsigned little-endian values. The fixed header is
48 bytes, followed immediately by the application image.

| Offset | Size | Field | Required value / meaning |
|---:|---:|---|---|
| 0 | 4 | Magic | ASCII `GAPP` |
| 4 | 2 | Version | `1` |
| 6 | 2 | Header size | `48` |
| 8 | 4 | Architecture | `0x34365652` (RV64 identifier) |
| 12 | 4 | Flags | `0` (no flags defined yet) |
| 16 | 8 | Entry offset | Offset into image; must be less than image size |
| 24 | 8 | Image size | Non-zero bytes; must fit within the supplied file after the header |
| 32 | 8 | Memory size | Required image memory; must be at least image size |
| 40 | 8 | Reserved | Must be zero |

The validator rejects truncated headers, unsupported versions/architectures,
unknown flags, nonzero reserved bytes, empty or truncated images, invalid entry
offsets, and memory sizes smaller than the image. The caller must still treat
the file as untrusted.

## Not implemented yet

- ELF parsing, relocations, dynamic linking, and executable page permissions
- User address spaces, page tables, process creation, scheduling, and exit
- A safe U-mode trap stack switch and syscall access checks
- Filesystem/package installation, signatures, and permissions
- Lua VM integration, ART Minimal, Win32 Basic, and graphical services

GAPP v1 is a small, testable format foundation, not an executable-app runtime.
