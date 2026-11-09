    .section .text.trap, "ax"
    .align  4
    .global __gravity_s_trap_vector
    .type   __gravity_s_trap_vector, @function

/* Save the interrupted RV64 register set and return through the frame selected
 * by Rust. The handler returns a TrapFrame pointer in a0, allowing a real task
 * context switch by changing the restore stack rather than merely changing PC.
 */
__gravity_s_trap_vector:
    addi    sp, sp, -288
    sd      zero, 0(sp)
    sd      ra,   8(sp)
    sd      t0,  40(sp)
    addi    t0, sp, 288
    sd      t0,  16(sp)
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
    mv      sp, a0
    j       .Lgravity_s_trap_restore

    .global __gravity_s_trap_resume
    .type   __gravity_s_trap_resume, @function
/* Enter the first task from Rust using the same complete frame restore path. */
__gravity_s_trap_resume:
    mv      sp, a0

.Lgravity_s_trap_restore:
    ld      t0, 264(sp)
    andi    t1, t0, 256
    bnez    t1, .Lgravity_restore_s_mode
    addi    t1, sp, 288
    csrw    sscratch, t1
    j       .Lgravity_restore_status
.Lgravity_restore_s_mode:
    csrw    sscratch, zero
.Lgravity_restore_status:
    csrw    sstatus, t0
    ld      t0, 256(sp)
    csrw    sepc, t0
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
    .size __gravity_s_trap_resume, . - __gravity_s_trap_resume

/* Keep the existing full-GPR breakpoint round-trip probe. */
    .align  4
    .global __gravity_s_trap_roundtrip_test
    .type   __gravity_s_trap_roundtrip_test, @function
__gravity_s_trap_roundtrip_test:
    addi    sp, sp, -512
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
    addi    t0, sp, 512
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
    sd      ra, 128(sp)
    sd      gp, 136(sp)
    sd      tp, 144(sp)
    sd      t0, 160(sp)
    sd      t1, 168(sp)
    sd      t2, 176(sp)
    sd      s0, 184(sp)
    sd      s1, 192(sp)
    sd      a0, 200(sp)
    sd      a1, 208(sp)
    sd      a2, 216(sp)
    sd      a3, 224(sp)
    sd      a4, 232(sp)
    sd      a5, 240(sp)
    sd      a6, 248(sp)
    sd      a7, 256(sp)
    sd      s2, 264(sp)
    sd      s3, 272(sp)
    sd      s4, 280(sp)
    sd      s5, 288(sp)
    sd      s6, 296(sp)
    sd      s7, 304(sp)
    sd      s8, 312(sp)
    sd      s9, 320(sp)
    sd      s10, 328(sp)
    sd      s11, 336(sp)
    sd      t3, 344(sp)
    sd      t4, 352(sp)
    sd      t5, 360(sp)
    sd      t6, 368(sp)
    ld      t6, 128(sp)
    li      t5, 0x101
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 136(sp)
    ld      t5, 104(sp)
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 144(sp)
    ld      t5, 112(sp)
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 160(sp)
    li      t5, 0x105
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 168(sp)
    li      t5, 0x106
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 176(sp)
    li      t5, 0x107
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 184(sp)
    li      t5, 0x108
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 192(sp)
    li      t5, 0x109
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 200(sp)
    li      t5, 0x10a
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 208(sp)
    li      t5, 0x10b
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 216(sp)
    li      t5, 0x10c
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 224(sp)
    li      t5, 0x10d
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 232(sp)
    li      t5, 0x10e
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 240(sp)
    li      t5, 0x10f
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 248(sp)
    li      t5, 0x110
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 256(sp)
    li      t5, 0x111
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 264(sp)
    li      t5, 0x112
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 272(sp)
    li      t5, 0x113
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 280(sp)
    li      t5, 0x114
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 288(sp)
    li      t5, 0x115
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 296(sp)
    li      t5, 0x116
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 304(sp)
    li      t5, 0x117
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 312(sp)
    li      t5, 0x118
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 320(sp)
    li      t5, 0x119
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 328(sp)
    li      t5, 0x11a
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 336(sp)
    li      t5, 0x11b
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 344(sp)
    li      t5, 0x11c
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 352(sp)
    li      t5, 0x11d
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 360(sp)
    li      t5, 0x11e
    bne     t6, t5, .Lgravity_trap_fail
    ld      t6, 368(sp)
    li      t5, 0x11f
    bne     t6, t5, .Lgravity_trap_fail
    addi    t6, sp, 512
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
    addi    sp, sp, 512
    ret
    .size __gravity_s_trap_roundtrip_test, . - __gravity_s_trap_roundtrip_test
