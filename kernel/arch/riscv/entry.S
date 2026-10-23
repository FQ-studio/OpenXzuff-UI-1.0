/* RV64 entry for an SBI/OpenSBI-style direct payload. */
.section .text.entry
.globl _start
.type _start, @function
_start:
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    la sp, __stack_top
    andi sp, sp, -16
    call kernel_main
1:
    wfi
    j 1b
.size _start, .-_start
