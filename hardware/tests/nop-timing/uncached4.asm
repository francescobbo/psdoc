#include "./preamble.inc"

jal move_to_uncached_memory
nop

lh $t1, 0($t0)
nop
nop
nop
nop
lh $t2, 0($t0)
nop

#include "./finalize.inc"
