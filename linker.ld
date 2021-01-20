OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80000000;
KERNAL_ENTRY = 0x80200000;
PROVIDE(_stack_size = 4K);
PROVIDE(_heap_size = 16K);

SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    start = .;

    .text : {
        _stext = .;
        /* Place init sections first */
        KEEP(*(.init));
        KEEP(*(.init.rust));
        *(.text .text.*)
        _etext = .;
    }

    .rodata : {
        _srodata = .;
        *(.srodata .srodata.*);
        *(.rodata .rodata.*)
        /* 4-byte align the end (VMA) of this section.
        This is required by LLD to ensure the LMA of the following .data
        section will have the correct alignment. */
        . = ALIGN(4);
        _erodata = .;
    }

    .data : {
        sidata = LOADADDR(.data);
        _sdata = .;
        /* Must be called __global_pointer$ for linker relaxations to work. */
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.* .sdata2 .sdata2.*);
        *(.data .data.*)
        _edata = .;
    }

    .stack (NOLOAD) : ALIGN(4K) {
        _estack = .;
        . += _stack_size;
        _sstack = .;
    }


    .bss(NOLOAD) : {
        _sbss = .;
        *(.bss .bss.*)
        _ebss = .;
    }

    .heap (NOLOAD) : {
        _sheap = .;
        . += _heap_size;
        . = ALIGN(4);
        _eheap = .;
    }

    . = KERNAL_ENTRY;
    .payload : {
		_payload_start = .;
		*(.payload)
		. = ALIGN(8);
		_payload_end = .;
	}

    PROVIDE(end = .);
    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr);
    }
}