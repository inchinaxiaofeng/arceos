/* linker.ld */
ENTRY(_start)
OUTPUT_ARCH(riscv64gc)

BASE_ADDRESS = 0xffffffc080100000;

SECTIONS
{
  /* Load the app at this address: "." means the current address */
  . = BASE_ADDRESS;
  start = .;

  .text : {
    stext = .;
    *(.text.entry)
    *(.text .text.*)
    . = ALIGN(4K);
    etext = .;
  }

  .rodata : {
    srodata = .;
    *(.rodata .rodata.*)
    . = ALIGN(4K);
    erodata = .;
  }

  .data : {
    sdata = .;
    *(.data .data.*)
    edata = .;
  }

  .stack : {
    *(.bss.stack)
  }

  .bss : {
    sbss = .;
    *(.bss .bss.*)
    ebss = .;
  }

  PROVIDE(end = .);
}
