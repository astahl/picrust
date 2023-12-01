ENTRY(_start)

SECTIONS
{
	/* Starts at LOADER_ADDR. */
	. = 0x80000;
	__start = .;
	.text :	{ KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)	}
	.rodata : {	*(.rodata .rodata.* .gnu.linkonce.r*) }
    PROVIDE(_data = .);
	.data : { *(.data .data.* .gnu.linkonce.d*)	}
    . = ALIGN(0x1000);
    __bss_start = .;
	.bss : {
		*(.bss .bss.*)
        *(COMMON)
	}
	__bss_end = .;
	__end = .;

   /DISCARD/ : { 
    *(.comment) 
    *(.gnu*) 
    *(.note*) 
    *(.eh_frame*) 
    *(.ARM.exidx*)
    }
}

