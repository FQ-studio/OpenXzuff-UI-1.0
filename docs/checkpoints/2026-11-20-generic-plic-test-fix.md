# 2026-11-20 — Generic PLIC test fixture fix

## Diagnosis
GitHub Actions run #44687549591 failed at:
`plic_fdt_discovers_generic_riscv_compatible`
with `FdtError::Structure`. The failing fixture in
`kernel/allocator-tests/src/lib.rs` used the Rust byte string
`b"riscv,plic0\\\\0"`, which encodes a literal backslash followed by
zero rather than a NUL terminator. The compatible-list parser correctly
rejected that malformed string list.

## Fix
Changed the fixture to `b"riscv,plic0\\0"` (actual NUL terminator).
Commit: `36bf2e0d3fcd5fe5a6928f4c5b171e8f90afa774`.

The commit diff was inspected and confirms this is a one-line test-fixture
correction. No production parser behavior was changed.

## Verification status
The prior failing run recorded 96 passed / 1 failed. The correction has
been pushed to `feat/fdt-ranges-root-translation`. A workflow run for the
new commit was not yet returned by the available GitHub Actions lookup at
checkpoint time. Therefore 97/97, RV64 cross-build, kernel ELF and QEMU
for this corrected head are NOT claimed PASS.

## Next
Wait for / inspect Actions on commit `36bf2e0`. If green, confirm RV64
ELF and QEMU serial assertions, then move on from FDT/PLIC to actual
Sv39 device mapping and PLIC interrupt delivery. Avoid repeating FDT
discovery work unless a new regression is evidenced.
