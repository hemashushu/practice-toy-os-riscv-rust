.altmacro

.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # >> 交换 sp 和 sscratch 的值
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # >> 现在 sp 指向 kernel stack，sscratch 指向用户栈

    # >> 在 kernel stack 上分配结构体 TrapContext 的空间
    # >> 该结构体大小是 34 bytes
    # allocate a TrapContext on kernel stack
    addi sp, sp, -34*8
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # >> CSR 寄存器需要使用 csrr 和 csrw 来读写，
    # >> 读到通用寄存器之后再存入内存；
    # >> 或者从内存读取到通用寄存器之后，再写入 CSR。
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # >> `mv a0, sp` 将 sp 的值写入寄存器 a0，而 sp 指向的是
    # >> 结构体 TrapContext 的地址，所以该结构体的实例将作为 trap_handler() 的参数
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap

    # >> ch3 已经不需要这个指令
    # >> https://rcore-os.github.io/rCore-Tutorial-Book-v3/chapter3/3multiprogramming.html#id7
    # mv sp, a0

    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    # now sp->kernel stack, sscratch->user stack
    # >> 交换 sp 和 sscratch
    # >> 注：现在 sp 指向用户栈，sscratch 指向 kernel stack
    csrrw sp, sscratch, sp
    sret
