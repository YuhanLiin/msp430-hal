/* Cargo doesn't notice changes to this file. After a change, clean the examples so they are linked
   again: cargo clean -p msp430fr247x-hal-examples --target msp430-none-elf
*/

/* DEVICE SELECTION:
   To use MSP430FR2475:
   - Change RAM LENGTH to 0x1800
   (SLASEO7C Table 9-31, p. 73: the MSP430FR2475 has 6KB of RAM, 2000h to 37FFh, and 32KB of
   FRAM, 8000h to FFFFh)
*/

/* INTERRUPT VECTORS IN RAM:
   For examples/ram_vectors.rs:
   - Change RAM LENGTH to 0x1F80 (0x1780 on the MSP430FR2475)
   The interrupt vector table in RAM takes the top 128 bytes of RAM, 3F80h to 3FFFh on the
   MSP430FR2476 (SLAU445I 1.3.6.1, p. 36), and the stack starts at the end of RAM, so RAM must leave
   them out. The other examples work either way.
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