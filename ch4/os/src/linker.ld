OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)

        # ch4 新增 ----------\
        . = ALIGN(4K);       | ALIGN(4k) 是为了让这段 strampoline 代码/指令单独一个 page
        strampoline = .;     | 因为这个跳板 page 是所有应用共享的（位于内存最高处），
        *(.text.trampoline); | 跳板是为了让 trap 切换时，指令是平滑的（连续的），即切换任务时
        . = ALIGN(4K);       | 伴随着地址空间的切换，让指令在地址空间切换后仍然能继续执行。
        # ch4 新增 ----------/

        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .; # ch4 新增
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}