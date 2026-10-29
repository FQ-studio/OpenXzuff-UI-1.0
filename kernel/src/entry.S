    .section .text.entry
    .align  4
    .global _start
    .type   _start, @function
_start:
    /* OpenSBI enters S-mode with hart ID in a0 and FDT pointer in a1. */

    /* Clear BSS before selecting the stack stored inside that BSS. */
    la      t0, __bss_start
    la      t1, __bss_end
1:
    bgeu    t0, t1, 2f
    sd      zero, 0(t0)
    addi    t0, t0, 8
    j       1b
2:
    la      sp, __stack_top
    andi    sp, sp, -16
    call    rust_main
3:
    wfi
    j       3b
    .size _start, . - _start

    .section .bss.stack, "aw", @nobits
    .align  4
    .global __stack_bottom
__stack_bottom:
    .space  16384
    .global __stack_top
__stack_top:
