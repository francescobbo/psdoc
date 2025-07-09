#include "./preamble.inc"

lh $t1, 0($t0)
nop
nop
lw $t3, 0($zero)
lh $t2, 0($t0)
nop

#include "./finalize.inc"
