# RV64 Sv39 bootstrap mappings and W^X verification

## Scope

The current RV64 bootstrap builds two static Sv39 roots for one 1 GiB identity-mapped physical window:

- **Kernel root:** supervisor-only identity mappings.
- **Probe-user root:** the same kernel image and bootstrap window, with only the built-in U-mode probe text and its private user stack exposed to U-mode.

This is an early-boot protection layout, not a general virtual-memory manager or a process loader.

## Mapping layout

The kernel linker script page-aligns the start and end of `.text`, `.rodata`, `.data` and `.bss`. It also page-aligns the `.text.umode_user` probe subsection and reserves a page-aligned 4 KiB `UMODE_STACK`.

The kernel image's 2 MiB L1 region is split into 4 KiB leaf PTEs. The remaining part of the 1 GiB bootstrap window uses supervisor RW 2 MiB leaves.

| Region | Kernel root | Probe-user root |
|---|---|---|
| Kernel `.text` | S: R-X | S: R-X, except probe pages |
| `.text.umode_user` | S: R-X | U: R-X |
| `.rodata` | S: R-- | S: R-- |
| `.data` | S: RW- | S: RW- |
| `.bss` | S: RW-, including user and trap stacks | S: RW-, except the private user stack |
| `UMODE_STACK` | S: RW- | U: RW- |
| Per-task supervisor trap stacks | S: RW- | S: RW- |
| Other bootstrap-window pages | S: RW- | S: RW- |

`U` is clear for all kernel and supervisor-owned pages in the probe-user root. The U-mode probe code is not writable; the user stack is not executable.

## Runtime verification

`vm::verify_isolation` walks every 4 KiB page in each linker-defined section. It does not infer an entire section's permissions from a single sample address.

For `.text` and `.bss`, the verifier applies explicit expected-permission exceptions to the U-mode probe text and user stack respectively. It also checks the entire U-mode code and stack ranges, the kernel entry page, and each complete per-task supervisor trap-stack page in both roots.

The linker layout and range verifier require page-aligned section and user-region boundaries. The bootstrap currently rejects a user code or stack range that crosses its supported 2 MiB L1 region with `VmError::UserRegionTooLarge`; the kernel image is likewise constrained to one 2 MiB region.

The RV64 workflow requires the QEMU serial markers for the Sv39 roots, W^X section permissions and all 16 supervisor-only trap-stack slots. The latest full-range verifier was built and exercised by [RV64 S-Mode Kernel run #156](https://github.com/FQ-studio/OpenXzuff-UI-1.0/actions/runs/43168461040), using OpenSBI and QEMU `virt`.

## Current limitations

- The bootstrap identity maps one 1 GiB window; it does not allocate arbitrary virtual regions.
- The probe-user root is a single static test address space, not a per-process address-space manager.
- The runtime currently executes one U-mode task on one hart. The other reserved trap-stack slots are checked for mapping and layout but are not evidence of multi-U-task or SMP operation.
- No ASIDs, TLB shootdown, page-table reclamation, demand paging, guard-page policy, user-memory copy-in/copy-out or general process loader are established.
- QEMU validation does not establish physical-board boot or device-driver readiness.

## Regression policy

A change to page-table construction or linker section boundaries must keep the full-range permission walk and the serial assertions enabled. A successful host contract test alone is not evidence that the RV64 page tables enforce these permissions at runtime.