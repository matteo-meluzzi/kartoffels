MEMORY {
    ram : ORIGIN = 0x00100000, LENGTH = 128K
}

SECTIONS {
    . = ORIGIN(ram);

    .text : {
        KEEP(*(.init))
        *(.text .text.*)
        *(.data .data.*)
        *(.rodata .rodata.*)
        *(.sdata .sdata.*)
        *(.srodata .srodata.*)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(.eh_frame)
    } > ram
    
    _stack_end = ORIGIN(ram) + LENGTH(ram) - 16;
}
