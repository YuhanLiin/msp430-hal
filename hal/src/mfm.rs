//! Manchester Function Module (MFM)
//!
//! Only on the MSP430FR2x5x (SLASEC4D 1.1, p. 1). The MFM encodes and decodes Manchester-coded packets of
//! 256 data bits (512 bits on the wire, the first data bit 1) on P5.0 (MFM.RX) and P5.1 (MFM.TX), selected by
//! their alternate function 2 (SLAU445I 25.2, p. 666 to p. 667; SLAU445I Figure 25-2, p. 667;
//! SLASEC4D 6.10.14, p. 79; SLASEC4D Table 6-67, p. 104: P5SELx = 10b). It works as the SPI master of
//! eUSCI_B1, which must be a 4-wire SPI slave (SLASEC4D 6.10.14, p. 79): received data arrives in eUSCI_B1's
//! Rx buffer byte by byte, and the data to send is read from its Tx buffer (SLAU445I 25.2, p. 666: "The
//! entire 256-bit data is divided into 32 bytes").
//!
//! - It works in active mode, LPM0 and LPM1 (SLAU445I 25.1, p. 666).
//! - It oversamples 8 times, so the bit rate is SMCLK / 8: 500 kbit/s needs a 4 MHz SMCLK (SLAU445I 25.2,
//!   p. 666; SMCLK clocks the MFM, SLASEC4D Table 6-9, p. 68). SLAU445I 25.3, p. 667 also says "SMCLK at
//!   4 times the target data rate", which contradicts its own 500-kbps and 4-MHz example; this module
//!   follows SLAU445I 25.2, p. 666.
//! - A rising edge of TB2's CCR0 output starts sending a packet, so write the first byte before it. TB2's
//!   capture input CCI0B is the MFM complete event, for timing (SLAU445I 25.2, p. 666; SLAU445I 25.6.1,
//!   p. 668; SLASEC4D Table 6-18, p. 74: "MFM start trigger", "MFM Complete Event").
//! - Each received byte has to be read within about 8 bit times, 16 µs at 500 kbit/s (64 SMCLK cycles at
//!   4 MHz), so use eUSCI_B1's receive interrupt (SLAU445I 25.6.1, p. 668; SLAU445I 25.6.3, p. 669).
//!
//! The user's guide doesn't say which SPI clock mode and STE polarity the MFM uses (SLAU445I 25.6.1, p. 668
//! only names "a standard 4-wire interface"), so this module leaves them to
//! [`SpiConfig`](crate::spi::SpiConfig). It is not tested on hardware yet.

use crate::{
    gpio::{Alternate2, Pin, Pin0, Pin1, P5},
    pac::EUsciB1,
    pin_mapping::{DefaultMapping, PinMap},
    spi::{SpiErr, SpiSlave, SpiUsci},
};
use core::convert::Infallible;

/// The Manchester Function Module, with eUSCI_B1 as its SPI slave (SLASEC4D 6.10.14, p. 79)
pub struct Mfm<M: PinMap = DefaultMapping>
where
    EUsciB1: SpiUsci<M>,
{
    spi: SpiSlave<EUsciB1, M>,
}

impl<M: PinMap> Mfm<M>
where
    EUsciB1: SpiUsci<M>,
{
    /// Enable the MFM by putting its pins in their MFM function (SLAU445I 25.2, p. 666: "To enable the MFM
    /// module, the port selection must be configured properly"; SLASEC4D Table 6-67, p. 104: MFM.RX on P5.0
    /// and MFM.TX on P5.1 at P5SELx = 10b). `spi` is eUSCI_B1 as a 4-wire SPI slave, from
    /// [`SpiConfig::mfm_slave`](crate::spi::SpiConfig::mfm_slave).
    #[inline]
    pub fn new<RXDIR, TXDIR>(
        spi: SpiSlave<EUsciB1, M>,
        _rx: Pin<P5, Pin0, Alternate2<RXDIR>>,
        _tx: Pin<P5, Pin1, Alternate2<TXDIR>>,
    ) -> Self {
        Mfm { spi }
    }

    /// A received byte, or `WouldBlock` if none has arrived (eUSCI_B1's UCRXIFG and UCBxRXBUF:
    /// SLAU445I Table 23-18, p. 624; SLAU445I Table 23-15, p. 623).
    #[inline]
    pub fn read(&mut self) -> nb::Result<u8, SpiErr> { self.spi.read() }

    /// Queue a byte to send, or `WouldBlock` if the Tx buffer is full (eUSCI_B1's UCTXIFG and UCBxTXBUF:
    /// SLAU445I Table 23-18, p. 624; SLAU445I Table 23-16, p. 623).
    #[inline]
    pub fn write(&mut self, byte: u8) -> nb::Result<(), Infallible> { self.spi.write(byte) }

    /// eUSCI_B1, to enable its interrupts or read its interrupt source ("The decoder does not support
    /// interrupt capability", SLAU445I 25.6.3, p. 669).
    #[inline]
    pub fn spi(&mut self) -> &mut SpiSlave<EUsciB1, M> { &mut self.spi }
}
