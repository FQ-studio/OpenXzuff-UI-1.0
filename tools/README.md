# OpenXzuff Package Installer (host-side foundation)

This first installer validates and extracts `.lap` (Luau App Package) and
`.luap` (Lua App Package) ZIP containers into a local application directory.
It uses only the Python 3 standard library.

## Use

Validate without installing:

```sh
python3 tools/oxz-install.py check MyApp.lap
```

Install:

```sh
python3 tools/oxz-install.py install MyApp.lap --root ./installed-apps
python3 tools/oxz-install.py install Calculator.luap --root ./installed-apps
```

Replacing an installed app with the same ID is refused by default. To explicitly
replace it, pass `--replace`.

## Safety and limitations

- Checks package extension, manifest schema fields, language/format agreement,
  entry script, semantic version, duplicate paths/JSON keys, path traversal,
  symbolic links, encrypted entries, compression types, and size limits.
- Rejects all non-empty permissions until a real permission registry and
  sandbox exist (fail-closed).
- Extracts to a staging directory and atomically swaps it into place.
- Does not execute scripts, verify signatures, resolve dependencies, or embed
  Luau/Lua. It is a host-side developer tool, **not yet the native OS GUI
  installer or a kernel-integrated package manager**.
- Package contents remain untrusted. Do not run scripts from packages unless
  they are handled by a properly sandboxed runtime.
