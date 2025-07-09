ENTRY(_start)

SECTIONS {
  . = 0x80040000;
  .init : { *(.init) }
  .text : { *(.text) }

  .shstrtab : { *(.shstrtab) }

  /DISCARD/ : {
    *(*)
  }
}
