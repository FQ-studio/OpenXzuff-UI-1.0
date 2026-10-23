# OpenXzuff Script App Packages

Status: **format proposal v1**. This document specifies package layouts; it does
not claim that the OXLE runtime, installer, signature verifier, or app execution
path is implemented yet.

## Extensions

- `.lap` — **Luau App Package**. Entry script defaults to `main.luau` and is
  executed by OpenXzuff Luau Engine (OXLE).
- `.luap` — **Lua App Package**. Entry script defaults to `main.lua` and is
  executed by a compatible Lua runtime selected by the manifest.

These are separate formats. A package must not be silently treated as the other
language. The runtime must reject a language/runtime mismatch before executing
any script.

## Container

Both formats use a ZIP container with the extension above. A package contains:

```text
MyApp.lap (or MyApp.luap)
├── manifest.json
├── main.luau (or main.lua)
└── assets/
    └── ...
```

The first implementation should support stored/deflated ZIP entries and UTF-8
JSON. It must reject absolute paths, `..` path traversal, duplicate manifest
keys, duplicate normalized entry paths, encrypted ZIP entries, unsupported
compression, oversized entries, and archives exceeding configured size/count
limits. Extracted files must remain inside the app's private package directory.

## Manifest v1

Example `.lap` manifest:

```json
{
  "format": "LAP",
  "formatVersion": 1,
  "id": "org.example.hello",
  "name": "Hello OpenXzuff",
  "version": "1.0.0",
  "language": "Luau",
  "runtime": "OXLE",
  "runtimeApi": 1,
  "entry": "main.luau",
  "permissions": [],
  "assets": ["assets/"]
}
```

Example `.luap` manifest:

```json
{
  "format": "LUAP",
  "formatVersion": 1,
  "id": "org.example.hello-lua",
  "name": "Hello Lua",
  "version": "1.0.0",
  "language": "Lua",
  "runtime": "Lua 5.4",
  "runtimeApi": 1,
  "entry": "main.lua",
  "permissions": [],
  "assets": ["assets/"]
}
```

### Required validation

1. The filename extension, manifest `format`, `language`, runtime and entry
   extension must agree.
2. `formatVersion` must be supported; reject unknown mandatory fields/versions
   rather than guessing.
3. `id` must be a stable reverse-DNS-style identifier; `version` must be a
   valid semantic version.
4. `entry` must be a relative path to an existing script inside the archive.
5. `permissions` must be a list of recognized permission identifiers. Unknown
   permissions are denied by default.
6. Validate the complete archive before running any script. Never execute an
   entry while still extracting the package.

## Runtime and compatibility boundary

- OXLE must embed or link an actual Luau VM before `.lap` files can run.
- `.luap` needs a separately selected Lua runtime; Lua and Luau bytecode are
  not interchangeable.
- Roblox-style APIs such as `game:GetService()`, `Instance.new()`,
  `workspace`, and `script.Parent` are host APIs, not features provided by
  the Luau language itself. OpenXzuff must implement its own compatible APIs;
  Roblox services, networking, and game servers are not included automatically.
- Packages are untrusted input. Run apps with a restricted capability model;
  do not expose raw kernel memory, arbitrary filesystem access, or syscalls
  directly to scripts.
- Package signing, trusted publisher identities, installer/update behavior,
  sandbox enforcement, and executable runtime integration are future work.
  Unsigned packages should be clearly marked as untrusted until verification is
  implemented.

## Relation to GAPP

`GAPP` is a separate native-image metadata format. `.lap` and `.luap` are
script application containers and must not be parsed as GAPP binaries. A future
installer may wrap or index these formats, but that does not make them native
machine-code applications.
