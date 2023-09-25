/* Place _start procedure at the entry address for RPI */
__rpi_32_phys_binary_load_addr = 0x8000;
__isr_table_addr = 0;
__stack_size = 0x300000; /* 3MB stack */
ENTRY(__rpi_32_phys_binary_load_addr)   /* enry point */

SECTIONS
{
    .isr_table (NOLOAD) :  
    {
        . = __isr_table_addr;
        /* allocate space for the table */
        . = . + 0x40;
    }
    . = __rpi_32_phys_binary_load_addr;
    .text : 
    {
        KEEP(*(.text._start)) /*put _start first, `KEEP` disables linker optimizations*/
        *(.text*)
    }
    /*readonly data - readonly global variables*/
    .rodata :
    {
        *(.rodata*)
    }
    /*global variables*/
    .data :
    {
        *(.data*)
    }
    .stack (NOLOAD) : ALIGN(16)
    {
        . = . + __stack_size;
        __cpu0_stack_start = .; /* stack grows from high address to low address */
    }
    /*bss must be at the end of the linker script in order to keep the image small (otherwise objcopy will gap the space between bss and the next section)*/
    /*uninitialized global variables*/
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*)
        __bss_end = .;
    }
    /* uncached memory - used by drivers and devices */
    /* Since L1 memory map entry is 1MB, make sure it's alligned correctly*/
    .uncached (NOLOAD) : ALIGN(0x100000)
    {
        __uncached_data_start = .;
        __cached_data_map_size = __uncached_data_start / 0x100000;
        *(.uncached*)
    }

    /* Remove those sections from the final binary */
    /DISCARD/ : 
    {
        *(.ARM.attributes)
        *(.ARM.exidx) /* Used for stack unwinding - not relevant for now */  
        *(.comment) /* comments about the compiler and linker - not intresting */
    }
}