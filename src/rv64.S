.equ REGBYTES, 8

.macro STORE reg, offset
    sd  \reg, \offset*REGBYTES(sp)
.endm

.macro LOAD reg, offset
    ld  \reg, \offset*REGBYTES(sp)
.endm