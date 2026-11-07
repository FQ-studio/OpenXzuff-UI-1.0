# OpenXzuff JavaScript Runtime

Status: **planned / engine candidate selected for proof-of-concept only**. No JavaScript runtime is implemented or RV64-tested in this directory yet.

## Goal

Provide an embeddable JavaScript runtime for native OpenXzuff UI user-space applications. The runtime is not a browser: DOM, `window`, `document`, layout, and browser rendering APIs are out of scope until a separate browser project requires them.

The runtime must eventually run on RV64 through the real OpenXzuff user-space ABI and system services. A Linux/RISC-V executable is useful evidence of ISA portability, but it is not proof that an engine can run directly on the current bare-metal Gravity Kernel.

## Engine evaluation

### Preferred proof-of-concept: QuickJS-NG

QuickJS-NG is the preferred engine to evaluate first, not yet an unconditional production dependency.

Reasons:
- It is an embeddable JavaScript engine with a C-facing integration model.
- Its project targets modern ECMAScript features and active cross-platform development.
- Its v0.18.0 release publishes a `qjs-linux-riscv64` binary. This demonstrates a Linux RISC-V64 build exists; it does **not** demonstrate bare-metal support or compatibility with the OpenXzuff ABI.
- Its repository uses the MIT license. Preserve the upstream copyright and license notices if vendored or redistributed.

Before adoption, build the source from a pinned release for the host and RV64 targets, inspect required libc/platform functions, run its upstream tests, and measure binary size, startup time, memory use, and execution behavior. Do not rely on a prebuilt Linux binary as the OpenXzuff runtime.

### Alternative considered: Duktape

Duktape is compact, portable, embeddable, and MIT-licensed, but its primary compatibility target is ECMAScript 5.1 with only partial ES2015/ES2016 support. That makes it a weaker fit for modern JavaScript packages such as YouTube.js. Keep it as a low-footprint scripting alternative only if the application language requirements are deliberately reduced.

### Selection gate

QuickJS-NG becomes the selected engine only after:
1. Host build and upstream tests pass from a pinned source release.
2. A small OpenXzuff host embedding test can evaluate scripts, call native bindings, report exceptions, and enforce memory limits.
3. An RV64 cross-build and QEMU execution test pass using the actual target ABI/runtime environment.
4. License and upstream security/update process are recorded.

## Runtime layers

1. **ECMAScript engine** — parse and execute JavaScript; manage contexts, values, exceptions, garbage collection, and bytecode only as supported by the pinned engine.
2. **OpenXzuff JS Host** — own engine lifecycle, app contexts, native binding registration, logging, clocks, timers, and module loading.
3. **System API adapters** — expose only approved user-space services. Networking must use the OpenXzuff network service; storage must use the filesystem/storage service; do not add direct kernel access.
4. **Application SDK** — stable, versioned JavaScript APIs for native OpenXzuff applications.
5. **Optional browser layer** — DOM, HTML, CSS, browser events, rendering, and browser security model are a separate future project, not part of the initial runtime.

## YouTube.js compatibility requirements

YouTube.js is a JavaScript/TypeScript client built on YouTube's private InnerTube API. It is not automatically compatible merely because an engine can execute basic JavaScript.

Its current getting-started documentation identifies these runtime requirements:
- `fetch`;
- `EventTarget` and `CustomEvent`;
- a standards-compliant `Response`; `ReadableStream` is required for its `VideoInfo#download` path;
- a custom JavaScript interpreter/evaluation hook for features that decipher obfuscated player code and streaming URLs.

Therefore, compatibility work must be staged:
1. Run a pinned YouTube.js version in a supported reference runtime and record the exact features used by the intended XzuffTube flows.
2. Implement/test the required web-standard APIs over OpenXzuff system services.
3. Provide the required safe evaluation hook only within the application's isolated JS context.
4. Test session creation, search, metadata, player information, and actual media playback separately.
5. Treat playback as incomplete until the OpenXzuff media path can demux/decode supported streams and deliver audio/video.

Do not claim YouTube.js compatibility from a successful `import` alone. Do not implement CAPTCHA, DRM, login, or access-control circumvention. Respect YouTube restrictions and applicable terms.

## Isolation and safety requirements

- Create a separate runtime context per application (or another explicitly reviewed isolation boundary).
- Set per-app memory, stack, execution-time, and resource limits.
- Add an interrupt/termination path for runaway scripts.
- Expose capability-checked service handles, never raw kernel pointers or unrestricted system calls.
- Keep module resolution inside the app's signed package scope; reject path traversal and untrusted native modules.
- Treat downloaded scripts and package updates as untrusted until integrity/signature checks pass.
- Version the host API and preserve backward-compatibility rules.

## Implementation and test sequence

| Stage | Deliverable | Required evidence |
|---|---|---|
| 0 | Pin and evaluate QuickJS-NG | License review, build requirements, upstream test results, size/memory measurements |
| 1 | Host-side embedded JS runner | Host build; evaluate arithmetic, functions, modules, exceptions, native calls, memory-limit and timeout tests |
| 2 | OpenXzuff JS Host API | Unit tests for logging, clock, timers, module loader, permission checks and failure paths |
| 3 | RV64 target port | Cross-build and QEMU execution using the actual OpenXzuff ABI; record toolchain, target triple and logs |
| 4 | System API bindings | Tests for network, storage and IPC adapters after those services actually exist |
| 5 | YouTube.js compatibility prototype | Separate pass/fail evidence for import, fetch, session, search, metadata, player info and playback |

Do not mark any stage implemented, tested, RV64-tested, or runnable until its evidence exists. If the real user-space ABI or required services are not ready, keep the runtime host prototype isolated and report the integration blocker rather than inventing syscall bindings.

## Current repository dependencies and blockers

- `user/app/runtime/README.md` currently reserves that directory for Lua and Luau adapters and explicitly says those adapters are scaffold-only. Keep Lua `.luap` and Luau `.lap` separate from JavaScript.
- `user/app/README.md` says the application SDK/syscall/IPC wrappers are future work; the manifest is metadata only.
- `sys/README.md` describes a host-testable registry that does not yet launch processes or perform real IPC.
- The Gravity Kernel's current RV64 workflow demonstrates an early S-mode boot, FDT parsing, page allocation, and heap smoke checks. It does not yet demonstrate a complete user-space process ABI, scheduler-backed application execution, networking, storage, or browser services.

## Upstream references

- QuickJS-NG documentation: https://quickjs-ng.github.io/quickjs/
- QuickJS-NG releases (including the Linux RISC-V64 asset): https://github.com/quickjs-ng/quickjs/releases
- QuickJS-NG MIT license: https://github.com/quickjs-ng/quickjs/blob/master/LICENSE
- Duktape documentation: https://duktape.org/guide
- YouTube.js getting started and runtime prerequisites: https://ytjs.dev/guide/getting-started
- YouTube.js FAQ: https://ytjs.dev/guide/faq
