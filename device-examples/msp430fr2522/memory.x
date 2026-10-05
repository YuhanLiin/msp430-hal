/* Cargo doesn't notice changes to this file. After a change, clean the examples so they are linked
   again: cargo clean -p msp430fr25x2-hal-examples --target msp430-none-elf
*/

/* INTERRUPT VECTORS IN RAM:
   For examples/ram_vectors.rs:
   - Change RAM LENGTH to 0x780
   The interrupt vector table in RAM takes the top 128 bytes of RAM, 2780h to 27FFh on the
   MSP430FR2522 and MSP430FR2512 (SLAU445I 1.3.6.1, p. 36; RAM: SLASEE4C Table 6-19, p. 62), and the
   stack starts at the end of RAM, so RAM must leave them out. The other examples work either way.
*/

/* PROGRAM FRAM LEFT WRITABLE (FRWPOA):
   For examples/frwpoa.rs:
   - Change ROM to ORIGIN = 0xE700, LENGTH = 0x1880
   FRWPOA leaves the start of program FRAM writable, here its first KiB, E300h to E6FFh (SLAU445I
   1.12.4.1, p. 53; SLAU445I Table 1-29, p. 80), so the program must not be in it. Change it back
   afterwards: dco_delay_test doesn't fit in the smaller ROM.
*/

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
