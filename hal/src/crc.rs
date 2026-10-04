//! Cyclic Redundancy Check (CRC).
//!
//! The CRC module produces a signature for a given sequence of data values.
//! The CRC signature is based on the polynomial given in the CRC-CCITT standard: f(x) = x<sup>16</sup> + x<sup>12</sup> + x<sup>5</sup> + 1.
//! (SLAU445I 11.1, p. 353: Equation 12)
//!
//! To prepare the CRC module, call [`Crc::new()`] and provide the initial output 'seed' value (SLAU445I 11.3,
//! p. 354).
//!
//! # Note
//!
//! The CRC-CCITT standard assumes that bit 0 of each byte is the Most Significant bit (MSb).
//! This runs counter to most microcontroller architectures (including the MSP430), where bit 0 is the Least Significant bit (LSb).
//! To account for this, the MSP430 has bit-reversal hardware which can reverse the order of bits in CRC inputs or outputs. The functions
//! that reverse the bit order are suffixed with `_lsb`, whereas the functions that do not reverse the bit order end in `_msb`.
//! (SLAU445I 11.2, p. 353; CRCDIRB and CRCRESR: SLAU445I 11.3.1, p. 354, and SLAU445I Table 11-5, p. 359)
//!
//! Unless you have recieved already bit-reversed values from an external source, or have bit-reversed them yourself, you probably want to use the `_lsb`  
//! insertion functions and the regular result function.
//! (SLAU445I Example 11-2, p. 356: written to CRCDIRB, the bytes of "123456789" give 029B1h in CRCINIRES)
//!

use crate::_pac;

/// Struct representing a Cyclic Redundancy Check (CRC) peripheral initialised with a seed (SLAU445I 11.3,
/// p. 354; the CRC registers: SLAU445I Table 11-1, p. 357).
pub struct Crc(_pac::Crc);

impl Crc {
    /// Create a new CRC peripheral, setting the initial output to `seed` (CRCINIRES: SLAU445I Table 11-4,
    /// p. 359).
    ///
    /// The generated signature is based on the polynomial given in the CRC-CCITT standard: x<sup>16</sup> + x<sup>12</sup> + x<sup>5</sup> + 1.
    /// (SLAU445I 11.1, p. 353: Equation 12)
    #[inline(always)]
    pub fn new(crc: _pac::Crc, seed: u16) -> Self {
        crc.crcinires().write(|w| w.crcinires().set(seed));
        Self(crc)
    }

    /// Insert a byte into the CRC peripheral, assuming that bit 0 is the LSb (CRCDIRB reverses the bits of
    /// each byte: SLAU445I 11.3.1, p. 354).
    #[inline(always)]
    pub fn add_byte_lsb(&mut self, byte: u8) {
        // A byte write to the lower byte of CRCDIRB adds one byte; a word write would add two
        // (CRCDIRB_L at offset 02h: SLAU445I Table 11-1, p. 357; byte writes to CRCDIRB_L as in SLAU445I
        // Example 11-2, p. 356)
        self.0.crcdirb_l().write(|w| w.crcdirb().set(byte));
    }

    /// Insert a slice of bytes into the CRC peripheral, assuming that bit 0 is the LSb of each byte. The byte at index 0 is included first.
    /// Each byte goes through CRCDIRB, which reverses its bits (SLAU445I 11.3.1, p. 354).
    #[inline]
    pub fn add_bytes_lsb(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.add_byte_lsb(byte);
        }
    }

    /// Insert a 16-bit word into the CRC peripheral, assuming that bit 0 is the LSb of the lower byte and bit 8 is the LSb of the upper byte.
    ///
    /// The lower byte is included first, followed by the upper byte. (CRCDIRB: SLAU445I 11.3.1, p. 354, and
    /// SLAU445I Table 11-3, p. 358)
    #[inline(always)]
    pub fn add_word_lsb(&mut self, word: u16) {
        // (SLAU445I 11.3.1, p. 354: "it takes two clock cycles to process word data")
        msp430::asm::nop(); // u16 insertions take two cycles, delay to allow back-to-back u16 insertions to finish
        self.0.crcdirb().write(|w| w.crcdirb().set(word));
    }

    /// Insert a slice of u16's into the CRC peripheral, assuming that bit 0 and bit 8 are the LSbs of each byte.
    ///
    /// The lower byte of each u16 is included first. The u16 at index 0 is included first.
    /// (SLAU445I 11.3.1, p. 354)
    #[inline]
    pub fn add_words_lsb(&mut self, words: &[u16]) {
        for &word in words {
            self.add_word_lsb(word);
        }
    }

    /// Insert a byte into the CRC peripheral. This byte is included in the output signature according to the CRC-CCITT standard, which assumes bit 0 is the MSb.
    /// (CRCDI: SLAU445I Table 11-2, p. 358; not bit reversed: SLAU445I 11.3.1, p. 354)
    ///
    /// If your data has bit 0 as the LSb (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline(always)]
    pub fn add_byte_msb(&mut self, byte: u8) {
        // A byte write to the lower byte of CRCDI adds one byte; a word write would add two (CRCDI_L at
        // offset 00h: SLAU445I Table 11-1, p. 357)
        self.0.crcdi_l().write(|w| w.crcdi().set(byte));
    }

    /// Insert a slice of bytes into the CRC peripheral. The byte at index 0 is included first.
    ///
    /// These bytes are included in the output signature according to the CRC-CCITT standard, which assumes bit 0 is the MSb of each byte.
    /// Each byte goes through CRCDI, which does not reverse its bits (SLAU445I 11.2, p. 353; SLAU445I 11.3.1,
    /// p. 354).
    ///
    /// If your data has bit 0 as the LSb (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline]
    pub fn add_bytes_msb(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.add_byte_msb(byte);
        }
    }

    /// Insert a 16-bit word into the CRC peripheral. The lower byte is included first, followed by the upper byte.
    /// (SLAU445I 11.3.1, p. 354)
    ///
    /// These bytes are included in the output signature according to the CRC-CCITT standard, which assumes bit 0 and bit 8 are the MSbs of each byte.
    /// (CRCDI: SLAU445I Table 11-2, p. 358)
    ///
    /// If your data has bit 0 and bit 8 as the LSbs (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline(always)]
    pub fn add_word_msb(&mut self, word: u16) {
        // (SLAU445I 11.3.1, p. 354: "it takes two clock cycles to process word data")
        msp430::asm::nop(); // u16 insertions take two cycles, delay to allow back-to-back u16 insertions to finish
        self.0.crcdi().write(|w| w.crcdi().set(word));
    }

    /// Insert a slice of u16's into the CRC peripheral. The u16 at index 0 is included first. The lower byte of each u16 is included first.
    /// (SLAU445I 11.3.1, p. 354)
    ///
    /// These bytes are included in the output signature according to the CRC-CCITT standard, which assumes bit 0 is the MSb of each byte.
    /// (CRCDI: SLAU445I Table 11-2, p. 358)
    ///
    /// If your data has bit 0 as the LSb (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline]
    pub fn add_words_msb(&mut self, words: &[u16]) {
        for &word in words {
            self.add_word_msb(word);
        }
    }

    /// Get the computed CRC signature of the data passed in so far.
    ///
    /// This returns the result according to the CRC-CCITT standard (CRCINIRES: SLAU445I Table 11-4, p. 359).
    #[inline(always)]
    pub fn result(&mut self) -> u16 {
        // (SLAU445I 11.3.1, p. 354: "it takes two clock cycles to process word data")
        msp430::asm::nop(); // u16 insertions take two cycles, delay in case a u16 insertion was just performed.
        self.0.crcinires().read().bits()
    }

    /// Get the computed CRC signature of the data passed in so far. Bit-reverse the result (CRCRESR: SLAU445I
    /// Table 11-5, p. 359).
    #[inline(always)]
    pub fn result_reversed(&mut self) -> u16 {
        // (SLAU445I 11.3.1, p. 354: "it takes two clock cycles to process word data")
        msp430::asm::nop(); // u16 insertions take two cycles, delay in case a u16 insertion was just performed.
        self.0.crcresr().read().bits()
    }

    /// Set the output of the CRC module to the specified seed, effectively resetting the CRC module
    /// (CRCINIRES: SLAU445I Table 11-4, p. 359: "Writing to this register initializes the CRC calculation").
    #[inline(always)]
    pub fn reset(&mut self, seed: u16) {
        self.0.crcinires().write(|w| w.crcinires().set(seed));
    }
}
