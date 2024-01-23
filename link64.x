ENTRY(_start)

SECTIONS
{
	. = 0x40000;
	__main_stack = .;
	. = 0x80000;
	__kernel_start = .;
	.text :	{ 
		KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)	
	}
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
	.data : { 
		PROVIDE(__data_start = .);
		*(.data .data.* .gnu.linkonce.d*)	
	}
	. = ALIGN(0x1000);
	.bss (NOLOAD) : {
		__bss_start = .;
			*(.bss .bss.*)
			*(COMMON)
		. = ALIGN(8);
		__bss_end = .;
	}
	. = ALIGN(0x1000);
	__kernel_end = .;
	/DISCARD/ : { 
		*(.comment) 
		*(.gnu*) 
		*(.note*) 
		*(.eh_frame*) 
		*(.ARM.exidx*)
	}
}

__bss_size = (__bss_end - __bss_start) / 8;
