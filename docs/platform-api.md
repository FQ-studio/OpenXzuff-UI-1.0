# Platform adapter contract

The OpenXzuff UI core depends on a tiny host API. This file documents the boundary; it does **not** assert that Gravity Kernel already provides these functions.

## Drawing

The host object passed to `UI.new(platform)` and `Desktop.new(platform, options)` must implement:

```lua
platform.fill_rect(x, y, width, height, rgba)
platform.draw_text(text, x, y, rgba)
```

- `x`, `y`, `width`, and `height` are framebuffer pixel coordinates.
- `rgba` is a four-element array of integer channel values in the range 0–255.
- Rectangles with zero width or height should be harmless no-ops.
- Text coordinates identify the top-left of the text baseline box according to the host's documented font metrics.
- The host must clip drawing to the active framebuffer bounds.
- The first host can implement `draw_text` with a tiny built-in bitmap font; no external font library is required by the UI core.

## Input

The host forwards normalized events:

```lua
desktop:pointer_pressed(x, y, button)
desktop:pointer_released(x, y, button)
```

- Coordinates must use the same pixel coordinate space as drawing.
- `button == 1` represents primary pointer/touch activation.
- Touch down maps to press and touch up maps to release.
- The host is responsible for display rotation, scaling, multi-touch policy, and pointer capture.
- Keyboard support is not implemented in this foundation.

## VM lifecycle

The platform integration must define and test:

- how Lua VM memory is allocated and bounded;
- how the VM is created and destroyed;
- how scripts/modules are loaded;
- how runtime errors are surfaced without taking down the kernel;
- how the UI loop receives events and requests redraws;
- which standard libraries are enabled.

Do not expose unrestricted filesystem, process, or native-module access to untrusted Lua scripts by default.

## Not yet defined

No kernel syscall numbers, physical addresses, framebuffer pixel formats, interrupt identifiers, scheduler hooks, or input-device registers are specified here. Those values must come from a reviewed Gravity Kernel interface, not guesswork.
