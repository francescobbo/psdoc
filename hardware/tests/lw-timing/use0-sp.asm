#include "./preamble.inc"

li $t3, SCRATCHPAD_START

lh $t1, 0($t0)
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
lw $t3, 0($t3)
addu $t4, $t3, $t3
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
lh $t2, 0($t0)
nop

#include "./finalize.inc"
