//! Serial UART
//!
//! The eUSCI_A peripherals can be used as serial UARTs (SLAU445I 22.1, p. 575). Pins used (pins with
//! `RemappedMapping` in brackets):
//!
//! | Device       | eUSCI | TXD             | RXD             | External clock  |
//! |:-------------|:-----:|:---------------:|:---------------:|:---------------:|
//! | MSP430FR2x5x | A0    | `P1.7`          | `P1.6`          | `P1.5`          |
//! | MSP430FR2x5x | A1    | `P4.3`          | `P4.2`          | `P4.1`          |
//! | MSP430FR2433 | A0    | `P1.4`          | `P1.5`          | `P1.6`          |
//! | MSP430FR2433 | A1    | `P2.6`          | `P2.5`          | `P2.4`          |
//! | MSP430FR247x | A0    | `P1.4` (`P5.2`) | `P1.5` (`P5.1`) | `P1.6` (`P5.0`) |
//! | MSP430FR247x | A1    | `P2.6`          | `P2.5`          | `P2.4`          |
//! | MSP430FR25x2 | A0    | `P1.4` (`P2.0`) | `P1.5` (`P2.1`) | `P1.6`          |
//!
//! - MSP430FR2x5x: SLASEC4D Table 6-14, p. 72; port pin functions, including UCA0CLK and UCA1CLK:
//!   SLASEC4D Table 6-63, p. 96 (P1) and SLASEC4D Table 6-66, p. 102 (P4).
//! - MSP430FR2433: SLASE59F Table 6-10, p. 49; port pin functions, including UCA0CLK and UCA1CLK: SLASE59F
//!   Table 6-17, p. 55 (P1) and SLASE59F Table 6-19, p. 58 (P2).
//! - MSP430FR247x: SLASEO7C Table 9-11, p. 54; port pin functions, including UCA0CLK and UCA1CLK: SLASEO7C
//!   Table 9-23, p. 65 (P1), SLASEO7C Table 9-24, p. 66 (P2) and SLASEO7C Table 9-27, p. 69 (P5).
//! - MSP430FR25x2: SLASEE4C Table 6-11, p. 53; port pin functions, including UCA0CLK: SLASEE4C Table 6-15,
//!   p. 58 (P1) and SLASEE4C Table 6-16, p. 60 (P2).
//!
//! On the MSP430FR2433 the PAC exposes each eUSCI once per mode, for example `usci_a0_uart_mode` and
//! `usci_a0_spi_mode`. Both are the same hardware, so only use one of them for each eUSCI.
//!
//! On the MSP430FR2x5x, eUSCI_A1's pins in alternate function 2 (`to_alternate2()`) invert the polarity of
//! TXD and RXD, and a rising edge then starts a character (SLASEC4D 6.10.8, p. 73; SLASEC4D Table 6-15,
//! p. 73, "eUSCI_A1 UART Polarity Configurations"; SLASEC4D Table 6-66, p. 102: P4SELx = 10b). SLASEC4D
//! Table 6-15, p. 73 names P4.4 for RXD, but UCA1RXD is on P4.2 (SLASEC4D Table 6-14, p. 72 and SLASEC4D
//! Table 6-66, p. 102); the HAL follows the pin function table, where P4.4 is UCB1STE.
//!
//! Begin configuration by calling [`SerialConfig::new()`]. After configuration, [`Rx`] and/or [`Tx`] structs are produced by
//! providing the corresponding GPIO pins.
//!
//! The [`Tx`] and [`Rx`] structs are used to send and receive bytes via serial. They implement both [`embedded-io`](embedded_io)'s
//! serial traits (which are buffer-based, blocking), and the single-byte-based non-blocking [`embedded-hal-nb`](embedded_hal_nb::serial) version.
//!
//! As the MSP430 has only a single byte buffer (UCAxTXBUF and UCAxRXBUF, SLAU445I 22.4.6, p. 597 and
//! SLAU445I 22.4.7, p. 597), `embedded-io`'s buffer-based traits can be a bit unwieldy -
//! [`emb_io::Write::write`](embedded_io::Write::write) and [`emb_io::Read::read`](embedded_io::Read::read) will
//! always only send or recieve a single byte, despite taking slices as inputs.
//!
//! For reading or writing single bytes it is recommended to use `embedded-hal-nb`'s
//! [`emb_hal_nb::Read::read`](embedded_hal_nb::serial::Read::read) and
//! [`emb_hal_nb::Write::write`](embedded_hal_nb::serial::Write::write) ([`nb::block`] can be used to make them blocking).
//!
//! For writing multiple bytes, embedded_io's [`Write::write_all`](embedded_io::Write::write_all) and
//! [`Read::read_exact`](embedded_io::Read::read_exact) methods are useful.
//!
//! Besides plain UART, [`SerialConfig::mode`] selects the multiprocessor formats, which mark address characters
//! ([`Tx::send_address`], [`Rx::set_dormant`]; SLAU445I 22.3.3, p. 577 to p. 579), and automatic baud-rate
//! detection from a LIN break and synch field (SLAU445I 22.3.4, p. 580). [`SerialConfig::deglitch`] sets how
//! short a pulse on RXD is ignored (SLAU445I 22.3.7.1, p. 583).
//!

#[cfg(feature = "eusci_aclk")]
use crate::clock::Aclk;
use crate::clock::{Clock, Smclk};
use crate::hw_traits::eusci::{EUsciUart, UartUcxStatw, UcaCtlw0, Ucssel};
use crate::pin_mapping::*;
use core::convert::Infallible;
use core::fmt::Display;
use core::marker::PhantomData;
use core::num::NonZeroU32;

/// Bit order of transmit and receive (UCMSB, SLAU445I Table 22-8, p. 593)
#[derive(Clone, Copy)]
pub enum BitOrder {
    /// LSB first (typically the default; SLAU445I 22.3.2, p. 577: "LSB first is typically required for UART
    /// communication")
    LsbFirst,
    /// MSB first
    MsbFirst,
}

impl BitOrder {
    // UCMSB: 0 = LSB first, 1 = MSB first (SLAU445I Table 22-8, p. 593)
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            BitOrder::LsbFirst => false,
            BitOrder::MsbFirst => true,
        }
    }
}

/// Number of bits per transaction (UC7BIT, SLAU445I Table 22-8, p. 593)
#[derive(Clone, Copy)]
pub enum BitCount {
    /// 8 bits
    EightBits,
    /// 7 bits
    SevenBits,
}

impl BitCount {
    // UC7BIT: 0 = 8-bit data, 1 = 7-bit data (SLAU445I Table 22-8, p. 593)
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            BitCount::EightBits => false,
            BitCount::SevenBits => true,
        }
    }
}

/// Number of stop bits at end of each byte (UCSPB, SLAU445I Table 22-8, p. 593)
#[derive(Clone, Copy)]
pub enum StopBits {
    /// 1 stop bit
    OneStopBit,
    /// 2 stop bits
    TwoStopBits,
}

impl StopBits {
    // UCSPB: 0 = one stop bit, 1 = two stop bits (SLAU445I Table 22-8, p. 593)
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            StopBits::OneStopBit => false,
            StopBits::TwoStopBits => true,
        }
    }
}

/// Parity bit for error checking (UCPEN and UCPAR, SLAU445I Table 22-8, p. 593)
#[derive(Clone, Copy)]
pub enum Parity {
    /// No parity
    NoParity,
    /// Odd parity
    OddParity,
    /// Even parity
    EvenParity,
}

impl Parity {
    // UCPEN: 1 = parity enabled; UCPAR: 0 = odd, 1 = even (SLAU445I Table 22-8, p. 593)
    #[inline(always)]
    fn ucpen(self) -> bool {
        match self {
            Parity::NoParity => false,
            _ => true,
        }
    }

    #[inline(always)]
    fn ucpar(self) -> bool {
        match self {
            Parity::OddParity => false,
            Parity::EvenParity => true,
            _ => false,
        }
    }
}

/// Loopback settings (UCLISTEN, SLAU445I Table 22-12, p. 596)
#[derive(Clone, Copy)]
pub enum Loopback {
    /// No loopback
    NoLoop,
    /// Tx feeds into Rx (UCLISTEN, SLAU445I Table 22-12, p. 596)
    Loopback,
}

impl Loopback {
    // UCLISTEN: 1 = UCAxTXD fed back to the receiver (SLAU445I Table 22-12, p. 596)
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            Loopback::NoLoop => false,
            Loopback::Loopback => true,
        }
    }
}

/// How short a pulse on RXD the receiver ignores (UCGLIT). The "about" times are the user's guide's
/// approximate ones (SLAU445I Table 22-9, p. 594); the data sheets give typical deglitch times tt of 12, 40,
/// 68 and 110 ns instead (SLASEC4D Table 5-15, p. 45; SLASE59F Table 5-15, p. 30; SLASEO7C 8.12.7.2, p. 35;
/// SLASEE4C Table 5-15, p. 32). The variant names follow the user's guide's register table.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum UartDeglitch {
    /// About 2 ns (tt typically 12 ns)
    _2ns = 0,
    /// About 50 ns (tt typically 40 ns)
    _50ns = 1,
    /// About 100 ns (tt typically 68 ns)
    _100ns = 2,
    /// About 200 ns, as after reset (tt typically 110 ns)
    #[default]
    _200ns = 3,
}

/// The length of the break delimiter sent before the synch field in automatic baud-rate mode (UCDELIM,
/// SLAU445I 22.3.4.1, p. 581 and SLAU445I Table 22-15, p. 598)
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum BreakDelimiter {
    /// 1 bit time
    #[default]
    _1Bit = 0,
    /// 2 bit times
    _2Bits = 1,
    /// 3 bit times
    _3Bits = 2,
    /// 4 bit times
    _4Bits = 3,
}

/// The UART mode (UCMODE, SLAU445I Table 22-8, p. 593; SLAU445I 22.3.3, p. 577; SLAU445I 22.3.4, p. 580)
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum UartMode {
    /// Plain UART
    #[default]
    Uart,
    /// Idle-line multiprocessor format: the first character after an idle line of 10 or more bits is an
    /// address (SLAU445I 22.3.3.1, p. 577)
    IdleLineMultiprocessor,
    /// Address-bit multiprocessor format: each character has an extra bit that marks addresses (SLAU445I
    /// 22.3.3.2, p. 579)
    AddressBitMultiprocessor,
    /// Automatic baud-rate detection, as in LIN: a received break and synch field (0x55) set the baud rate.
    /// For LIN, use 8 data bits, LSB first, no parity and one stop bit. The receiver measures with the
    /// transmitter's baud-rate generator, so it can't measure a break and synch field it sends itself, in
    /// loopback for example (SLAU445I 22.3.4, p. 580: "The transmit baud-rate generator is used for the
    /// measurement", "The eUSCI_A cannot transmit data while receiving the break/sync field").
    AutoBaud {
        /// The length of the break delimiter [`Tx::send_break`] sends
        delimiter: BreakDelimiter,
    },
}

impl UartMode {
    // UCMODEx (SLAU445I Table 22-8, p. 593)
    #[inline(always)]
    fn ucmode(self) -> u8 {
        match self {
            UartMode::Uart => 0b00,
            UartMode::IdleLineMultiprocessor => 0b01,
            UartMode::AddressBitMultiprocessor => 0b10,
            UartMode::AutoBaud { .. } => 0b11,
        }
    }
}

/// The highest-priority pending UART interrupt among the enabled ones, in the order of UCAxIV, as
/// returned by `interrupt_source()` (SLAU445I 22.3.15.4, p. 591 and SLAU445I Table 22-19, p. 602)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UartVector {
    /// No interrupt pending
    None,
    /// A character was received (UCRXIFG). Reading it clears the flag (SLAU445I 22.3.15.2, p. 591:
    /// "UCRXIFG is automatically reset when UCAxRXBUF is read").
    RxBufFull,
    /// The Tx buffer is empty (UCTXIFG). Writing to it clears the flag (SLAU445I 22.3.15.1, p. 590:
    /// "UCTXIFG is automatically reset if a character is written to UCAxTXBUF"), so the interrupt keeps
    /// firing until a character is written or Tx interrupts are disabled.
    TxBufEmpty,
    /// A start bit was received (UCSTTIFG, SLAU445I Table 22-6, p. 591)
    StartBit,
    /// A character was sent completely (UCTXCPTIFG, SLAU445I Table 22-6, p. 591)
    TxComplete,
}

// The highest-priority pending interrupt among the enabled ones, in the priority order of UCAxIV:
// UCRXIFG, UCTXIFG, UCSTTIFG, UCTXCPTIFG (SLAU445I Table 22-19, p. 602; "Disabled interrupts do not affect
// the UCAxIV value", SLAU445I 22.3.15.4, p. 591). It's worked out from UCAxIFG and UCAxIE (SLAU445I
// Table 22-18, p. 601; SLAU445I Table 22-17, p. 600) instead of read from UCAxIV, because a UCAxIV read
// "automatically resets the highest-pending Interrupt condition and flag" (SLAU445I 22.3.15.4, p. 591):
// UCRXIFG and UCTXIFG would then be clear, and read() and write(), which test them, would block. Reading
// UCAxRXBUF and writing UCAxTXBUF clear those two flags (SLAU445I 22.3.15.2, p. 591; SLAU445I 22.3.15.1,
// p. 590). UCSTTIFG and UCTXCPTIFG have no such access, so they're cleared here, as a UCAxIV read would.
#[inline(always)]
fn uart_vector<USCI: EUsciUart>(usci: &USCI) -> UartVector {
    let pending = usci.pending_rd();
    if pending.rx {
        UartVector::RxBufFull
    } else if pending.tx {
        UartVector::TxBufEmpty
    } else if pending.start_bit {
        usci.sttifg_clear();
        UartVector::StartBit
    } else if pending.tx_complete {
        usci.txcptifg_clear();
        UartVector::TxComplete
    } else {
        UartVector::None
    }
}

/// Marks a USCI type that can be used as a serial UART
pub trait SerialUsci<M: PinMap = DefaultMapping>: EUsciUart {
    /// Pin used for serial UCLK (the UCAxCLK pin, the clock source with UCSSELx = 00b, SLAU445I Table 22-8,
    /// p. 593)
    type ClockPin;
    /// Pin used for Tx (UCAxTXD, SLAU445I 22.2, p. 575)
    type TxPin;
    /// Pin used for Rx (UCAxRXD, SLAU445I 22.2, p. 575)
    type RxPin;

    /// Additional configuration, such as the eUSCI_A0 remapping bit USCIA0RMP in SYSCFG3 (SLAU445I
    /// Table 1-32, p. 83)
    #[inline(always)]
    fn configure_pin_mapping() {}
}

// The pin's alternate function defaults to Alternate1 (PxSEL = 01b, the UCAxTXD, UCAxRXD and UCAxCLK
// function in the port pin function tables, for example SLASEC4D Table 6-63, p. 96; SLASE59F Table 6-17,
// p. 55; SLASEO7C Table 9-23, p. 65; SLASEE4C Table 6-15, p. 58)
macro_rules! impl_serial_pin {
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl_serial_pin!($struct_name, $port, $pin, Alternate1);
    };
    ($struct_name: ident, $port: ty, $pin: ty, $alt: ident) => {
        impl<DIR> From<Pin<$port, $pin, $alt<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, $alt<DIR>>) -> Self { $struct_name }
        }
    };
}
pub(crate) use impl_serial_pin;

/// Typestate for a serial interface with an unspecified clock source
pub struct NoClockSet {
    baudrate: NonZeroU32,
}

/// Typestate for a serial interface with a specified clock source
pub struct ClockSet {
    baud_config: BaudConfig,
    clksel: Ucssel,
}

/// Builder object for configuring a serial UART
///
/// Once the clock source has been selected, the builder can be converted into pins that can
/// transmit or received bytes via a serial connection.
pub struct SerialConfig<USCI, S, M: PinMap = DefaultMapping>
where USCI: SerialUsci<M>
{
    usci: USCI,
    order: BitOrder,
    cnt: BitCount,
    stopbits: StopBits,
    parity: Parity,
    loopback: Loopback,
    mode: UartMode,
    deglitch: UartDeglitch,
    break_interrupts: bool,
    state: S,
    _map: PhantomData<M>,
}

macro_rules! serial_config {
    ($conf:expr, $state:expr) => {
        SerialConfig::<_, _, _> {
            usci: $conf.usci,
            order: $conf.order,
            cnt: $conf.cnt,
            stopbits: $conf.stopbits,
            parity: $conf.parity,
            loopback: $conf.loopback,
            mode: $conf.mode,
            deglitch: $conf.deglitch,
            break_interrupts: $conf.break_interrupts,
            state: $state,
            _map: core::marker::PhantomData,
        }
    };
}

impl<USCI, S, M> SerialConfig<USCI, S, M>
where
    USCI: SerialUsci<M>,
    M: PinMap,
{
    /// Select the UART mode: plain UART, a multiprocessor format or automatic baud-rate detection (UCMODEx,
    /// SLAU445I Table 22-8, p. 593).
    #[inline]
    pub fn mode(mut self, mode: UartMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set how short a pulse on RXD the receiver ignores. After reset that's about 200 ns (UCGLITx = 3,
    /// SLAU445I Table 22-9, p. 594; see [`UartDeglitch`] for the data sheets' times).
    #[inline]
    pub fn deglitch(mut self, deglitch: UartDeglitch) -> Self {
        self.deglitch = deglitch;
        self
    }

    /// Report received breaks: a break then reads as [`RecvError::Break`] (UCBRKIE, SLAU445I Table 22-8,
    /// p. 593). In automatic baud-rate mode, the break and synch field are reported that way (SLAU445I
    /// 22.3.4, p. 580: "If the UCBRKIE bit is set, reception of the break/synch sets the UCRXIFG").
    #[inline]
    pub fn break_interrupts(mut self) -> Self {
        self.break_interrupts = true;
        self
    }
}

impl<USCI, M> SerialConfig<USCI, NoClockSet, M>
where
    USCI: SerialUsci<M>,
    M: PinMap,
{
    /// Create a new serial configuration using a EUSCI peripheral
    #[inline]
    pub fn new(
        usci: USCI,
        order: BitOrder,
        cnt: BitCount,
        stopbits: StopBits,
        parity: Parity,
        loopback: Loopback,
        baudrate: u32,
    ) -> Self {
        const ONE: NonZeroU32 = NonZeroU32::new(1).unwrap();
        SerialConfig {
            order,
            cnt,
            stopbits,
            parity,
            loopback,
            usci,
            // The reset values: UCMODEx = 00b and UCGLITx = 11b (SLAU445I Table 22-8, p. 593; SLAU445I
            // Table 22-9, p. 594)
            mode: UartMode::Uart,
            deglitch: UartDeglitch::_200ns,
            break_interrupts: false,
            state: NoClockSet { baudrate: NonZeroU32::new(baudrate).unwrap_or(ONE) },
            _map: PhantomData,
        }
    }

    /// Configure serial UART to use external UCLK, passing in the appropriately configured pin
    /// used as the clock signal as well as the frequency of the clock (UCSSELx = 00b, SLAU445I Table 22-8,
    /// p. 593).
    ///
    /// # Panics
    ///
    /// If the baud rate is above a third of the clock frequency, the most the eUSCI supports (SLAU445I
    /// 22.3.9.1, p. 584).
    #[inline(always)]
    pub fn use_uclk<P: Into<USCI::ClockPin>>(
        self,
        _clk_pin: P,
        freq: u32,
    ) -> SerialConfig<USCI, ClockSet, M> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(freq, self.state.baudrate),
                clksel: Ucssel::Uclk,
            }
        )
    }

    #[cfg(feature = "eusci_aclk")]
    /// Configure serial UART to use ACLK (UCSSELx = 01b on these devices: SLASEC4D Table 6-9, p. 68;
    /// SLASEO7C Table 9-8, p. 50; SLASEE4C Table 6-8, p. 49).
    ///
    /// # Panics
    ///
    /// If the baud rate is above a third of the clock frequency, the most the eUSCI supports (SLAU445I
    /// 22.3.9.1, p. 584).
    #[inline(always)]
    pub fn use_aclk(self, aclk: &Aclk) -> SerialConfig<USCI, ClockSet, M> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(aclk.freq() as u32, self.state.baudrate),
                clksel: Ucssel::DeviceSpecific,
            }
        )
    }

    #[cfg(feature = "eusci_modclk")]
    /// Configure serial UART to use MODCLK (UCSSELx = 01b on the MSP430FR2433, SLASE59F Table 6-7, p. 46).
    ///
    /// # Panics
    ///
    /// If the baud rate is above a third of the clock frequency, the most the eUSCI supports (SLAU445I
    /// 22.3.9.1, p. 584).
    #[inline(always)]
    pub fn use_modclk(self) -> SerialConfig<USCI, ClockSet, M> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(
                    crate::device_specific::MODCLK_FREQ_HZ,
                    self.state.baudrate
                ),
                clksel: Ucssel::DeviceSpecific,
            }
        )
    }

    /// Configure serial UART to use SMCLK (UCSSELx = 10b, SLAU445I Table 22-8, p. 593).
    ///
    /// # Panics
    ///
    /// If the baud rate is above a third of the clock frequency, the most the eUSCI supports (SLAU445I
    /// 22.3.9.1, p. 584).
    #[inline(always)]
    pub fn use_smclk(self, smclk: &Smclk) -> SerialConfig<USCI, ClockSet, M> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(smclk.freq(), self.state.baudrate),
                clksel: Ucssel::Smclk,
            }
        )
    }
}

struct BaudConfig {
    br: u16,
    brs: u8,
    brf: u8,
    ucos16: bool,
}

#[inline]
fn calculate_baud_config(clk_freq: u32, bps: NonZeroU32) -> BaudConfig {
    // In low-frequency mode the baud rate can be at most a third of the clock (SLAU445I 22.3.9.1,
    // p. 584)
    assert!(clk_freq / bps.get() >= 3, "baud rate above a third of the UART clock");
    // Ensure n stays within the 16 bit boundary (UCBRx, SLAU445I Table 22-10, p. 595)
    let n = (clk_freq / bps).min(0xFFFF);

    let brs = lookup_brs(clk_freq, bps);

    // N = fBRCLK / baud rate. Oversampling (UCOS16 = 1) when N >= 16, with UCBRx = INT(N / 16) and
    // UCBRFx = INT((N / 16 - INT(N / 16)) * 16); otherwise UCBRx = INT(N) (SLAU445I 22.3.10, p. 586: "If N
    // is equal or greater than 16, it is recommended to use the oversampling baud-rate generation mode";
    // SLAU445I 22.3.10.1, p. 586; SLAU445I 22.3.10.2, p. 587). The quick set-up note on the same page says
    // "if N > 16" instead; this follows the text, so N = 16 also oversamples, within the 1/16 limit of
    // SLAU445I 22.3.9.2, p. 585.
    if (n >= 16) && (bps.get() < u32::MAX / 16) {
        //  div = bps * 16
        const SIXTEEN: NonZeroU32 = NonZeroU32::new(16).unwrap();
        let div = bps.saturating_mul(SIXTEEN);

        // n / 16, but more precise
        let br = (clk_freq / div) as u16;

        // same as n % 16, but more precise
        let brf = ((clk_freq % div) / bps) as u8;
        BaudConfig { ucos16: true, br, brf, brs }
    } else {
        // UCBRFx is ignored with UCOS16 = 0 (SLAU445I Table 22-11, p. 595)
        BaudConfig { ucos16: false, br: n as u16, brf: 0, brs }
    }
}

#[inline(always)]
fn lookup_brs(clk_freq: u32, bps: NonZeroU32) -> u8 {
    // bps is between [1, 5_000_000] (datasheet max: fBITCLK 5 MHz in SLASEC4D Table 5-14, p. 45,
    // SLASE59F Table 5-14, p. 30, SLASEO7C 8.12.7.1, p. 35 and SLASEE4C Table 5-14, p. 32)
    // clk_freq is between [0, 24_000_000] (datasheet max: feUSCI 24 MHz on the MSP430FR2x5x, SLASEC4D
    // Table 5-14, p. 45; 16 MHz on the others, SLASE59F Table 5-14, p. 30, SLASEO7C 8.12.7.1, p. 35 and
    // SLASEE4C Table 5-14, p. 32)

    // modulo = clk_freq % bps => modulo is between [0, 4_999_999]
    let modulo = clk_freq % bps;

    // fraction = modulo * 10_000 / (bps), so within [0, ((bps-1) * 10_000) / bps].
    // To prove upper bound we note `(bps-1)/bps` is largest when bps == 5_000_000:
    // (4_999_999 * 10_000) / 5_000_000 = 49_999_990_000 (watch out for overflow!) / 5_000_000 = 9999.99... truncated to 9_999 because integer division
    // So fraction is within [0, 9999]
    let fraction_as_ten_thousandths = if modulo < u32::MAX / 10_000 {
        // Most accurate
        ((modulo * 10_000) / bps) as u16
    } else {
        // Avoid overflow if modulo is large. Assume modulo < 5_000_000 from datasheet max (fBITCLK, above)
        (((modulo * 500) / bps) * 20) as u16
    };

    // See SLAU445I Table 22-4, p. 586, in the MSP430FR4xx and MSP430FR2xx family user's guide (Rev. I).
    // A row's UCBRSx is valid from its fractional portion up to the next row's.
    match fraction_as_ten_thousandths {
        0..529     => 0x00,
        529..715   => 0x01,
        715..835   => 0x02,
        835..1001  => 0x04,
        1001..1252 => 0x08,
        1252..1430 => 0x10,
        1430..1670 => 0x20,
        1670..2147 => 0x11,
        2147..2224 => 0x21,
        2224..2503 => 0x22,
        2503..3000 => 0x44,
        3000..3335 => 0x25,
        3335..3575 => 0x49,
        3575..3753 => 0x4A,
        3753..4003 => 0x52,
        4003..4286 => 0x92,
        4286..4378 => 0x53,
        4378..5002 => 0x55,
        5002..5715 => 0xAA,
        5715..6003 => 0x6B,
        6003..6254 => 0xAD,
        6254..6432 => 0xB5,
        6432..6667 => 0xB6,
        6667..7001 => 0xD6,
        7001..7147 => 0xB7,
        7147..7503 => 0xBB,
        7503..7861 => 0xDD,
        7861..8004 => 0xED,
        8004..8333 => 0xEE,
        8333..8464 => 0xBF,
        8464..8572 => 0xDF,
        8572..8751 => 0xEF,
        8751..9004 => 0xF7,
        9004..9170 => 0xFB,
        9170..9288 => 0xFD,
        9288..     => 0xFE,
    }
}

impl<USCI, M> SerialConfig<USCI, ClockSet, M>
where
    USCI: SerialUsci<M>,
    M: PinMap,
{
    #[inline]
    fn config_hw(self) {
        let ClockSet { baud_config, clksel } = self.state;
        let usci = self.usci;

        // Set UCSWRST, then initialize the registers (SLAU445I 22.3.1, p. 577, steps 1 and 2)
        usci.ctl0_reset();
        // UCAxBRW holds UCBRx; UCAxMCTLW holds UCBRSx, UCBRFx and UCOS16 (SLAU445I Table 22-10, p. 595;
        // SLAU445I Table 22-11, p. 595)
        usci.brw_settings(baud_config.br);
        usci.mctlw_settings(baud_config.ucos16, baud_config.brs, baud_config.brf);
        // UCLISTEN in UCAxSTATW (SLAU445I Table 22-12, p. 596)
        usci.loopback(self.loopback.to_bool());
        // UCGLITx in UCAxCTLW1 (SLAU445I Table 22-9, p. 594)
        usci.ctl1_settings(self.deglitch as u8);
        // UCABDEN and UCDELIMx in UCAxABCTL (SLAU445I Table 22-15, p. 598)
        match self.mode {
            UartMode::AutoBaud { delimiter } => usci.abctl_settings(true, delimiter as u8),
            _ => usci.abctl_settings(false, 0),
        }
        // UCAxCTLW0, with UCSWRST still set (SLAU445I Table 22-8, p. 593 to p. 594)
        usci.ctl0_settings(UcaCtlw0 {
            ucpen: self.parity.ucpen(),
            ucpar: self.parity.ucpar(),
            ucmsb: self.order.to_bool(),
            uc7bit: self.cnt.to_bool(),
            ucspb: self.stopbits.to_bool(),
            ucmode: self.mode.ucmode(),
            ucssel: clksel,
            // We want erroneous bytes to trigger RXIFG so all errors can be caught (UCRXEIE, SLAU445I
            // Table 22-8, p. 593)
            ucrxeie: true,
            ucbrkie: self.break_interrupts,
        });
        // Configure the ports (SLAU445I 22.3.1, p. 577, step 3): the caller passes the pins already in
        // their eUSCI function, and the remapping bits are set here
        USCI::configure_pin_mapping();
        // Everything is configured while UCSWRST is set, then the eUSCI is released (SLAU445I 22.3.1,
        // p. 577, step 4). Step 5, enabling interrupts, is left to Tx::enable_tx_interrupts and
        // Rx::enable_rx_interrupts.
        usci.ctl0_clear_rst();
    }

    /// Perform hardware configuration and split into Tx and Rx pins from appropriate GPIOs
    #[inline]
    pub fn split<T: Into<USCI::TxPin>, R: Into<USCI::RxPin>>(
        self,
        _tx: T,
        _rx: R,
    ) -> (Tx<USCI, M>, Rx<USCI, M>) {
        self.config_hw();
        (Tx(PhantomData, PhantomData), Rx(PhantomData, PhantomData))
    }

    /// Perform hardware configuration and create Tx pin from appropriate GPIO
    #[inline]
    pub fn tx_only<T: Into<USCI::TxPin>>(self, _tx: T) -> Tx<USCI, M> {
        self.config_hw();
        Tx(PhantomData, PhantomData)
    }

    /// Perform hardware configuration and create Rx pin from appropriate GPIO
    #[inline]
    pub fn rx_only<R: Into<USCI::RxPin>>(self, _rx: R) -> Rx<USCI, M> {
        self.config_hw();
        Rx(PhantomData, PhantomData)
    }
}

/// Serial transmitter pin
pub struct Tx<USCI, M = DefaultMapping>(PhantomData<USCI>, PhantomData<M>)
where
    USCI: SerialUsci<M>,
    M: PinMap;

impl<USCI, M> Tx<USCI, M>
where
    USCI: SerialUsci<M>,
    M: PinMap,
{
    /// Enable Tx interrupts, which fire when ready to send (UCTXIE, SLAU445I 22.3.15.1, p. 590).
    #[inline(always)]
    pub fn enable_tx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.txie_set();
    }

    /// Disable Tx interrupts (UCTXIE, SLAU445I Table 22-17, p. 600)
    #[inline(always)]
    pub fn disable_tx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.txie_clear();
    }

    /// Enable interrupts when a character has been sent completely (UCTXCPTIE, SLAU445I Table 22-17, p. 600).
    /// Due to erratum USCI42 the flag is set after each character, even while the next one waits in the Tx
    /// buffer (SLAZ695J USCI42, SLAZ664S USCI42, SLAZ726B USCI42 and SLAZ705H USCI42).
    #[inline(always)]
    pub fn enable_tx_complete_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.txcptie_set();
    }

    /// Disable interrupts when a character has been sent completely (UCTXCPTIE, SLAU445I Table 22-17, p. 600)
    #[inline(always)]
    pub fn disable_tx_complete_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.txcptie_clear();
    }

    /// The highest-priority pending interrupt of this eUSCI among the enabled ones, in the order of UCAxIV
    /// (SLAU445I Table 22-19, p. 602), shared with [`Rx::interrupt_source`]. It clears `StartBit` and
    /// `TxComplete`, as a UCAxIV read does (SLAU445I 22.3.15.4, p. 591), but leaves the flags of `RxBufFull`
    /// and `TxBufEmpty` for reading or writing the character to clear, so the read and write methods work
    /// after it.
    #[inline(always)]
    pub fn interrupt_source(&mut self) -> UartVector {
        let usci = unsafe { USCI::steal() };
        uart_vector(&usci)
    }

    /// Send an address character, in the multiprocessor modes (UCTXADDR): preceded by an idle line in the
    /// idle-line format, with the address bit set in the address-bit format (SLAU445I 22.3.3.1.1, p. 578 and
    /// SLAU445I 22.3.3.2, p. 579). Returns `WouldBlock` until the Tx buffer is free.
    #[inline]
    pub fn send_address(&mut self, address: u8) -> nb::Result<(), Infallible> {
        let usci = unsafe { USCI::steal() };
        if !usci.txifg_rd() {
            return Err(nb::Error::WouldBlock);
        }
        // "Set UCTXADDR, then write the address character to UCAxTXBUF. UCAxTXBUF must be ready for new data
        // (UCTXIFG = 1)." (SLAU445I 22.3.3.1.1, p. 578)
        usci.txaddr_set();
        usci.tx_wr(address);
        Ok(())
    }

    /// Send a break: all bits low for a character time, or in automatic baud-rate mode a 13-bit break, the
    /// break delimiter and the synch field 0x55, as LIN needs (UCTXBRK; SLAU445I 22.3.3.2.1, p. 579 and
    /// SLAU445I 22.3.4.1, p. 581). Returns `WouldBlock` until the Tx buffer is free.
    #[inline]
    pub fn send_break(&mut self) -> nb::Result<(), Infallible> {
        let usci = unsafe { USCI::steal() };
        if !usci.txifg_rd() {
            return Err(nb::Error::WouldBlock);
        }
        // UCMODEx (bits 10-9) = 11b: automatic baud-rate mode, which needs 055h instead of 0h in UCAxTXBUF
        // (SLAU445I Table 22-8, p. 593 to p. 594). Set UCTXBRK, then write UCAxTXBUF, which must be ready for
        // new data, UCTXIFG = 1 (SLAU445I 22.3.3.2.1, p. 579 and SLAU445I 22.3.4.1, p. 581).
        let auto_baud = usci.auto_baud_mode();
        usci.txbrk_set();
        usci.tx_wr(if auto_baud { 0x55 } else { 0x00 });
        Ok(())
    }

    // Internal flush function: done once the Tx buffer is empty and the last character has left the
    // shift register. UCBUSY also covers a character being received, so this can wait for that too
    // (UCTXIFG, UCBUSY: SLAU445I Table 22-18, p. 601 and SLAU445I Table 22-12, p. 596).
    // (UCTXCPTIFG can't be used: erratum USCI42 sets it after each character; SLAZ695J USCI42, SLAZ664S
    // USCI42, SLAZ726B USCI42 and SLAZ705H USCI42.)
    #[inline]
    fn flush(&mut self) -> nb::Result<(), Infallible> {
        let usci = unsafe { USCI::steal() };
        if usci.txifg_rd() && !usci.statw_rd().ucbusy() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline(always)]
    /// Writes a byte into the Tx buffer with no checks for validity
    /// # Safety
    /// May clobber unsent data still in the buffer (UCAxTXBUF holds the data waiting to be moved into the
    /// shift register, SLAU445I Table 22-14, p. 597)
    pub unsafe fn write_no_check(&mut self, data: u8) {
        let usci = unsafe { USCI::steal() };
        usci.tx_wr(data);
    }

    // Internal send function. It writes UCAxTXBUF only while UCTXIFG is set: "UCTXIFG is set when new data
    // can be written into UCAxTXBUF" (SLAU445I 22.3.8, p. 583), and writing clears it (SLAU445I Table 22-14,
    // p. 597).
    #[inline]
    fn send(&mut self, data: u8) -> nb::Result<(), Infallible> {
        let usci = unsafe { USCI::steal() };
        if usci.txifg_rd() {
            usci.tx_wr(data);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// Serial receiver pin
pub struct Rx<USCI, M = DefaultMapping>(PhantomData<USCI>, PhantomData<M>)
where
    USCI: SerialUsci<M>,
    M: PinMap;

impl<USCI, M> Rx<USCI, M>
where
    USCI: SerialUsci<M>,
    M: PinMap,
{
    /// Enable Rx interrupts, which fire when ready to read (UCRXIE, SLAU445I 22.3.15.2, p. 591)
    #[inline(always)]
    pub fn enable_rx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.rxie_set();
    }

    /// Disable Rx interrupts (UCRXIE, SLAU445I Table 22-17, p. 600)
    #[inline(always)]
    pub fn disable_rx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.rxie_clear();
    }

    /// Enable interrupts when a start bit is received (UCSTTIE, SLAU445I Table 22-17, p. 600 and SLAU445I
    /// Table 22-6, p. 591), for example to wake up from a low-power mode in time for the character (SLAU445I
    /// 22.2, p. 575: "Receiver start-edge detection for automatic wake up from LPMx modes")
    #[inline(always)]
    pub fn enable_start_bit_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        // Clear an old UCSTTIFG, then set UCSTTIE (SLAU445I Table 22-18, p. 601; SLAU445I Table 22-17, p. 600)
        usci.sttifg_clear();
        usci.sttie_set();
    }

    /// Disable interrupts when a start bit is received (UCSTTIE, SLAU445I Table 22-17, p. 600)
    #[inline(always)]
    pub fn disable_start_bit_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.sttie_clear();
    }

    /// The highest-priority pending interrupt of this eUSCI among the enabled ones, in the order of UCAxIV
    /// (SLAU445I Table 22-19, p. 602), shared with [`Tx::interrupt_source`]. It clears `StartBit` and
    /// `TxComplete`, as a UCAxIV read does (SLAU445I 22.3.15.4, p. 591), but leaves the flags of `RxBufFull`
    /// and `TxBufEmpty` for reading or writing the character to clear, so the read and write methods work
    /// after it.
    #[inline(always)]
    pub fn interrupt_source(&mut self) -> UartVector {
        let usci = unsafe { USCI::steal() };
        uart_vector(&usci)
    }

    /// In the multiprocessor and automatic baud-rate modes, receive only address characters, or the character
    /// after a break and synch field, while dormant (UCDORM, SLAU445I Table 22-8, p. 594). After receiving
    /// one that's for this device, leave the dormant state to receive the data that follows (SLAU445I
    /// 22.3.3.1, p. 578; SLAU445I 22.3.3.2, p. 579; SLAU445I 22.3.4, p. 580: "user software must reset UCDORM
    /// to continue receiving data").
    #[inline(always)]
    pub fn set_dormant(&mut self, dormant: bool) {
        let usci = unsafe { USCI::steal() };
        usci.dormant(dormant);
    }

    /// Like reading a character, but also returns whether it's an address character, in the multiprocessor
    /// modes (UCADDR, UCIDLE, SLAU445I Table 22-12, p. 596).
    #[inline]
    pub fn read_with_address_flag(&mut self) -> nb::Result<(u8, bool), RecvError> {
        let usci = unsafe { USCI::steal() };
        // The flag is cleared when the character is read, so read it first (SLAU445I Table 22-13, p. 597)
        let address = usci.statw_rd().ucaddr_ucidle();
        self.recv().map(|data| (data, address))
    }

    /// In automatic baud-rate mode: whether a break was longer than 22 bit times (UCBTOE), and whether a
    /// synch field was too long to measure (UCSTOE). See SLAU445I Table 22-15, p. 598; the text in SLAU445I
    /// 22.3.4, p. 580 says UCBTOE is set when the break "exceeds 21 bit times". This follows the register
    /// table, which describes the flag itself.
    #[inline]
    pub fn auto_baud_errors(&self) -> (bool, bool) {
        let usci = unsafe { USCI::steal() };
        (usci.btoe_rd(), usci.stoe_rd())
    }

    /// Reads raw value from Rx buffer with no checks for validity
    /// # Safety
    /// May read duplicate data (UCAxRXBUF keeps the last received character, SLAU445I Table 22-13, p. 597)
    #[inline(always)]
    pub unsafe fn read_no_check(&mut self) -> u8 {
        let usci = unsafe { USCI::steal() };
        usci.rx_rd()
    }

    // Internal recieve function
    fn recv(&mut self) -> nb::Result<u8, RecvError> {
        let usci = unsafe { USCI::steal() };

        if usci.rxifg_rd() {
            // UCAxSTATW first: reading UCAxRXBUF clears the error flags (SLAU445I 22.3.6, p. 582)
            let statw = usci.statw_rd();
            let data = usci.rx_rd();
            // Reading UCAxRXBUF "clears all error flags except UCOE, if UCAxRXBUF was overwritten between
            // the read access to UCAxSTATW and to UCAxRXBUF. Therefore, the UCOE flag should be checked
            // after reading UCAxRXBUF to detect this condition" (SLAU445I 22.3.6, p. 582)
            let overrun_between_reads = usci.statw_rd().ucoe();

            // UCBRK, UCFE, UCPE and UCOE in UCAxSTATW (SLAU445I Table 22-12, p. 596)
            if statw.ucbrk() {
                Err(nb::Error::Other(RecvError::Break))
            } else if statw.ucfe() {
                Err(nb::Error::Other(RecvError::Framing))
            } else if statw.ucpe() {
                Err(nb::Error::Other(RecvError::Parity))
            } else if statw.ucoe() || overrun_between_reads {
                Err(nb::Error::Other(RecvError::Overrun(data)))
            } else {
                Ok(data)
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// Serial receive errors
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
pub enum RecvError {
    /// A break was received: all bits low, see [`SerialConfig::break_interrupts`] (UCBRK, SLAU445I
    /// Table 22-1, p. 582)
    Break,
    /// Framing error (UCFE, SLAU445I Table 22-1, p. 582)
    Framing,
    /// Parity error (UCPE, SLAU445I Table 22-1, p. 582)
    Parity,
    /// Buffer overrun error. Contains the most recently read byte, which is still valid (UCOE: the character
    /// was loaded into UCAxRXBUF before the prior one was read, SLAU445I Table 22-1, p. 582).
    Overrun(u8),
}
// embedded-io requires this
impl Display for RecvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl core::error::Error for RecvError {}

mod emb_io {
    use super::*;
    use embedded_io::{Error, ErrorType, Read, ReadReady, Write, WriteReady};
    use nb::block;

    impl<USCI, M> ErrorType for Rx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        type Error = RecvError;
    }
    impl Error for RecvError {
        fn kind(&self) -> embedded_io::ErrorKind {
            match self {
                RecvError::Break        => embedded_io::ErrorKind::Other,
                RecvError::Framing      => embedded_io::ErrorKind::Other,
                RecvError::Parity       => embedded_io::ErrorKind::Other,
                RecvError::Overrun(_)   => embedded_io::ErrorKind::Other,
            }
        }
    }
    impl<USCI, M> Read for Rx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        #[inline]
        /// Read one byte into the specified buffer, then returns the number of bytes sent (1).
        /// If a byte isn't currently available to read, this function blocks until one is available.
        ///
        /// If `buf` is length zero, `write` returns `Ok(0)` without blocking.
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            if buf.is_empty() { return Ok(0) }
            buf[0] = block!(self.recv())?;
            Ok(1)
        }
    }
    impl<USCI, M> ReadReady for Rx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            let usci = unsafe { USCI::steal() };
            // UCRXIFG: UCAxRXBUF has received a complete character (SLAU445I Table 22-18, p. 601)
            Ok(usci.rxifg_rd())
        }
    }

    impl<USCI, M> ErrorType for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        type Error = Infallible;
    }

    impl<USCI, M> Write for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
        /// even if there's still more buffered (SLAZ695J USCI42, SLAZ664S USCI42, SLAZ726B USCI42 and
        /// SLAZ705H USCI42). Thus, the implementation uses UCTXIFG and UCBUSY instead. When `flush()`
        /// completes, the Tx buffer is empty and the last character has left the shift register (UCBUSY,
        /// SLAU445I Table 22-12, p. 596).
        ///
        /// As the error type is `Infallible`, this can be safely unwrapped.
        #[inline]
        fn flush(&mut self) -> Result<(), Self::Error> { block!(self.flush()) }

        #[inline]
        /// This function sends only **THE FIRST** byte in the buffer, blocking until the writer is ready to accept, then returns `Ok(1)`.
        /// If you want to send the entire buffer use `write_all()` instead.
        ///
        /// If `buf` is length zero, `write` returns `Ok(0)` without blocking.
        ///
        /// As the error type is `Infallible`, this can be safely unwrapped.
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            if buf.is_empty() { return Ok(0) }
            block!(self.send(buf[0]))?;
            Ok(1)
        }
        // The default version of this impl panics if .write() returns Ok(0) when given a non-empty buffer. Our impl never does this, so remove it.
        /// Write an entire buffer into this writer.
        ///
        /// This function calls `write()` in a loop until exactly `buf.len()` bytes have
        /// been written, blocking if needed.
        ///
        /// If you are using [`WriteReady`] to avoid blocking, you should not use this function.
        /// `WriteReady::write_ready()` returning true only guarantees the first call to `write()` will
        /// not block, so this function may still block in subsequent calls.
        fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Self::Error> {
            while !buf.is_empty() {
                let Ok(n) = self.write(buf);
                buf = &buf[n..];
            }
            Ok(())
        }
    }
    impl<USCI, M> WriteReady for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        /// Whether the writer is ready for immediate writing. If this returns `true`, the next call to [`Write::write`] will not block.
        /// (UCTXIFG: the Tx buffer is empty, SLAU445I Table 22-18, p. 601.)
        ///
        /// As the error type is `Infallible`, this can be safely unwrapped.
        fn write_ready(&mut self) -> Result<bool, Self::Error> {
            let usci = unsafe { USCI::steal() };
            Ok(usci.txifg_rd())
        }
    }
}

mod ehal_nb1 {
    use super::*;
    use embedded_hal_nb::serial::{Error, ErrorKind, ErrorType, Read, Write};

    impl Error for RecvError {
        fn kind(&self) -> ErrorKind {
            match self {
                RecvError::Break        => ErrorKind::FrameFormat,
                RecvError::Framing      => ErrorKind::FrameFormat,
                RecvError::Parity       => ErrorKind::Parity,
                RecvError::Overrun(_)   => ErrorKind::Overrun,
            }
        }
    }
    impl<USCI, M> ErrorType for Rx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        type Error = RecvError;
    }

    impl<USCI, M> Read<u8> for Rx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        #[inline]
        /// Check if Rx interrupt flag is set. If so, try reading the received byte and clear the flag
        /// (reading UCAxRXBUF clears UCRXIFG, SLAU445I 22.3.15.2, p. 591).
        /// Otherwise return `WouldBlock`. May return errors caused by data corruption or
        /// buffer overruns.
        fn read(&mut self) -> nb::Result<u8, Self::Error> { self.recv() }
    }

    impl<USCI, M> ErrorType for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        type Error = Infallible;
    }

    impl<USCI, M> Write<u8> for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
        /// even if there's still more buffered (SLAZ695J USCI42, SLAZ664S USCI42, SLAZ726B USCI42 and
        /// SLAZ705H USCI42). Thus, the implementation uses UCTXIFG and UCBUSY instead. When `flush()`
        /// completes, the Tx buffer is empty and the last character has left the shift register (UCBUSY,
        /// SLAU445I Table 22-12, p. 596).
        #[inline]
        fn flush(&mut self) -> nb::Result<(), Self::Error> { self.flush() }

        #[inline]
        /// Check if Tx interrupt flag is set (UCTXIFG, set when the Tx buffer is empty, SLAU445I Table 22-18,
        /// p. 601). If so, write a byte into the Tx buffer. Otherwise return `WouldBlock`
        fn write(&mut self, data: u8) -> nb::Result<(), Self::Error> { self.send(data) }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::serial::{Read, Write};

    impl<USCI, M> Read<u8> for Rx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        type Error = RecvError;

        #[inline]
        /// Check if Rx interrupt flag is set. If so, try reading the received byte and clear the flag
        /// (reading UCAxRXBUF clears UCRXIFG, SLAU445I 22.3.15.2, p. 591).
        /// Otherwise return `WouldBlock`. May return errors caused by data corruption or
        /// buffer overruns.
        fn read(&mut self) -> nb::Result<u8, Self::Error> { self.recv() }
    }

    impl<USCI, M> Write<u8> for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {
        type Error = void::Void;

        /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
        /// even if there's still more buffered (SLAZ695J USCI42, SLAZ664S USCI42, SLAZ726B USCI42 and
        /// SLAZ705H USCI42). Thus, the implementation uses UCTXIFG and UCBUSY instead. When `flush()`
        /// completes, the Tx buffer is empty and the last character has left the shift register (UCBUSY,
        /// SLAU445I Table 22-12, p. 596).
        #[inline]
        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.flush().map_err(|_| nb::Error::WouldBlock)
        }

        #[inline]
        /// Check if Tx interrupt flag is set (UCTXIFG, set when the Tx buffer is empty, SLAU445I Table 22-18,
        /// p. 601). If so, write a byte into the Tx buffer. Otherwise return `WouldBlock`
        fn write(&mut self, data: u8) -> nb::Result<(), Self::Error> {
            self.send(data).map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl<USCI, M> embedded_hal_02::blocking::serial::write::Default<u8> for Tx<USCI, M>
    where
        USCI: SerialUsci<M>,
        M: PinMap,
    {}
}
