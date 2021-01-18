    .section .init, "ax"
    .global _start
_start:
    .globl _abs_start
_abs_start:
    .option push
    .option norelax
    la  gp, __global_pointer$
    .option pop
    la  sp, _sstack
    j main

_start_abort:
    wfi
    j   _start_abort