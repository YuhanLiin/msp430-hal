/* MSP430FR2355 memory map: SLASEC4D 6.4, Table 6-4, p. 65 */
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
