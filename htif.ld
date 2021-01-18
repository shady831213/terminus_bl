HTIF_BASE = 0x01000000;
SECTIONS
{
    . = HTIF_BASE;
    .tohost (NOLOAD): AT(HTIF_BASE){
        *(.tohost .tohost.*)
    }
}
