ENTRY(_start)

SECTIONS
{
	/* Starts at LOADER_ADDR. */
	. = 0x80000;
	__start = .;
	INCLUDE link_common.x
}

