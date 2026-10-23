# OpenXzuff UI native OS roadmap

## Phase 1 — host-tested Lua GUI
- Keep Lua UI tests passing in CI.
- Define the graphics/input adapter contract.

## Phase 2 — freestanding RV64 bring-up
- Build the minimal payload with a cross compiler.
- Add verified serial output and SBI probing, then memory-map and device-tree parsing.
- Test in an emulator before claiming a board boots.

## Phase 3 — display and input
- Add framebuffer discovery and drawing primitives.
- Add input events, allocator, timer and scheduler prerequisites.
- Embed Lua and run the desktop shell over the native platform API.

## Phase 4 — ARM64 UEFI PC
- Implement EFI PE/COFF entry/loader and UEFI GOP/memory-map support.
- Bring up exceptions, GIC/timer, and a verified device discovery path.

## Phase 5 — board ports and compatibility
- Port RV64 support separately to Unmatched and VisionFive 2.
- Define a narrow Win32 Basic API profile.
- Evaluate a minimal ART-based Android compatibility profile only after core OS services exist.

## Acceptance rule
A target is “bootable” only after the image is built reproducibly and the kernel reaches a visible, testable milestone in QEMU or real hardware. Documentation and stubs do not count as boot support.
