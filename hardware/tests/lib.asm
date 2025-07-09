move_to_uncached_memory:
    sw $s0, -4($sp)

    lui $s0, 0xa000
    or $ra, $ra, $s0

    jr $ra
    lw $s0, -4($sp)

move_to_cached_memory:
    sw $s0, -4($sp)

    lui $s0, 0x2000
    xor $ra, $ra, $s0

    jr $ra
    lw $s0, -4($sp)
