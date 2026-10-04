/* DEVICE SELECTION:
   To use MSP430FR2475: 
   - Change RAM LENGTH to 0x1800
   - Change ROM LENGTH to 0x8000
   (SLASEO7C Table 9-31, p. 73: the MSP430FR2475 has 6KB of RAM, 2000h to 37FFh, and 32KB of
   FRAM, 8000h to FFFFh)
*/

MEMORY
{
  /* Current Values for MSP430FR2476 (SLASEO7C Table 9-31, p. 73): 8KB of RAM, 2000h to 3FFFh, and
     64KB of FRAM, 8000h to 17FFFh, which holds the interrupt vectors and signatures at FF80h to
     FFFFh (also SLASEO7C 9.4, p. 46: "The interrupt vectors and the power-up start address are in
     the address range 0FFFFh to 0FF80h") */
  RAM     : ORIGIN = 0x2000, LENGTH = 0x2000
  ROM     : ORIGIN = 0x8000, LENGTH = 0x10000  
  VECTORS : ORIGIN = 0xFF80, LENGTH = 0x80
}