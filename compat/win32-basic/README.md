# Win32 Basic compatibility layer

This is a planned subset API, not a full Windows implementation. Begin with a documented small API surface (basic file, memory, timing, event and window abstractions) mapped to OpenXzuff services. Windows binaries also depend on PE loading, calling conventions, CRT behavior, graphics APIs and many OS services; unsupported APIs must fail clearly rather than be silently misrepresented.
