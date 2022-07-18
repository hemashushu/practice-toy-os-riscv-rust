OUTPUT_ARCH(riscv)
ENTRY(_start)

# 虚拟地址
BASE_ADDRESS = 0x10000;

SECTIONS
{
    . = BASE_ADDRESS;

    text = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    rodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    data = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    bss = .;
    .bss : {

        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}