ENTRY(_start)

SECTIONS
{
	__stack_bottom = .;
	. = 0x80000;
	__stack_top = .;
	__kernel_start = .;
	.text.boot : { 
		KEEP(*(.text.boot))
	}
	.text.vector : ALIGN(0x800) {
		KEEP(*(.text.vector))
	}
	.text : ALIGN(0x1000) {
		*(.text .text.* .gnu.linkonce.t*)	
	}
	.rodata : ALIGN(0x1000) {	
		*(.rodata* .gnu.linkonce.r*)
	}
	.data : ALIGN(0x1000) { 
		*(.data .data.* .gnu.linkonce.d*)	
	}
	.bss (NOLOAD) : ALIGN(0x1000) {
		__bss_start = .;
		*(.bss .bss.*)
		*(COMMON)
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

__kernel_size = (__kernel_end - __kernel_start) / 8;
