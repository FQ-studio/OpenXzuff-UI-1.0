# OpenXzuff UI graphics boundary

The GUI layer is planned as a user-space compositor/window manager. This initial host-testable framebuffer is a software pixel buffer, not a hardware display driver. It is independent of OpenXzuff3D; the renderer can later target a framebuffer adapter when the kernel display/IPC contract is known.
