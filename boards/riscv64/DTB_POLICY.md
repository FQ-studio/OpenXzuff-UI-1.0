# Device Tree Blob (DTB) policy

A `.dts` file is Device Tree **Source**; a `.dtb` is the compiled binary. Do not rename a source file to `.dtb` or commit a hand-written binary blob and claim it is validated.

For the three initial RV64 boards, OpenXzuff UI should first parse the DTB passed by firmware. This avoids stale RAM sizes, reserved-memory ranges, clocks, and peripheral descriptions. Board-specific DTB builds can be added once the repository pins the exact upstream DTS/DTSI source revision and its license.

## Required validation before a board is called supported

- Compile source with `dtc` and fail on errors.
- Run structural checks and verify `/compatible`, `/chosen/stdout-path`, CPU/hart descriptions, memory ranges, reserved-memory, and UART `reg`/compatible.
- Boot using the board's documented firmware path.
- Record serial-console smoke-test output and tested board revision.

**Current status:** board documentation only; this repository does not yet contain verified board DTB binaries.