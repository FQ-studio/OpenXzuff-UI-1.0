    .section .text.trap, "ax"
    .align  4
    .global __gravity_s_trap_vector
    .type   __gravity_s_trap_vector, @function

/* S-mode-only direct trap vector. The current image has no U-mode execution;
 * a future U-mode path must switch to a supervisor-owned stack before using
 * this entry. TrapFrame offsets match the 288-byte C++/Rust layout contract.
 */
__gravity_s_trap_vector:
    addi    sp, sp, -288

    /* Save every integer register before using a temporary register. */
    sd      zero, 0(sp)
    sd      ra,   8(sp)
    sd      t0,  40(sp)
    addi    t0, sp, 288
    sd      t0,  16(sp)       /* interrupted sp */
    sd      gp,  24(sp)
    sd      tp,  32(sp)
    sd      t1,  48(sp)
    sd      t2,  56(sp)
    sd      s0,  64(sp)
    sd      s1,  72(sp)
    sd      a0,  80(sp)
    sd      a1,  88(sp)
    sd      a2,  96(sp)
    sd      a3, 104(sp)
    sd      a4, 112(sp)
    sd      a5, 120(sp)
    sd      a6, 128(sp)
    sd      a7, 136(sp)
    sd      s2, 144(sp)
    sd      s3, 152(sp)
    sd      s4, 160(sp)
    sd      s5, 168(sp)
    sd      s6, 176(sp)
    sd      s7, 184(sp)
    sd      s8, 192(sp)
    sd      s9, 200(sp)
    sd      s10, 208(sp)
    sd      s11, 216(sp)
    sd      t3, 224(sp)
    sd      t4, 232(sp)
    sd      t5, 240(sp)
    sd      t6, 248(sp)

    csrr    t0, sepc
    sd      t0, 256(sp)
    csrr    t0, sstatus
    sd      t0, 264(sp)
    csrr    t0, scause
    sd      t0, 272(sp)
    csrr    t0, stval
    sd      t0, 280(sp)

    mv      a0, sp
    call    gravity_s_trap_handler

    /* Restore the return PC/status selected by the handler. */
    ld      t0, 256(sp)
    csrw    sepc, t0
    ld      t0, 264(sp)
    csrw    sstatus, t0

    ld      ra,   8(sp)
    ld      gp,  24(sp)
    ld      tp,  32(sp)
    ld      t1,  48(sp)
    ld      t2,  56(sp)
    ld      s0,  64(sp)
    ld      s1,  72(sp)
    ld      a0,  80(sp)
    ld      a1,  88(sp)
    ld      a2,  96(sp)
    ld      a3, 104(sp)
    ld      a4, 112(sp)
    ld      a5, 120(sp)
    ld      a6, 128(sp)
    ld      a7, 136(sp)
    ld      s2, 144(sp)
    ld      s3, 152(sp)
    ld      s4, 160(sp)
    ld      s5, 168(sp)
    ld      s6, 176(sp)
    ld      s7, 184(sp)
    ld      s8, 192(sp)
    ld      s9, 200(sp)
    ld      s10, 208(sp)
    ld      s11, 216(sp)
    ld      t3, 224(sp)
    ld      t4, 232(sp)
    ld      t5, 240(sp)
    ld      t6, 248(sp)
    ld      t0,  40(sp)
    ld      sp,  16(sp)
    sret

    .size __gravity_s_trap_vector, . - __gravity_s_trap_vector

/* ABI-conforming test helper. It seeds x1 and x5-x31 with distinct sentinels,
 * executes one delegated breakpoint, checks every register after sret, then
 * restores the callee-saved registers and returns 0 (pass) or 1 (failure).
 */
    .align  4
    .global __gravity_s_trap_roundtrip_test
    .type   __gravity_s_trap_roundtrip_test, @function
__gravity_s_trap_roundtrip_test:
    addi    sp, sp, -128
    sd      s0,   0(sp)
    sd      s1,   8(sp)
    sd      s2,  16(sp)
    sd      s3,  24(sp)
    sd      s4,  32(sp)
    sd      s5,  40(sp)
    sd      s6,  48(sp)
    sd      s7,  56(sp)
    sd      s8,  64(sp)
    sd      s9,  72(sp)
    sd      s10, 80(sp)
    sd      s11, 88(sp)
    sd      ra,  96(sp)
    sd      gp, 104(sp)
    sd      tp, 112(sp)
    addi    t0, sp, 128
    sd      t0, 120(sp)

    li      ra,  0x101
    li      t0,  0x105
    li      t1,  0x106
    li      t2,  0x107
    li      s0,  0x108
    li      s1,  0x109
    li      a0,  0x10a
    li      a1,  0x10b
    li      a2,  0x10c
    li      a3,  0x10d
    li      a4,  0x10e
    li      a5,  0x10f
    li      a6,  0x110
    li      a7,  0x111
    li      s2,  0x112
    li      s3,  0x113
    li      s4,  0x114
    li      s5,  0x115
    li      s6,  0x116
    li      s7,  0x117
    li      s8,  0x118
    li      s9,  0x119
    li      s10, 0x11a
    li      s11, 0x11b
    li      t3,  0x11c
    li      t4,  0x11d
    li      t5,  0x11e
    li      t6,  0x11f
    ebreak

    /* t6 is checked first; it then becomes the comparison scratch register. */
    li      t5, 0x11f
    bne     t6, t5, .Lgravity_trap_fail
    li      t6, 0x11e
    bne     t5, t6, .Lgravity_trap_fail
    li      t6, 0x11d
    bne     t4, t6, .Lgravity_trap_fail
    li      t6, 0x11c
    bne     t3, t6, .Lgravity_trap_fail
    li      t6, 0x11b
    bne     s11, t6, .Lgravity_trap_fail
    li      t6, 0x11a
    bne     s10, t6, .Lgravity_trap_fail
    li      t6, 0x119
    bne     s9, t6, .Lgravity_trap_fail
    li      t6, 0x118
    bne     s8, t6, .Lgravity_trap_fail
    li      t6, 0x117
    bne     s7, t6, .Lgravity_trap_fail
    li      t6, 0x116
    bne     s6, t6, .Lgravity_trap_fail
    li      t6, 0x115
    bne     s5, t6, .Lgravity_trap_fail
    li      t6, 0x114
    bne     s4, t6, .Lgravity_trap_fail
    li      t6, 0x113
    bne     s3, t6, .Lgravity_trap_fail
    li      t6, 0x112
    bne     s2, t6, .Lgravity_trap_fail
    li      t6, 0x111
    bne     a7, t6, .Lgravity_trap_fail
    li      t6, 0x110
    bne     a6, t6, .Lgravity_trap_fail
    li      t6, 0x10f
    bne     a5, t6, .Lgravity_trap_fail
    li      t6, 0x10e
    bne     a4, t6, .Lgravity_trap_fail
    li      t6, 0x10d
    bne     a3, t6, .Lgravity_trap_fail
    li      t6, 0x10c
    bne     a2, t6, .Lgravity_trap_fail
    li      t6, 0x10b
    bne     a1, t6, .Lgravity_trap_fail
    li      t6, 0x10a
    bne     a0, t6, .Lgravity_trap_fail
    li      t6, 0x109
    bne     s1, t6, .Lgravity_trap_fail
    li      t6, 0x108
    bne     s0, t6, .Lgravity_trap_fail
    li      t6, 0x107
    bne     t2, t6, .Lgravity_trap_fail
    li      t6, 0x106
    bne     t1, t6, .Lgravity_trap_fail
    li      t6, 0x105
    bne     t0, t6, .Lgravity_trap_fail
    li      t6, 0x101
    bne     ra, t6, .Lgravity_trap_fail

    ld      t6, 104(sp)
    bne     gp, t6, .Lgravity_trap_fail
    ld      t6, 112(sp)
    bne     tp, t6, .Lgravity_trap_fail
    addi    t6, sp, 128
    ld      t5, 120(sp)
    bne     t6, t5, .Lgravity_trap_fail

    li      a0, 0
    j       .Lgravity_trap_restore

.Lgravity_trap_fail:
    li      a0, 1

.Lgravity_trap_restore:
    ld      s0,   0(sp)
    ld      s1,   8(sp)
    ld      s2,  16(sp)
    ld      s3,  24(sp)
    ld      s4,  32(sp)
    ld      s5,  40(sp)
    ld      s6,  48(sp)
    ld      s7,  56(sp)
    ld      s8,  64(sp)
    ld      s9,  72(sp)
    ld      s10, 80(sp)
    ld      s11, 88(sp)
    ld      ra,  96(sp)
    ld      gp, 104(sp)
    ld      tp, 112(sp)
    addi    sp, sp, 128
    ret

    .size __gravity_s_trap_roundtrip_test, . - __gravity_s_trap_roundtrip_test
