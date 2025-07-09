# LW timing

This is a followup from the [nop-timing](../nop-timing/README.md) test. After
having discovered the weirdness of the PS1 CPU timings, I wanted to
investigate further and see if I could measure the time it takes to execute a
load word instruction (`lw`), specifically reading from RAM.

This, once again, turned out not to be that trivial. The timing of `lw` seems
to be dependent to how close it is placed to other "bus" operations, including
reading the timer.

Keep in mind that, from the previous tests, we know that simply reading the
timer twice in a row reports a 4 cycles difference. While adding 1 to 3 `nop`
instructions in between make it return 6.

The baseline test (I'm using $t3 as destination instead of $zero to avoid some
potential optimizations by the CPU) is this:

```asm
lh $t1, 0($t0)    # Read timer
lw $t3, 0($zero)  # Read from RAM
lh $t2, 0($t0)    # Read timer again
nop               # Load delay slot
```

This reports 10 cycles (98% of the time, or 16 cycles - DRAM refreshes?). I
would love to have the pipeline diagram for this, but I don't understand it
yet.

## Adding a NOP before the `lw`

```asm
lh $t1, 0($t0)    # Read timer
nop               # 1 or more NOP instructions placed here
lw $t3, 0($zero)  # Read from RAM
lh $t2, 0($t0)    # Read timer again
nop               # Load delay slot
```

| NOPs | Cycles |
|------|--------|
| 0    | 10     |
| 1    | 12     |
| 2    | 12     |
| 3    | 12     |
| 4    | 13     |
| 5    | 14     |


We can see that adding NOPs has a similar effect to the previous test, where
`nop` instructions added before the load are taking up possibly unused, stall
cycles, instead of adding to the total time? Just an hypothesis.

## Adding a NOP after the `lw`

```asm
lh $t1, 0($t0)    # Read timer
lw $t3, 0($zero)  # Read from RAM
nop               # 1 or more NOP instructions placed here
lh $t2, 0($t0)    # Read timer again
nop               # Load delay slot
```

| NOPs | Cycles |
|------|--------|
| 0    | 10     |
| 1    | 12     |
| 2    | 12     |
| 3    | 12     |
| 4    | 12     |
| 5    | 12     |
| 6    | 13     |

I thought I would see exactly the same results, but weirdly, I could add a whole
5 `nop`s and always get 12 cycles, until I added a 6th one, which
increased the total to 13 cycles.

The mystery deepens.

## NOPs around the `lw`

```asm
lh $t1, 0($t0)    # Read timer
nop               # 1 or more NOP instructions placed here
lw $t3, 0($zero)  # Read from RAM
nop               # Same number of NOPs as before
lh $t2, 0($t0)    # Read timer again
nop               # Load delay slot
```

| NOPs | Cycles |
|------|--------|
| 0    | 10     |
| 1    | 14     |
| 2    | 14     |
| 3    | 14     |
| 4    | 15     |
| 5    | 16     |
| 6    | 18     |

I would expect each step here to add 2 cycles... but we have to wait until 6
NOPs to see this effect.

## LW after the timer

I could not resist this. Will placing `lw` from RAM after two timer reads
affect the timing? I sure hoped not, and I was right.

```asm
lh $t1, 0($t0)    # Read timer
lh $t2, 0($t0)    # Read timer again
nop               # Zero or more NOPs here
lw $t3, 0($zero)  # Read from RAM
nop
```

This consistently returns 4 cycles, regardless of how many `nop`s I place in
between the two timer reads and the `lw` instruction.

## LW but the result is used

This is when things turned even more interesting. You should be familiar with
the load delay slot. If you use the target register of `lw` in the following 
instruction, you will get a stale value. Wait one more instruction however, and
you are promised the read value.

Does it mean that the read is complete 2 cycles after the `lw` instruction? It
seems not. Using the target register 2 cycles after the `lw` instruction seems
to stall the CPU, hinting that the read is not complete yet.

(By the way, this makes the CPU even smarter and more complex than I expected.
At this point why even bother having a load delay slot? If the CPU has the
capability to stall the pipeline, it could just wait for the read to complete it
could just do so right away!).

For this test I placed 12 `NOPs` before and after the `lw` instruction, to have
space to play with the target register without it being potentially affected by
the `lh` for the timers.

I then replace one of the following `NOP`s with an `addu` instruction that uses
the target register. Like this:

```asm
lh $t1, 0($t0)    # Read timer
12x nop
lw $t3, 0($zero)  # Read from RAM
12x nop           # One of these will be replaced with an addu
lh $t2, 0($t0)    # Read timer again
```

| Test     | Cycles    | Notes |
|----------|-----------|-------|
| No `lw` (`nop`)   | 28 cycles |        |
| Baseline          | 30 cycles |        |
| Use after `lw`    | 30 cycles | Stale value (LDS not waited) |
| 1 nop after `lw`  | 34 cycles | Stall! |
| 2 nops after `lw` | 34 cycles | Stall! |
| 3 nops after `lw` | 33 cycles | Stall! |
| 4 nops after `lw` | 32 cycles | Stall! |
| 5 nops after `lw` | 31 cycles | Stall! |
| 6 nops after `lw` | 30 cycles | Not stalled |

So, it would seem that replacing a `nop` with an `lw` goes from 1 cycle to 3
(28 to 30 cycles). However, while the result may not be "programmer visible"
until two instructions later, it seems that the data is actually, really ready
7 whole instructions (or cycles) later. (This may hint at longer RAM read times
like 5 or 6, rather than 4).

When using the scratchad instead of the main RAM as a source, the code always
takes a fixed 28 cycles, regardless of `nop` placements. It looks like the
baseline cost (the 3 cycles) is dependent on the address being accessed.