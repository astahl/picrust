ENTRY(_start)

SECTIONS
{
	. = 0x80000;
	.text.boot : { 
		KEEP(*(.text.boot))
	}
	.text.vector : ALIGN(0x800) {
		KEEP(*(.text.vector))
	}
	.text : {
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
	/DISCARD/ : { 
		*(.comment) 
		*(.gnu*) 
		*(.note*) 
		*(.eh_frame*) 
		*(.ARM.exidx*)
	}
}

__bss_size = (__bss_end - __bss_start) / 8;
