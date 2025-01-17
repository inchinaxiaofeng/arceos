/* link.ld */
ENTRY(_start)
OUTPUT_ARCH(riscv)

BASE_ADDRESS = 0xffffffc080300000;

SECTIONS
{
  /* Load the app at this address: "." means the current address */
  . = BASE_ADDRESS;
  start = .;

  .text : ALIGN(4K) {
    stext = .;
    *(.text.entry)
    *(.text .text.*)
    etext = .;
  }

  .plt : ALIGN(4K) {
    splt = .;
    *(.plt .plt.*)
    eplt = .;
  }

  .rodata : ALIGN(4K) {
    srodata = .;
    *(.rodata .rodata.*)
    erodata = .;
  }

  .data : ALIGN(4K) {
    sdata = .;
    *(.data .data.*)
    edata = .;
  }

  .bss : ALIGN(4K) {
    sbss = .;
    *(.bss .bss.*)
    ebss = .;
  }

  /* Automatically set the global pointer to the start of the .data section */
  PROVIDE(__global_pointer$ = sdata);

  PROVIDE(end = .);
}
