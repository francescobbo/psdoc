TARGETS := \
  uncached0.exe uncached1.exe uncached2.exe uncached3.exe uncached4.exe uncached5.exe \
  cached0.exe cached1.exe cached2.exe cached3.exe cached4.exe cached5.exe cached6.exe cached7.exe \
  cached8.exe cached9.exe

all: $(TARGETS)

%.exe: %.asm
	@cpp -P $< > $*.tmp.asm
	@llvm-mc -arch=mipsel -filetype=obj $*.tmp.asm -o $*.o
	@ld.lld -T ../link.ld -o $*.elf $*.o
	@llvm-objcopy -O binary $*.elf $*.bin
	@make bin=$(shell pwd)/$*.bin -f ../Makefile build

	@rm -f $*.tmp.asm $*.o $*.elf $*.bin

ship_it:
	make bin=$(bin) -f ../Makefile ship_it