# 2026-11-26 — Process-owned VFS capability integration

## Repository state
- Repository: `FQ-studio/OpenXzuff-UI-1.0`
- Branch: `feat/fdt-ranges-root-translation`
- PR: #15, open; base `feature/win32-gaming-gravity-port`
- Latest checkpoint/documentation head at creation: `03cf17960ead367ca74ae760417d54b05fd10cd3`
- `main` is unchanged.

## Starting baseline verified
The preceding UART RX FIFO checkpoint is verified on code head
`79c004ff3af532091dac8842b197090b8306b196`:
- RV64 PR run #45720938345: SUCCESS; host suite 136/136; RV64 ecall adapter
  cross-compile; ELF64 RISC-V kernel; OpenSBI/QEMU four-byte UART RX burst
  marker `UART PLIC RX IRQ burst: PASS; source=a; count=4; bytes=5a,6b,7c,8d`.
- Kernel ELF and serial log artifact:
  https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/45720938345/artifacts/15070889449
- VisionFive 2 run #45720938335: SUCCESS; host CTest 16/16 and board-contract
  checks only, not physical-board boot.
- Win32 bridge run #45720938413: SUCCESS.
- OpenXzuff UI Core run #45721170521: SUCCESS. Its log shows root CTest 16/16,
  Xzuff Go/Coder Checker and .xexe smoke, Gravity RV64 NDK, Android ARM64 xgo
  package, Luau runtime execution and .lap pack/run, JavaScript 2/2, Lua 2/2,
  OpenXzuff3D 1/1, rendered mesh scene and artifact upload.

## New implementation
1. Added `kernel/src/process_io.rs`, joining the existing ProcessId,
   generation-safe HandleTable and generation-safe RAM VFS NodeId.
2. Added file create/open/read/write/close and explicit reduced-rights
   capability duplication between processes.
3. Encodes VFS node slot + generation in the kernel-owned handle object ID;
   stale inode generations are still checked by RamFs on every read/write.
4. If handle allocation fails after file creation, the new inode is removed
   before returning the handle-table error.
5. Added host tests for process ownership, read/write rights, explicit
   read-only duplication, stale-handle rejection, directory-open rejection,
   and rollback when the handle table is full.
6. Wired `process_io` into the RV64 kernel and added a boot-time smoke using
   two ProcessIds supplied by TaskManager. It verifies owner-only access,
   a reduced READ-only duplicate, write denial through that duplicate,
   stale-handle rejection after close, and zero remaining open handles.
7. Added a mandatory QEMU serial assertion:
   `Process-owned VFS handles: PASS; owner-isolation=1; duplicate-read=1; stale-close=1`.

## New code / workflow commits
- `c1a4081`: add ProcessIo capability/VFS layer.
- `d658e56` and `8ba550c`: host test integration edits; the intermediate
  duplicate module insertion was removed in `026910d`.
- `9bef9f2`: initial RV64 runtime smoke integration.
- `f7642a3`: correct serial marker escaping.
- `706bf1c`: require the ProcessIo smoke marker in QEMU workflow.
- `026910d`: remove duplicate host test module declaration.
- `03cf179`: update driver/HAL matrix with the already verified UART burst
  evidence (no driver behavior changed).

## Verification status for ProcessIo change

### Intermediate failure diagnosed and corrected
RV64 run #45770102321 failed in the host test compile before RV64 build or
QEMU. Its log showed Rust E0428: `process_io` and `process_io_tests` were
declared twice in `kernel/allocator-tests/src/lib.rs`. The duplicate
insertion was removed in commit `026910df80d47699ebb4db95d0e0c3f8dc073efc`.
This was a test-module declaration error, not a ProcessIo implementation or
kernel runtime failure.

### Latest verification in progress
For code head `03cf17960ead367ca74ae760417d54b05fd10cd3`:
- RV64 S-Mode Kernel run #45770332404: IN PROGRESS.
- OpenXzuff UI Core run #45770332533: IN PROGRESS.
- Win32 bridge run #45770332399: both jobs SUCCESS.
- VisionFive 2 readiness run #45770332437: SUCCESS; CTest 16/16 and
  board-contract checks only, not physical-board boot.

The RV64 QEMU serial log for the new ProcessIo marker is not yet available.
Therefore the new ProcessIo host suite, RV64 ELF and QEMU marker are NOT
claimed PASS. Earlier baseline results above do not prove this new change.

## Scope boundaries
- This is a kernel-internal process-owned file capability layer, not yet
  exposed through U-mode OPEN/READ/WRITE/CLOSE syscalls.
- `RamFs` remains bounded, volatile RAM storage (64 nodes, 256 bytes per file);
  no block-device or persistent filesystem backend is present.
- The current QEMU user task still uses the bootstrap user address space;
  this change does not implement one independent Sv39 root per process or a
  dynamic executable loader.
- The process identity passed into ProcessIo must be validated by the future
  syscall dispatcher; ProcessIo itself enforces handle ownership, not process
  table membership.
- No physical-board boot or long-duration stability is claimed.

## Next
1. Inspect exact-head Actions and require the new ProcessIo serial marker.
2. Fix any host/RV64/QEMU regression before adding more service code.
3. If green, connect OPEN/READ/WRITE/CLOSE ABI calls to ProcessIo with
   user-pointer validation and process identity lookup.
4. Then implement real per-process Sv39 roots and process teardown cleanup.
