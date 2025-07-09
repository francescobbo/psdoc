The consumer models of the PlayStation 1 come with 2 MB of RAM, while the
developer boards come with 8 MB.

The 2 MB of RAM are provided in two configurations. In earlier motherboard
revisions, the RAM is provided by four 512KB ICs that act as a single 2 MB
memory bank. In later revisions, the RAM is provided by a single 2 MB IC.

There have been a few different ICs used for the RAM, but they all should be
compatible with each other. Since the CPU is not (should not be) aware of the
specific RAM used, the PS1 should work with any of them. The ICs I have seen
used in the PS1 are (note that `SEC` is Samsung Electronics):

| Manufacturer | Part Number       | Size / Configuration  | Found in    | Datasheet |
|--------------|-------------------|---------------------- |-------------|-----------|
| SEC          | KM48V514BJ-6      | 512Kx8 (4 pieces)     | Up to PU-18 | [Open](/docs/Samsung_DRAM_1995.pdf) |
| SEC          | KM48V514DJ-6      | 512Kx8 (4 pieces)     | PU-20       | [Open](/docs/Samsung_DRAM_1995.pdf) |
| SEC          | KM48V2104AJ-6     | 2Mx8  (4 pieces)      | DTL-H2000   | [Open](/docs/Samsung_DRAM_1995.pdf) |
| SEC          | KM48V2104AT-6     | 2Mx8  (4 pieces)      | DTL-H2500   | [Open](/docs/Samsung_DRAM_1995.pdf) |
| NEC          | (µPD)424805AL-A60 | 512Kx8 (4 pieces)     | Up to PU-20 | [Open](/docs/NEC_Dynamic_RAMs_1996.pdf) |
| OKI          | M51V4805D-60J     | 512Kx8 (4 pieces)     | PU-20       | Missing |
| Toshiba      | T7X16             | 512Kx32 (1 piece)     | PU-8*       | Missing |
| SEC          | K4Q153212M-JC60   | 512Kx32 (1 piece)     | PU-8*       | [Open](/docs/Samsung_512Kx32.pdf) |

*The single 512Kx32 ICs have been used in the late revisions of most boards,
and consistently since PU-22.

It is likely that other ICs have been used in the PS1. I could not find all
datasheets. If you have more information, please [open an issue](/issues/new) or
[send a pull request](/pulls/new).

## Connections and addressing

The four-chip 512KB ICs configuration was used in the earlier revisions of the
PS1. From the point of view of the CPU, this configuration is equivalent to a
single 2 MB IC.

The CPU has 12 physical address pins. Two of these are not connected to any
memory IC. DRAM memory is organized in a grid of rows and columns. When
accessing a DRAM memory, the physical memory address (the one the CPU uses) is
divided into a row address and a column address.

When the CPU wishes to access memory it places the row address on the address
bus and triggers the <neg>RAS</neg> signal. The memory ICs activates the
corresponding row, and then the CPU places the column address on the address bus
and triggers the <neg>CAS</neg> signal.

All chips used expect a 10-bit row address and a 9-bit column address, for a
total of 2^19 = 524,288, unique locations. The CPU always perform 4-byte aligned
accesses, where the lower two bits of the address are always `00`. These two
bits are not even part of the physical address.

In theory, the pins on the CPU would allow for 2^(12 + 12 + 2) unique locations,
or 64 MB of memory. However in practice the CPU will always use just 9 bits for
the column address, so the maximum addressable memory (per bank) is 2^(10 + 9 +
2) = 8 MB, which is actually used in development boards.

## Addressing individual bytes

While there's a requirement for aligned memory accesses, the MIPS CPU still has
instructions like LB that allow for unaligned byte accesses. The CPU actually
has 4 <neg>CAS</neg> pins. Each of these pins is connected to a different IC
chip (in the older configuration). Accessing a byte at an address like 0x4321
would cause the CPU to place 0x4320 on the address bus (split appropriately),
and then only trigger <neg>CAS1</neg>, leaving <neg>CAS0/2/3</neg> untouched.

At that point only one of the 4 chips will have received the full address and
will provide an "unaligned" response (it will be placed in data lines 8 to 15,
and the CPU will read it as a byte).

This is also how complex instructions like SWL/R and LWL/R are implemented: by
only triggering the appropriate <neg>CAS</neg> pins, the CPU can effectively
read or write 1, 2 or 3 bytes from an apparently unaligned address.

Integrated, all-in-one, memory chips from the later revisions use the same
addressing scheme, retaining the four <neg>CAS</neg> pins, but in a single
package.

What I find interesting is that, at least in the earlier revisions, the actual
stored data is interleaved across the four chips. When reading an entire 32-bit
word, the CPU will trigger all four <neg>CAS</neg> pins, and each chip will
provide a different byte of the word.

## The second memory bank

Besides the 4 <neg>CAS</neg> lines, the CPU also has two <neg>RAS</neg> lines.
<neg>RAS1</neg> is sadly unutilized in consumer models. However, with some
patience and soldering skills it is possible to add a second RAM bank, expanding
the total memory to a theoretical maximum of 16 MB (2 banks of 8 MB each).

The I/O Port at `1f801060` can be used to configure the DRAM controller. In
particular bit 10 of this register enables the use of the second bank. Bits 9
and 11, together, control the size of each bank (they have to match), where each
can be 1, 2, 4 or 8 MB in size.

| Bit 9 | Bit 11 | Bank size |
|-------|--------|-----------|
| 0     | 0      | 1MB       |
| 0     | 1      | 2 MB       |
| 1     | 0      | 4 MB       |
| 1     | 1      | 8 MB       |

### Memory mirrors

It is often said that the PS1 2 MB of RAM is mirrored across the first 8 MB of
the physical address space. That is, accessing address `0x100` or `0x200100`
will return the same data. This is correct in practice and results follows from
a misconfiguration of the DRAM controller register by the BIOS.

At boot, the BIOS sets a value to the DRAM controller register, setting bits 9
and 11 to `1`. Accordingly, this informs the controller that a memory bank is 8
MB in size. When the CPU tries to access an address in the mirrored region, like
at `0x200100`, instead of reporting a Bus Error (as it should), the DRAM
controller will attempt to access the first bank of memory, splitting the
address into a 9-bit column address and an invalid 12-bit row address (instead
of 10). Since the upper two physical address lines are disconnected, the RAM ICs
will still receive the usual 10-bit row address, and behave as if the address
was in fact `0x100`.

Morale: don't lie to your hardware.

## DRAM refresh

If you ever benchmarked code on the PS1 you may have noticed that execution time
is not always deterministic.

Seemingly at random, an instruction expected to take 5 cycles will take 10 (or more). There are two likely explainations:

- The instruction cache had a miss, and the CPU did a burst-load (more on this
later).
- The DRAM controller was busy performing a row refresh.

The cache miss is more easily understood, as the CPU will have to wait for data
to be loaded from RAM. To understand DRAM refreshes, you need to understand how
DRAM works.

Dynamic RAM (DRAM) is a type of memory that uses capacitors to store bits.
These capacitors leak charge over time, so, periodically, they need to be
re-written with the original value.

To keep the memory chips simple (not requiring a crystal oscillator or other
timing circuitry), the DRAMs do not perform this operation on their own, but
require an external controller to do so. This is one of the jobs of the DRAM
controller in the CPU.

Refreshes happen row by row. The RAM chips used in the PS1 have 1024 rows that
must be refreshed each 16ms. To avoid halting the system for a long time, the
refresh operations are staggered. The controller refreshes a row every 16ms/1024
= 15.625us, or approximately every 592(.4) CPU cycles. If the CPU needs to
access RAM during one of these brief refresh cycles, it will be forced to wait.

These chips support 3 different refresh modes that I'll describe later, but the
PS1 CPU uses a common one called CAS-before-RAS.

## Access timings

The PS1 CPU incurs a 4-cycle cost when accessing the RAM (unconfirmed).

I will report here the important timings (there's too many) from the datasheets
of the SEC K4Q153212M-JC60 (which are conveniently identical to the SEC
KM48V514BJ-6). The NEC 424805AL-A60 has very similar, or identical values. These
are always "bounds", as in minimum and/or maximum timings. If the chip user (the
CPU) doesn't respect these timings, the chip will respond with undefined data.
Importantly, the CPU makes pin changes at either the rising or falling edge of
the clock, so from its perspective, the timings are always aligned to half a
clock cycle.

| Symbol | Parameter | Min | Max | Notes |
|--------|-----------|-----|-----|-------|
| tRC    | Random read or write cycle time | 104ns |  | The minimum time between the start of a read or write operation and the start of the next read or write operation. |
| tRAC   | Access time from <neg>RAS</neg> | | 60ns | The time from the falling edge of <neg>RAS</neg> to the available data on the data bus, assuming tRCD is respected. |
| tCAC   | Access time from <neg>CAS</neg> | | 17ns | The time from the falling edge of <neg>CAS</neg> to the available data on the data bus. |
| tCEZ   | Output buffer turn-off delay from <neg>CAS</neg> | 3ns | 15ns | The time from the rising edge of <neg>CAS</neg> to the data bus becoming high-impedance (inactive). |
| tRP    | <neg>RAS</neg> precharge time | 40ns |  | The minimum time between the end of a read or write operation and the start of the next read or write operation. |
| tRAS   | <neg>RAS</neg> pulse width | 60ns | 10ms | How long <neg>RAS</neg> must (and may) stay low.* |
| tCAS   | <neg>CAS</neg> pulse width | 12ns | 10ms | How long <neg>CAS</neg> must (and may) stay low. |
| tRCD   | <neg>RAS</neg> to <neg>CAS</neg> delay | 20ns | 43ns | The time between the falling edge of <neg>RAS</neg> and the falling edge of <neg>CAS</neg>. The maximum is a suggestion, in order to hold the tRAC timings. |
| tRAD   | <neg>RAS</neg> to column address delay | 15ns | 30ns | The time between the falling edge of <neg>RAS</neg> when the column address can be placed on the address bus. The maximum is a suggestion, in order to hold the tRAC timings. |
| tCRP   | <neg>CAS</neg> to <neg>RAS</neg> precharge time | 5ns | | The minimum time between the rising edge of <neg>CAS</neg> and the start of the next read or write operation. |
| tASR   | Row address set-up time | 0ns |  | For how long the row address must be stable before the falling edge of <neg>RAS</neg>. In this case, it can be set at the same time as the falling edge of <neg>RAS</neg>. |
| tRAH   | Row address hold time | 10ns |  | For how long the row address must be stable after the falling edge of <neg>RAS</neg>. |
| tASC   | Column address set-up time | 0ns |  | For how long the column address must be stable before the falling edge of <neg>CAS</neg>. |
| tCAH   | Column address hold time | 0ns |  | For how long the column address must be stable after the falling edge of <neg>CAS</neg>. |
| tRCH   | Read command set-up time | 0ns |  | For how long the W signal must be high before the falling edge of <neg>CAS</neg> to indicate a read operation. |
| tOEA   | <neg>OE</neg> access time | | 15ns | The time from the falling edge of <neg>OE</neg> to valid data being available on the data bus. |
| tCSR   | <neg>CAS</neg> set-up time in CBR mode | 5ns |  | For how long the <neg>CAS</neg> signal must be stable before the falling edge of <neg>RAS</neg> in CBR mode. |

__*__ RAS can be held low for a while, and CAS can oscillate between high and low
states, reading multiple columns in the same row, before it is released. This is
called a "burst read" and is used by the instruction cache to load multiple
instructions at once, and by the DMA controller to efficiently transfer data
from RAM to other devices. However, one must not forget to refresh the rest of
the rows in the meantime, so burst reads cannot exceed 10ms in total (enough
to read or write ~1.5KB in Hyper Page mode).

In short, to perform a read operation, it will take, with perfect timings and no
delays, at least tRC, 104ns or 3.5 cycles. Keeping in mind that 1 CPU cycle
takes 29.5ns (14.75ns for an half-cycle), this is a likely (but unconfirmed)
sequence of events:

1. At the start of cycle 1, the CPU places the row address on the address bus
   and triggers <neg>RAS</neg>. The row address must be kept for tRAH, 10ns, or
   half a cycle.
2. tRCD(min) is 20ns, which means an half-cycle later is too soon to start 
   triggering <neg>CAS</neg>. The CPU waits for the start of cycle 2.
3. At the start of cycle 2, the CPU places the column address on the address bus
   and triggers <neg>CAS</neg>, keeping <neg>W</neg> high and bringing
   <neg>OE</neg> low to indicate a read operation. Data will be available in
   tOEA: 15ns or, sadly, just a tiny bit over half a cycle (meaning that the
   remanining half-cycle will be wasted).
4. At the start of cycle 3, the data is fully available on the data bus, the CPU
   reads it, and then releases <neg>OE</neg>, <neg>CAS</neg> and <neg>RAS</neg>.
   A further tRP of 40ns is required before the next operation can be performed.
   To avoid a conflict, the CPU stalls for 40ns, or 1.35 cycles.
5. By the end of cycle 4, tRP is satisfied, and the CPU can start the next
   operation.

So it looks like the CPU is not able to do anything on the half-cycle edge, and
all operations happen at the start of a cycle. If this schedule is correct, it
would mean that the cost for the CPU to access the RAM is indeed 4 cycles.
Hardware tests to follow!

A load operation takes in total 5 cycles. One cycle is the baseline cost of
executing an instruction, and 4 cycles are the cost of accessing the RAM.

I will not be describing the timings for write operations, as they are
essentially the same, except that the CPU will trigger <neg>W</neg> low.

## Burst reads

A nice characteristic of these memory chips is that they support EDO or Extended
Data Out mode, also called Hyper Page mode. This is used by the CPU instruction
cache to load multiple instructions very quickly and by the DMA controller to
transfer data from RAM to other devices.

This feature allows the controller to read multiple columns in the same row,
while keeping <neg>RAS</neg> low, and <neg>CAS</neg> oscillating between high
and low states, every time with a different column address on the address bus.
By the time the data has been made available on the data bus, the CPU will
already have placed the next column address on the address bus, and the
controller will be able to read it immediately.

This effectively allows the CPU to incur the tRCD(min) cost only once. As soon
as data is available in cycle 3, it will be read, and a new column address will
be placed on the address bus. This second read will be available immediately in
cycle 4 and so on, until the controller has read everything it needs. It will
then release <neg>RAS</neg> and pay the tRP cost before performing the next
operation.

With this set-up, the timeline for reading 4 columns (or 16 bytes) looks like
this:

- The first word is available at the start of cycle 3.
- The second word is available at the start of cycle 4.
- The third word is available at the start of cycle 5.
- The fourth word is available at the start of cycle 6. <neg>RAS</neg> is
  released at this point.
- The CPU stalls until the end of cycle 7, when tRP is satisfied.

Not bad! Notice however, that these timings do not account for the actual
execution of the instruction that triggered the read operation (typically
another cycle), and for any delay that's necessary to write in the instruction
cache (which is unknown to me).

The benefits of EDO mode are even more significant with DMA, where the CPU can
potentially read hundreds of words in a single burst read (as long as tRAS is
respected and refreshes are performed).

## <neg>CAS</neg>-before-<neg>RAS</neg> refresh

The oldest, and "cheapest" approach to memory refreshes, is to delegate the
entire logic to the DRAM chip. Let's start with a new detail: activating a row
for a read or a write implicitly refreshes the entire row.

This means that a program that frequently accesses all rows of memory would, in
theory, remove the need for the DRAM controller to perform the refreshes. This
is rarely the case. However it hints at how this first type of refreshes are
done. These are called <neg>RAS</neg>-only refreshes. At the determined interval
the DRAM controller will place a row address on the bus, trigger <neg>RAS</neg>,
wait tRAS, release, and wait tRP. This takes exactly the same time as a regular
operation (tRC), or 4 cycles.

However this also needs the DRAM controller to keep a counter, and, if it wants
to be real smart, to keep track of recently accessed rows (which wouldn't need
a refresh).

A better approach, that moves some of this effort to the DRAM chip itself is the
<neg>CAS</neg>-before-<neg>RAS</neg> refresh. In this case it is responsibility
of the DRAM chip to keep track of the rows that need to be refreshed, usually
through a simple counter.

To perform this operation, the DRAM controller triggers, as the name implies,
<neg>CAS</neg> first. It then has to wait tCSR (5ns) before triggering
<neg>RAS</neg> for the usual tRAS + tRP. The total time is slightly longer, but
<confirm>should still fit in the same 4 cycles</confirm>.

This is the refresh mode used by the PS1 CPU.

## Mixed capabilities

The DRAM chips can perform more interesting operations like:

- atomic read and write operations to the same cell (read-modify-write), which
   would be useful in a multi-core system.
- mixed burst read-write operations, where the controller can read a column,
   write a column, and then read another column, or atomically read and write
   all in the same burst.
- a third refresh mode called the "hidden refresh", where the controller starts
   a <neg>CAS</neg>-before-<neg>RAS</neg> refresh, chaining it to a previous
   read or write operation.

I do not believe any of these are actually used in a PS1.

## Hardware tests to perform

1. Verify the timings of the RAM access, to confirm that the CPU incurs a
   4-cycle cost when accessing the RAM.
2. Read the same address from the RAM twice in a row (code must be cached for
     this to work, or the instruction fetch will interfere). The second
     read will take the same time as the first (probably) or less if the
     DRAM controller was smart enough to keep the row active.
3. Read a different address in the same row, like the previous test.
4. Read two words in different rows. The timing should be the same 4 + 1
     cycles.
5. Determine the cost of a burst instruction-cache read. This is _probably_
     7 cycles, but it would be nice to confirm.
6. Run memory load operations while in uncached mode. Each instruction should
     take 4 + 4 + 1 = 9 cycles, where the first 4 cycles are the cost of
     fetching the instruction, the second 4 cycles are the cost of accessing
     the RAM, and the last cycle is the cost of executing the instruction.
7. Try to find exactly how many cycles go between two DRAM refreshes. This
     should be somewhere around 592 cycles.
8. Determine how many cycles it takes to perform a DRAM refresh. This
     should be 4 cycles.

Way more precise timings could be obtained by using a logic analyzer connected
to one of the ICs, but I do not have one.