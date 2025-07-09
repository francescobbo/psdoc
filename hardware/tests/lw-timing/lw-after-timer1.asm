#include "./preamble.inc"

lh $t1, 0($t0)
lh $t2, 0($t0)
nop
lw $t3, 0($zero)
nop

#include "./finalize.inc"
