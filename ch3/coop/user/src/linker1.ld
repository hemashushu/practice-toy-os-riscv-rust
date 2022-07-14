OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80420000;

SECTIONS
{
    . = BASE_ADDRESS;

    text = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    rodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    data = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    .bss : {
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        end_bss = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}