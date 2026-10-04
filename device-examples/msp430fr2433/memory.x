/* MSP430FR2433 memory map (SLASE59F Table 6-23, p. 61):
   - RAM: the 4KB of RAM, 2000h to 2FFFh.
   - ROM: the 15KB of main FRAM from C400h, up to the JTAG and BSL signatures at FF80h to FF87h
     (SLASE59F Table 6-2, p. 42).
   - VECTORS: FF88h to FFFFh, from the lowest reserved vector up to the reset vector at FFFEh
     (SLASE59F Table 6-2, p. 41 to p. 42). */
MEMORY
{
  RAM : ORIGIN = 0x2000, LENGTH = 0x1000
  ROM : ORIGIN = 0xC400, LENGTH = 0x3B80
  VECTORS : ORIGIN = 0xFF88, LENGTH = 0x78
}
