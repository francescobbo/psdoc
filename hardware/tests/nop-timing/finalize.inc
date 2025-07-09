subu $v0, $t2, $t1
andi $v0, $v0, 0xffff

lw    $ra, 0($sp)
addiu $sp, $sp, 4

jr $ra
nop

.text

#include "../lib.asm"