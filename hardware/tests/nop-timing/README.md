# Timing timing

The purpose of this test was to measure the time it takes to read the PS1 timer.
This is important because this value acts as a baseline for all other
benchmarking tests.

For example if we try to do this:

```
let start = read_timer();
nop();
let end = read_timer();
```

we might expect to read a cycle count of 1 (as NOP should take 1 cycle), but
we are actually also paying the cost of reading the timer, which will skew
the result. This test was an attempt to measure that cost.

Unfortunately, this seemingly simple test sent me down a rabbit hole of
complexity.

## The problem

If you read source code for emulators, or even generally available
documentation, it may look like the timings of the PS1 CPU are deterministic.

For example you may read that:
- an instruction that was found in the I-Cache takes 1 cycle (or rather, does
  not stall the CPU, and will keep executing at 1 cycle per instruction)
- reading from main RAM takes 4 / 5 / 6 cycles (people have different opinions
  on this)
- reading from the timer takes 3 cycles

Most emulators will use a baseline of 2 cycles per instruction, adding more for
specific instructions that are known to take longer. It is absolutely NOT that
simple. Clearly, the MIPS pipeline is having an effect on the timings.

## The hint

I hit this problem really quickly. This code:

```asm
li $t0, 0x1f801100  # Timer address
lh $t1, 0($t0)      # Read timer
lh $t2, 0($t0)      # Read timer again
nop                 # Load delay slot

sub $v0, $t2, $t1   # Subtract the two values
```

is supposed to return the time it takes to read the timer. More precisely, it
should return the time it takes to execute the first `lh` instruction (or
the second one, depending on when exactly the timer value is latched).

This code returns 4, which is acceptable, I suppose. However, I then added a
`nop` in the middle:

```asm
lh $t1, 0($t0)      # Read timer
nop
lh $t2, 0($t0)      # Read timer again
nop
```

Everyone knows that a `nop` takes 1 cycle, right? So the result should be 5,
right? Well, it is not. It is 6. But wait, it gets worse. Let's add two more
`nop`s:

```asm
lh $t1, 0($t0)      # Read timer
nop
nop
nop
lh $t2, 0($t0)      # Read timer again
nop
```

This code returns 6 cycles once again. From this point onward, adding more
`nop`s will finally "normalize" and start adding 1 cycle per `nop`. With 4
`nop`s, the result is 7, with 5 `nop`s it is 8, and so on.

Here's the table of results. Each test has been run 1000 times.

| NOPs | Result    | Note |
|------|-----------|------|
| 0    | 4 cycles  |      |
| 1    | 6 cycles  |      |
| 2    | 6 cycles  |      |
| 3    | 6 cycles  |      |
| 4    | 7 cycles  |      |
| 5    | 8 cycles  |      |
| 6    | 9 cycles  | 6 cycles 40 out 1000 times |
| 7    | 10 cycles | 6 cycles 24 out of 1000 times |
| 8    | 11 cycles |      |
| 9    | 12 cycles |      |

The 6 and 7 tests make things even worse by returning 6 cycles a few times.
This is _not_ random. The tests return the same results every time I run
them, so it is not a matter of noise.

Can the timer lag in its updates?

### Isolate the cache

Not fully understanding what was going on, I tried to run this code in the
uncached segment (`kseg1`).

| NOPs | Result    | Note |
|------|-----------|------|
| 0    | 8 cycles  | 11 cycles 32/1000 times |
| 1    | 12 cycles | 17 cycles 21/1000 times. 18 cycles 22/1000 times |
| 2    | 18 cycles | 22 cycles 42/1000 times. 24 cycles 43/1000 times |
| 3    | 24 cycles | 28 cycles 58/1000 times. 30 cycles 29/1000 times |
| 4    | 30 cycles | 35 cycles 76/1000 times |
| 5    | 36 cycles | 40 cycles 123/1000 times. 41 cycles 64/1000 times |

These results are slightly more understandable (I think). Each single test has a
few attempts that took extra cycles, but this is the result of the DRAM refresh
(probably). In uncached memory, each instruction must be read from RAM (taking
4/5 cycles), but if the RAM is busy being refreshed, the CPU has to wait. So,
depending on how far the CPU is in the refresh cycle, it may take 4 to 6 extra
cycles to execute an instruction.

Apart from that skew, we can see that every additional `nop` adds 6 cycles to
the result (which seems to suggest that reading it from memory took 5 cycles?).

What I cannot (yet) easily explain is why the two `lh` instructions alone took 8
cycles, but adding a `nop` in the middle makes it take 12 cycles. I would have
expected it to be 8 + 6 = 14 cycles?