ENTRY(_start)

SECTIONS
{
	. = 0x80000;
	__main_stack = .;
	__kernel_start = .;
	.text :	{ 
	__kernel_txt_start = .;
		KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)	
	__kernel_txt_end = .;
	}
	. = ALIGN(0x1000);
	.rodata : {	
		__rodata_start = .;
		*(.rodata* .gnu.linkonce.r*)
		. = ALIGN(0x1000);
		__font_start = .;
		*(.font*)
		__font_end = .;
		__rodata_end = .;
	}
	. = ALIGN(0x1000);
	.data : { 
		PROVIDE(__data_start = .);
		*(.data .data.* .gnu.linkonce.d*)	
		__data_end = .;
	}
	. = ALIGN(0x1000);
	.bss (NOLOAD) : {
		__bss_start = .;
			*(.bss .bss.*)
			*(COMMON)
		. = ALIGN(8);
		__bss_end = .;
	}
	__kernel_end = .;
	. = ALIGN(0x1000);
	__free_memory_start = .;
	/DISCARD/ : { 
		*(.comment) 
		*(.gnu*) 
		*(.note*) 
		*(.eh_frame*) 
		*(.ARM.exidx*)
	}
}

__bss_size = (__bss_end - __bss_start) / 8;
