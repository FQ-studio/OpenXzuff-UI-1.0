# OpenXzuff Win32 Gaming Runtime — Gravity-native port plan

Status: architecture scaffold only; no executable compatibility runtime is implemented yet.

## Goal

Build a Windows application/game compatibility subsystem for OpenXzuff UI 1.0 Full on RV64. Reuse suitable upstream Wine/Proton components only after auditing their licenses and dependencies. Replace Linux host integration with Gravity Kernel and OpenXzuff services.

This is not a Linux distribution, Linux userspace, or a claim that upstream Proton can run unchanged on Gravity.

## Non-goals

- Do not embed or boot a Linux kernel.
- Do not depend on Linux syscalls, glibc, the Linux process model, epoll, ioctls, DRM/KMS, or Linux device nodes at runtime.
- Do not claim broad Windows game compatibility before the relevant APIs and instruction translation are implemented and tested.
- Do not add x86_64 or AArch64 as native OpenXzuff kernel targets. The OS remains RV64.

## Layering

1. Windows compatibility: PE/COFF loader, Win32/Win64 API subsets, DLL loader, registry/configuration subset.
2. Runtime translation: x86 and x86-64 instruction translation to RV64. This is a required subsystem, not supplied by Wine itself.
3. Graphics/audio/input adapters: OpenXzuff graphics, audio and input service backends. DXVK/VKD3D/FAudio are candidates for separately audited ports, not assumed working dependencies.
4. Win32 host adapter: process, thread, virtual memory, synchronization, files, clocks, networking and handles.
5. Gravity backend: implement the host adapter using documented Gravity Kernel syscalls and OpenXzuff system services only.

## Linux replacement map

| Upstream Linux-facing facility | OpenXzuff replacement |
|---|---|
| Linux process/thread creation | Gravity process and thread services |
| pthreads/futex | Gravity thread and synchronization ABI |
| mmap/mprotect | Gravity virtual-memory API |
| signals/exceptions | Gravity exception and trap-delivery contract |
| Linux file descriptors | OpenXzuff handle/VFS service |
| epoll/eventfd | Gravity waitable objects and event service |
| sockets | OpenXzuff networking service |
| DRM/KMS/Vulkan loader integration | OpenXzuff graphics/HAL interfaces |
| ALSA/PulseAudio integration | OpenXzuff audio service |
| evdev/input integration | OpenXzuff input service |
| Linux dynamic loader/libc | Dedicated RV64-native runtime support; no glibc assumption |

The table defines intended replacements. It does not assert that all replacement APIs currently exist.

## Repository layout proposal

```text
win32-gaming/
  README.md
  docs/
    linux-dependency-audit.md
    compatibility-matrix.md
  include/
    xzfg/host_abi.h
  src/
    host/
    pe/
    win32/
    translate/
    graphics/
    audio/
    input/
  tests/
    abi/
    pe/
    translation/
    integration/
```

The directory tree is a proposal; create source modules only after checking existing repository conventions and the actual Gravity ABI.

## Required implementation phases

### Phase 0 — dependency and ABI audit
- Inventory the current repository and Gravity syscall/service contracts.
- Select a pinned Wine/Proton source baseline.
- Record license obligations and every Linux-specific dependency.
- Reject any dependency that cannot be isolated behind the host adapter.

### Phase 1 — Gravity host ABI
- Define stable C-compatible types and versioned function table.
- Define handles, status codes, memory protection flags, wait semantics and thread lifecycle.
- Implement host-side mock backend and contract tests.
- Do not invent syscall numbers; bind only to existing, documented Gravity ABI entries.

### Phase 2 — PE loader
- Parse PE32 and PE32+ headers safely.
- Validate sections, imports, relocations and bounds.
- Add malformed-input and fuzz-style parser tests.
- Load a minimal RV64-native test fixture first; x86/x64 execution remains blocked on translation.

### Phase 3 — Win32 API subset
- Implement a deliberately small API surface and compatibility matrix.
- Add deterministic tests for handles, files, threads, synchronization and virtual memory.
- Add explicit unsupported API errors.

### Phase 4 — RV64 translation
- Select or develop an x86/x64-to-RV64 dynamic translation strategy.
- Validate instruction semantics, exceptions, atomics, memory ordering and calling conventions.
- No game compatibility claim until executable test fixtures pass on RV64.

### Phase 5 — graphics/audio/input
- Integrate OpenXzuff service backends.
- Port candidate upstream components individually after dependency and license review.
- Validate a simple window, input, audio and rendering test before game tests.

### Phase 6 — Proton-derived packaging
- Package only the audited compatibility components required by the selected profile.
- Produce reproducible RV64 artifacts and report compressed and installed sizes separately.
- Do not promise the earlier 100–300 MB planning budget until a real build measures it.

## CI acceptance gates

1. Host ABI contract tests pass.
2. PE parser tests pass, including malformed files.
3. Linux dependency audit reports no unresolved runtime dependency in the Gravity backend.
4. RV64 cross-build succeeds.
5. QEMU tests prove host adapter calls reach Gravity services.
6. RV64 translation fixtures pass before any Windows x86/x64 app execution claim.
7. Graphics/audio/input smoke tests pass against OpenXzuff services.
8. Build artifacts include source revision, toolchain versions, license manifest and measured sizes.

## Current verified state

This commit adds design documentation only. No Win32 runtime, Wine port, Proton port, PE loader, instruction translator, or game execution has been built or tested by this change.
