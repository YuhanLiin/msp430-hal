MEMORY
{
  /* These values are correct for the msp430fr2522 device. You will have to
     update accordingly for other devices. Memory organization of the MSP430FR2522 and
     MSP430FR2512: SLASEE4C Table 6-19, p. 62. */
  /* RAM: 2KB, 2000h to 27FFh */
  RAM : ORIGIN = 0x2000, LENGTH = 0x800
  /* Program FRAM: 7.25KB, E300h to FFFFh, minus the vectors and signatures at its top */
  ROM : ORIGIN = 0xE300, LENGTH = 0x1C80
  /* Main: interrupt vectors and signatures, FF80h to FFFFh (also SLASEE4C 6.4, p. 45;
     SLASEE4C Table 6-2, p. 46; SLASEE4C Table 6-3, p. 46) */
  VECTORS : ORIGIN = 0xFF80, LENGTH = 0x80
}

/* Stack begins at the end of RAM:
   _stack_start = ORIGIN(RAM) + LENGTH(RAM); */

/* TODO: Code (and data?) above 64kB mark, which is supported even without
   using MSP430X mode. The MSP430FR25x2 has no FRAM above 64kB, only the 1KB BSL2 ROM
   at FFC00h to FFFFFh (SLASEE4C Table 6-19, p. 62). */
