#include "../consts.asm"

.set noreorder
.section .init, "ax"
.globl _start

_start:
    addiu $sp, $sp, -4
    sw    $ra, 0($sp)

    // Just in case the sw may affect any timing
    nop
    nop
    nop
    nop
    nop

    li $t0, TMR0_COUNTER
    lui $t1, 0
    lui $t2, 0
