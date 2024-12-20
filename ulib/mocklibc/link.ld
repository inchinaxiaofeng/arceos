OUTPUT_ARCH(riscv)
ENTRY(_start)             

BASE_ADDRESS = 0xffffffc080100000;

SECTIONS

{
    . = BASE_ADDRESS;
    

    .text : ALIGN(4K) {
        *(.text.entry)   
        *(.text .text.*)
    }
    
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    
    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.sdata .sdata.*)

    }
    

    .bss : ALIGN(4K) {
        *(.bss.stack)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    
    /DISCARD/ : {
        *(.eh_frame)
        *(.note.GNU-stack)
    }
}
