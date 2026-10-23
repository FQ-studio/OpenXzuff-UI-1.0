OUTPUT_ARCH(riscv)
ENTRY(_start)
/* Common OpenSBI payload address for a controlled bring-up target only. */
SECTIONS
{
  . = 0x80200000;\n  __kernel_start = .;
  .text : ALIGN(4K) { KEEP(*(.text.entry)) *(.text .text.*) }
  .rodata : ALIGN(4K) { *(.rodata .rodata.*) }
  .data : ALIGN(4K) { *(.data .data.*) }
  .bss (NOLOAD) : ALIGN(4K) {
    __bss_start = .;
    *(.bss .bss.*) *(COMMON)
    __bss_end = .;
  }
  . = ALIGN(16);
  . += 64K;
  __stack_top = .;\n  __kernel_end = .;
  __global_pointer$ = . + 0x800;
  /DISCARD/ : { *(.comment) *(.eh_frame*) }
}
