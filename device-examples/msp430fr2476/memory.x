/* Cargo doesn't notice changes to this file. After a change, clean the examples so they are linked
   again: cargo clean -p msp430fr247x-hal-examples --target msp430-none-elf
*/

/* DEVICE SELECTION:
   To use MSP430FR2475:
   - Change RAM LENGTH to 0x1800
   (SLASEO7C Table 9-31, p. 73: the MSP430FR2475 has 6KB of RAM, 2000h to 37FFh, and 32KB of
   FRAM, 8000h to FFFFh)
*/

MEMORY
{
  /* Current Values for MSP430FR2476 (SLASEO7C Table 9-31, p. 73): 8KB of RAM, 2000h to 3FFFh, and
     64KB of FRAM, 8000h to 17FFFh, which holds the interrupt vectors and signatures at FF80h to
     FFFFh (also SLASEO7C 9.4, p. 46: "The interrupt vectors and the power-up start address are in
     the address range 0FFFFh to 0FF80h").
     ROM ends where VECTORS starts, so the two regions don't overlap, as on the other devices. The
     FRAM above FFFFh, 10000h to 17FFFh, is left out: the code is built for the MSP430 instruction set
     (-mcpu=msp430 in .cargo/config.toml), and "Only addresses in the lower 64KB address range can be
     reached with the BR or CALL instruction" (SLAU445I 4.3.1, p. 128). */
  RAM     : ORIGIN = 0x2000, LENGTH = 0x2000
  ROM     : ORIGIN = 0x8000, LENGTH = 0x7F80
  VECTORS : ORIGIN = 0xFF80, LENGTH = 0x80
}