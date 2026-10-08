//! Cyclic Redundancy Check (CRC).
//!
//! The CRC module produces a signature for a given sequence of data values.
//! The CRC signature is based on the polynomial given in the CRC-CCITT standard: f(x) = x<sup>16</sup> + x<sup>12</sup> + x<sup>5</sup> + 1.
//! See SLAU445I chapter 11 (the polynomial is in 11.1, p. 353 and the registers are in Table 11-1, p. 357).
//!
//! To prepare the CRC module, call [`Crc::new()`] and provide the initial output 'seed' value.
//!
//! # Note
//!
//! The CRC-CCITT standard assumes that bit 0 of each byte is the Most Significant bit (MSb).
//! This runs counter to most microcontroller architectures (including the MSP430), where bit 0 is the Least Significant bit (LSb).
//! To account for this, the MSP430 has bit-reversal hardware which can reverse the order of bits in CRC inputs or outputs. The functions
//! that reverse the bit order are suffixed with `_lsb`, whereas the functions that do not reverse the bit order end in `_msb`.
//! (SLAU445I 11.2, p. 353)
//!
//! Unless you have recieved already bit-reversed values from an external source, or have bit-reversed them yourself, you probably want to use the `_lsb`  
//! insertion functions and the regular result function.
//!

use crate::_pac;

/// Struct representing a Cyclic Redundancy Check (CRC) peripheral initialised with a seed.
pub struct Crc(_pac::Crc);

impl Crc {
    /// Create a new CRC peripheral, setting the initial output to `seed`.
    ///
    /// The generated signature is based on the polynomial given in the CRC-CCITT standard: x<sup>16</sup> + x<sup>12</sup> + x<sup>5</sup> + 1.
    #[inline(always)]
    pub fn new(crc: _pac::Crc, seed: u16) -> Self {
        crc.crcinires().write(|w| unsafe { w.bits(seed) });
        Self(crc)
    }

    /// Insert a byte into the CRC peripheral, assuming that bit 0 is the LSb.
    #[inline(always)]
    pub fn add_byte_lsb(&mut self, byte: u8) {
        // We must write *only* to the lower byte of CRCDIRB
        // The lower byte of CRCDIRB is the 2 + base address of the CRC module
        unsafe { ((_pac::Crc::PTR as *mut u8).add(2)).write_volatile(byte) };
    }

    /// Insert a slice of bytes into the CRC peripheral, assuming that bit 0 is the LSb of each byte. The byte at index 0 is included first.
    #[inline]
    pub fn add_bytes_lsb(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.add_byte_lsb(byte);
        }
    }

    /// Insert a 16-bit word into the CRC peripheral, assuming that bit 0 is the LSb of the lower byte and bit 8 is the LSb of the upper byte.
    ///
    /// The lower byte is included first, followed by the upper byte.
    #[inline(always)]
    pub fn add_word_lsb(&mut self, word: u16) {
        msp430::asm::nop(); // u16 insertions take two cycles (SLAU445I 11.3.1, p. 354), delay to allow back-to-back u16 insertions to finish
        self.0.crcdirb().write(|w| unsafe { w.bits(word) });
    }

    /// Insert a slice of u16's into the CRC peripheral, assuming that bit 0 and bit 8 are the LSbs of each byte.
    ///
    /// The lower byte of each u16 is included first. The u16 at index 0 is included first.  
    #[inline]
    pub fn add_words_lsb(&mut self, words: &[u16]) {
        for &word in words {
            self.add_word_lsb(word);
        }
    }

    /// Insert a byte into the CRC peripheral. This byte is included in the output signature according to the CRC-CCITT standard, which assumes bit 0 is the MSb.
    ///
    /// If your data has bit 0 as the LSb (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline(always)]
    pub fn add_byte_msb(&mut self, byte: u8) {
        // We must write *only* to the lower byte of CRCDI
        // The lower byte of CRCDI is the base address of the CRC module
        unsafe { (_pac::Crc::PTR as *mut u8).write_volatile(byte) };
    }

    /// Insert a slice of bytes into the CRC peripheral. The byte at index 0 is included first.
    ///
    /// These bytes are included in the output signature according to the CRC-CCITT standard, which assumes bit 0 is the MSb of each byte.
    ///
    /// If your data has bit 0 as the LSb (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline]
    pub fn add_bytes_msb(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.add_byte_msb(byte);
        }
    }

    /// Insert a 16-bit word into the CRC peripheral. The lower byte is included first, followed by the upper byte.
    ///
    /// These bytes are included in the output signature according to the CRC-CCITT standard, which assumes bit 0 and bit 8 are the MSbs of each byte.
    ///
    /// If your data has bit 0 and bit 8 as the LSbs (e.g. MSP430 memory locations, variables) use the `_lsb` method instead.
    #[inline(always)]
    pub fn add_word_msb(&mut self, word: u16) {
        msp430::asm::nop(); // u16 insertions take two cycles, delay to allow back-to-back u16 insertions to finish
        self.0.crcdi().write(|w| unsafe { w.bits(word) });
    }

    /// Insert a slice of u16's into the CRC peripheral. The u16 at index 0 is included first. The lower byte of each u16 is included first.
    ///
    /// These bytes are included in the output signature according to the CRC-CCITT standard, which assumes bit 0 is the MSb of each byte.
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
    /// This returns the result according to the CRC-CCITT standard.
    #[inline(always)]
    pub fn result(&mut self) -> u16 {
        msp430::asm::nop(); // u16 insertions take two cycles, delay in case a u16 insertion was just performed.
        self.0.crcinires().read().bits()
    }

    /// Get the computed CRC signature of the data passed in so far. Bit-reverse the result.
    #[inline(always)]
    pub fn result_reversed(&mut self) -> u16 {
        msp430::asm::nop(); // u16 insertions take two cycles, delay in case a u16 insertion was just performed.
        self.0.crcresr().read().bits()
    }

    /// Set the output of the CRC module to the specified seed, effectively resetting the CRC module.
    #[inline(always)]
    pub fn reset(&mut self, seed: u16) {
        self.0.crcinires().write(|w| unsafe { w.bits(seed) });
    }
}
