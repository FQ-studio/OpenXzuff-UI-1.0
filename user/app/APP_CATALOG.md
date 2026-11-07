# OpenXzuff UI Core — Basic Application Catalog

Status: planning scaffold. This catalog does not claim that these applications are implemented or runnable yet.

## Core 1.0 starter applications

These are the first native user-space applications to build once the application ABI and runtime are available:

1. **System Information** — display OS version, RV64 architecture, kernel build identity, memory/process information exposed by approved system services.
2. **Settings** — read and update only settings exposed through system-service contracts; no direct kernel-memory access.
3. **File Manager** — browse files and directories through a filesystem service API; do not assume a filesystem syscall exists until the ABI is defined.
4. **GSH Terminal** — launch or connect to the native Gravity Shell through its published service interface; do not embed Bash or Zsh.
5. **Application Manager** — list, inspect, install and remove packages only after a package format, signature/trust policy and installer contract are specified.

## Planned ecosystem applications

- **XzuffTube** — video discovery and playback client. Requires a defined network service, media/container decoding path, audio/video output interfaces and application permissions.
- **XzuffTube Music** — music library and playback client. Requires the audio service/API, supported codecs and media storage access.

These two media applications are ecosystem targets, not Core 1.0 blockers. Do not add proprietary service credentials, API keys or secrets to this repository.

## Engineering constraints

- Applications are user-space programs; never call kernel internals directly.
- Use the existing lightweight manifest contract only for metadata validation. It is not a package installer, executable loader, .lap parser or .luap parser.
- Keep .lap (Luau) and .luap (Lua) distinct. Do not invent a binary package format before its specification is agreed.
- Do not claim GUI functionality: the GUI/compositor and window-manager architecture is not yet defined.
- Add build targets and tests only when the app entry-point contract and required service APIs are available.
- Each application must have a host-side validation test and, later, an RV64/QEMU integration test before being marked runnable.

## Suggested build order

1. Finalize app manifest and app entry-point/launch contract.
2. Define service APIs and permissions required by each starter app.
3. Implement System Information and GSH Terminal integration first.
4. Implement Settings and File Manager after their service contracts exist.
5. Define package format, signing/trust and installer before Application Manager.
6. Develop XzuffTube and XzuffTube Music after networking and media services exist.

## Status vocabulary

- **Planned** — catalog entry only.
- **Implemented** — source code exists.
- **Host-tested** — host build/test passed.
- **RV64-tested** — target/QEMU test passed.
- **Runnable** — launch path and required OS services work end-to-end.