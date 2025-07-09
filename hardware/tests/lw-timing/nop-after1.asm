#include "./preamble.inc"

lh $t1, 0($t0)
lw $t3, 0($zero)
nop
lh $t2, 0($t0)
nop

#include "./finalize.inc"
