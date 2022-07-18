.altmacro

.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    # >> ch4 将 `.text` 改为 `.text.trampoline`
    # .section .text
    .section .text.trampoline

    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # >> 交换 sp 和 sscratch 的值
    csrrw sp, sscratch, sp
    # now sp-> *TrapContext in user space, sscratch->user stack
    # >> 现在 sp 指向 *TrapContext in user space（而不是之前的 kernel stack），sscratch 指向用户栈

    # >> ch4 注释了下面几行
    # # >> 在 kernel stack 上分配结构体 TrapContext 的空间
    # # >> 该结构体大小是 34 bytes
    # # allocate a TrapContext on kernel stack
    # addi sp, sp, -34*8

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
    # ch4 MODIFY:: # we can use t0/t1/t2 freely, because they were saved on kernel stack
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # ch4 MODIFY:: # read user stack from sscratch and save it on the kernel stack
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # ch4 注释了下面几行
    # # >> `mv a0, sp` 将 sp 的值写入寄存器 a0，而 sp 指向的是
    # # >> 结构体 TrapContext 的地址，所以该结构体的实例将作为 trap_handler() 的参数
    # # set input argument of trap_handler(cx: &mut TrapContext)
    # mv a0, sp
    # call trap_handler

    # >> ch4 新增

    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0

    # 清空 TLB
    sfence.vma

    # jump to trap_handler
    # >> 这里不能使用 call 指令了，因为编译器会对 call 指令使用相对地址，
    # >> 即 trap_handler 相对 call 指令的地址，显然这个相对值在切换地址空间之后
    # >> 是不正确的。所以需要使用 jr 指令。
    jr t1

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap

    # ch4 新增

    # a0: *TrapContext in user space(Constant);
    # a1: user space token (即 satp CSR 寄存器的值)
    # switch to user space
    csrw satp, a1
    sfence.vma

    csrw sscratch, a0
    mv sp, a0

    # ch4 注释了下面几行
    # # now sp->kernel stack(after allocated), sscratch->user stack
    # # restore sstatus/sepc
    # ld t0, 32*8(sp)
    # ld t1, 33*8(sp)
    # ld t2, 2*8(sp)
    # csrw sstatus, t0
    # csrw sepc, t1
    # csrw sscratch, t2

    # ch4 新增
    #
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc

    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # restore general-purpuse registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    # ch4 注释了下面几行
    # # release TrapContext on kernel stack
    # addi sp, sp, 34*8
    # # now sp->kernel stack, sscratch->user stack
    # # >> 交换 sp 和 sscratch
    # # >> 注：现在 sp 指向用户栈，sscratch 指向 kernel stack
    # csrrw sp, sscratch, sp

    # ch4 新增
    #
    # back to user stack
    ld sp, 2*8(sp)

    sret
