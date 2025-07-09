TARGETS := \
  baseline.exe \
  lw-after-timer0.exe lw-after-timer1.exe lw-after-timer2.exe lw-after-timer3.exe lw-after-timer4.exe lw-after-timer5.exe \
  nop-after1.exe nop-after2.exe nop-after3.exe nop-after4.exe nop-after5.exe nop-after6.exe \
  nop-around1.exe nop-around2.exe nop-around3.exe nop-around4.exe nop-around5.exe nop-around6.exe \
  nop-before1.exe nop-before2.exe nop-before3.exe nop-before4.exe nop-before5.exe \
  use-baseline.exe use0.exe use1.exe use2.exe use3.exe use4.exe use5.exe use6.exe use-nop.exe \
  use0-sp.exe use1-sp.exe use2-sp.exe use3-sp.exe 

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