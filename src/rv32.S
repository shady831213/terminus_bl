.equ REGBYTES, 4

.macro STORE reg, offset
    sw  \reg, \offset*REGBYTES(sp)
.endm

.macro LOAD reg, offset
    lw  \reg, \offset*REGBYTES(sp)
.endm