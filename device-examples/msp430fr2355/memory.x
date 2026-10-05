/* MSP430FR2355 memory map: SLASEC4D 6.4, Table 6-4, p. 65 */

/* Cargo doesn't notice changes to this file. After a change, clean the examples so they are linked
   again: cargo clean -p msp430fr2355-hal-examples --target msp430-none-elf
*/

/* INTERRUPT VECTORS IN RAM:
   For examples/ram_vectors.rs:
   - Change RAM LENGTH to 0xF80
   The interrupt vector table in RAM takes the top 128 bytes of RAM, 2F80h to 2FFFh on the
   MSP430FR2355 (SLAU445I 1.3.6.1, p. 36; RAM: SLASEC4D Table 6-4, p. 65), and the stack starts at
   the end of RAM, so RAM must leave them out. The other examples work either way.
*/

/* PROGRAM FRAM LEFT WRITABLE (FRWPOA):
   For examples/frwpoa.rs:
   - Change ROM to ORIGIN = 0x8400, LENGTH = 0x7B80
   FRWPOA leaves the start of program FRAM writable, here its first KiB, 8000h to 83FFh (SLAU445I
   1.12.4.1, p. 53; SLAU445I Table 1-24, p. 75), so the program must not be in it. The other examples
   fit in the smaller ROM too.
*/

MEMORY
{
  /* RAM: 4KB, 2000h to 2FFFh (SLASEC4D Table 6-4, p. 65) */
  RAM : ORIGIN = 0x2000, LENGTH = 0x1000
  /* Main FRAM code memory: 8000h up to the interrupt vectors and signatures at FF80h to FFFFh
     (SLASEC4D Table 6-4, p. 65) */
  ROM : ORIGIN = 0x8000, LENGTH = 0x7F80
  /* Interrupt vectors, ending with the reset vector at FFFEh (SLASEC4D 6.3, Table 6-2, p. 63 to
     p. 64). Sized for the 45 interrupt vectors of the msp430fr2355 PAC plus the reset vector; the
     vectors below FFCEh are "Reserved" (SLASEC4D Table 6-2, p. 64). It stays above the BSL I2C
     address at FFA0h and the signatures at FF80h to FF8Bh (SLASEC4D Table 6-3, p. 64). */
  VECTORS : ORIGIN = 0xFFA4, LENGTH = 0x5C
}
