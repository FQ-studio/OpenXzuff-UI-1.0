/* AArch64 entry stub. Firmware/UEFI handoff and EL setup are platform-specific. */
.section .text.entry, "ax"
.global _start
.type _start, %function
_start:
    ldr x0, =__stack_top
    mov sp, x0
    bl kernel_main
1:
    wfi
    b 1b
.size _start, .-_start
