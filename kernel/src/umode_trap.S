    .section .text.trap.umode, "ax"
    .align 4
    .global __gravity_umode_trap_vector
    .type __gravity_umode_trap_vector, @function

/*
 * sscratch convention:
 *   U-mode: sscratch = supervisor-owned trap-stack top.
 *   S-mode: sscratch = 0.
 * The initial swap is safe in both cases. S-origin traps swap back and tail
 * into the existing S-mode vector; U-origin traps allocate only on S stack.
 */
__gravity_umode_trap_vector:
    csrrw   sp, sscratch, sp
    beqz    sp, .Lumode_from_s

    addi    sp, sp, -288
    sd      zero,   0(sp)
    sd      ra,     8(sp)
    sd      gp,    24(sp)
    sd      tp,    32(sp)
    sd      t0,    40(sp)
    sd      t1,    48(sp)
    sd      t2,    56(sp)
    sd      s0,    64(sp)
    sd      s1,    72(sp)
    sd      a0,    80(sp)
    sd      a1,    88(sp)
    sd      a2,    96(sp)
    sd      a3,   104(sp)
    sd      a4,   112(sp)
    sd      a5,   120(sp)
    sd      a6,   128(sp)
    sd      a7,   136(sp)
    sd      s2,   144(sp)
    sd      s3,   152(sp)
    sd      s4,   160(sp)
    sd      s5,   168(sp)
    sd      s6,   176(sp)
    sd      s7,   184(sp)
    sd      s8,   192(sp)
    sd      s9,   200(sp)
    sd      s10,  208(sp)
    sd      s11,  216(sp)
    sd      t3,   224(sp)
    sd      t4,   232(sp)
    sd      t5,   240(sp)
    sd      t6,   248(sp)
    csrr    t0, sscratch
    sd      t0,    16(sp)       /* interrupted U-mode sp */
    csrw    sscratch, zero
    csrr    t0, sepc
    sd      t0,   256(sp)
    csrr    t0, sstatus
    sd      t0,   264(sp)
    csrr    t0, scause
    sd      t0,   272(sp)
    csrr    t0, stval
    sd      t0,   280(sp)

    mv      a0, sp
    call    gravity_u_trap_handler

    /* Re-arm sscratch with this supervisor stack's top before returning U. */
    addi    t0, a0, 288
    csrw    sscratch, t0
    mv      sp, a0
    ld      t0, 256(sp)
    csrw    sepc, t0
    ld      t0, 264(sp)
    csrw    sstatus, t0
    ld      ra,     8(sp)
    ld      gp,    24(sp)
    ld      tp,    32(sp)
    ld      t1,    48(sp)
    ld      t2,    56(sp)
    ld      s0,    64(sp)
    ld      s1,    72(sp)
    ld      a0,    80(sp)
    ld      a1,    88(sp)
    ld      a2,    96(sp)
    ld      a3,   104(sp)
    ld      a4,   112(sp)
    ld      a5,   120(sp)
    ld      a6,   128(sp)
    ld      a7,   136(sp)
    ld      s2,   144(sp)
    ld      s3,   152(sp)
    ld      s4,   160(sp)
    ld      s5,   168(sp)
    ld      s6,   176(sp)
    ld      s7,   184(sp)
    ld      s8,   192(sp)
    ld      s9,   200(sp)
    ld      s10,  208(sp)
    ld      s11,  216(sp)
    ld      t3,   224(sp)
    ld      t4,   232(sp)
    ld      t5,   240(sp)
    ld      t6,   248(sp)
    ld      t0,    40(sp)
    ld      sp,    16(sp)
    sret

.Lumode_from_s:
    /* Restore original S stack and sscratch=0, then use established S path. */
    csrrw   sp, sscratch, sp
    j       __gravity_s_trap_vector
    .size __gravity_umode_trap_vector, . - __gravity_umode_trap_vector

    .section .text.umode_user, "ax"
    .align 4
    .global __gravity_u_mode_ecall_probe
    .type __gravity_u_mode_ecall_probe, @function
__gravity_u_mode_ecall_probe:
    /* U-mode must not be able to read the supervisor-only kernel image. */
    la      t0, __kernel_start
    ld      t1, 0(t0)
    li      a7, 1
    li      a0, 0x47524156
    li      a1, 0x11
    li      a2, 0x22
    li      a3, 0x33
    li      a4, 0x44
    li      a5, 0x55
    ecall
    ebreak
    j       .
    .size __gravity_u_mode_ecall_probe, . - __gravity_u_mode_ecall_probe
