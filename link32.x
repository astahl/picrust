ENTRY(_start)

SECTIONS
{
	/* Starts at LOADER_ADDR. */
	. = 0x8000;
	__start = .;
	INCLUDE link_common.x
}

