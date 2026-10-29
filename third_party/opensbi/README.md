# OpenSBI firmware for OpenXzuff UI

OpenXzuff UI builds upstream OpenSBI firmware as a **separate M-mode firmware image**. It is not linked into Gravity Kernel. The source is fetched from the official upstream repository at the pinned release tag `v1.9` by the build script; this directory intentionally does not duplicate upstream source files.

- Upstream: https://github.com/riscv-software-src/opensbi
- Version: `v1.9`
- License: BSD-2-Clause; preserve upstream `COPYING.BSD` and notices when redistributing.
- Initial build target: `PLATFORM=generic`, output `fw_dynamic.bin`.
- Build outputs are generated artifacts, not committed firmware binaries.

**Important platform limitation:** generic OpenSBI is FDT-driven, but that alone does not prove firmware compatibility with all three target boards. HiFive Unmatched (FU740), VisionFive 2 (JH7110), and Lichee Pi 4A (TH1520) need separate boot-chain and physical-device validation. VisionFive 2 may require platform-specific load-address/payload handling; do not flash a generic image unless the board's documented firmware chain requires it.

The current build produces a generic firmware artifact for inspection/testing, not a claim that all three boards can boot it directly.