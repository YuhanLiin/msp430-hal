MEMORY
{
  /* These values are correct for the msp430fr2522 device. You will have to
     update accordingly for other devices. */
  RAM : ORIGIN = 0x2000, LENGTH = 0x800
  ROM : ORIGIN = 0xE300, LENGTH = 0x1C80
  VECTORS : ORIGIN = 0xFF80, LENGTH = 0x80
}

/* Stack begins at the end of RAM:
   _stack_start = ORIGIN(RAM) + LENGTH(RAM); */

/* TODO: Code (and data?) above 64kB mark, which is supported even without
   using MSP430X mode. */
