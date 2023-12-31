.text :	{ KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)	}
. = ALIGN(0x1000);
__rodata_start = .;
.rodata : {	
    *(.rodata* .gnu.linkonce.r*)
    . = ALIGN(0x1000);
    __font_start = .;
    *(.font*)
    __font_end = .;
}
__rodata_end = .;
. = ALIGN(0x1000);
PROVIDE(_data = .);
.data : { 
    *(.data .data.* .gnu.linkonce.d*)	}
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
