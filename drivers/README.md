# Native drivers

Driver work starts only after kernel primitives and platform discovery exist. Planned first interfaces: serial console, timer, interrupt controller, framebuffer/GOP, keyboard, pointer/touch and block storage. Driver code must be guarded by verified board/firmware capabilities; no unverified MMIO addresses should be hard-coded.
