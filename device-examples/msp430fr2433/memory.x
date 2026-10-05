/* MSP430FR2433 memory map (SLASE59F Table 6-23, p. 61):
   - RAM: the 4KB of RAM, 2000h to 2FFFh.
   - ROM: the 15KB of main FRAM from C400h, up to the JTAG and BSL signatures at FF80h to FF87h
     (SLASE59F Table 6-2, p. 42).
   - VECTORS: FF88h to FFFFh, from the lowest reserved vector up to the reset vector at FFFEh
     (SLASE59F Table 6-2, p. 41 to p. 42). */

/* Cargo doesn't notice changes to this file. After a change, clean the examples so they are linked
   again: cargo clean -p msp430fr2433-hal-examples --target msp430-none-elf
*/

/* INTERRUPT VECTORS IN RAM:
   For examples/ram_vectors.rs:
   - Change RAM LENGTH to 0xF80
   The interrupt vector table in RAM takes the top 128 bytes of RAM, 2F80h to 2FFFh on the
   MSP430FR2433 (SLAU445I 1.3.6.1, p. 36; RAM: SLASE59F Table 6-23, p. 61), and the stack starts at
   the end of RAM, so RAM must leave them out. The other examples work either way.
*/

MEMORY
{
  RAM : ORIGIN = 0x2000, LENGTH = 0x1000
  ROM : ORIGIN = 0xC400, LENGTH = 0x3B80
  VECTORS : ORIGIN = 0xFF88, LENGTH = 0x78
}
