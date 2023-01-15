ENTRY(_start)

SECTIONS
{
	. = 0x1000;

	.boot :
	{
		*(.multiboot)
	}

	.lowmem :
	{
		*(.lowmem)
	}

	. = 1M;

	.text BLOCK(4K) : ALIGN(4K)
	{
		*(.text)
	}
 
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.rodata)
	}
 
	.data BLOCK(4K) : ALIGN(4K)
	{
		*(.data)
	}
 
	.bss BLOCK(4K) : ALIGN(4K)
	{
		*(COMMON)
		*(.bss)
	}
}
