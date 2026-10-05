//! SPI
//!
//! Peripherals eUSCI_A0, eUSCI_A1, eUSCI_B0 and eUSCI_B1 can be used for SPI communication as either a
//! master or slave device (SLAU445I 23.1, p. 604; SLAU445I 23.2, p. 604).
//!
//! Begin by calling [`SpiConfig::new()`]. Once configured either an [`Spi`] or [`SpiSlave`] will be returned.
//!
//! Note that even if you are only using the legacy embedded-hal 0.2.7 trait implementations, configuration of the SPI bus
//! uses the embedded-hal 1.0 versions of types (e.g. [`Mode`]).
//!
//! # [`Spi`]
//! The SPI peripheral can be configured as a master device by calling one of the
//! [`to_master()`](SpiConfig::to_master_using_smclk) methods during configuration.
//!
//! [`Spi`] implements the embedded-hal [`SpiBus`](embedded_hal::spi::SpiBus) trait, which provides a simple blocking interface.
//! A non-blocking implementation is also available through [`embedded-hal-nb`](embedded_hal_nb)'s
//! [`FullDuplex`](embedded_hal_nb::spi::FullDuplex) trait.
//! Standalone methods are also provided for directly writing to the Tx and Rx buffers for interrupt-based implementations.
//!
//! An [`Spi`] can share the bus with other masters, set up with
//! [`multi_master_bus()`](SpiConfig::multi_master_bus): another master takes the bus through this device's
//! STE pin, and SCLK and MOSI then stop driving it (SLAU445I 23.3.3.1, p. 608). Writes wait while
//! another master has the bus, and a transfer that another master interrupts returns
//! [`SpiErr::BusConflict`].
//!
//! # [`SpiSlave`]
//! The SPI peripheral can be configured as a slave device by calling [`to_slave()`](SpiConfig::to_slave) during configuration.
//!
//! [`SpiSlave`] supports sharing the bus with other slave devices by calling the
//! [`shared_bus()`](SpiConfig::shared_bus) method during configuration. In this mode the STE pin controls
//! whether the MISO pin is an output or a high-impedance pin, allowing other slaves to use the MISO bus when
//! this device is not selected (SLAU445I 23.3.4.1, p. 609). The polarity of the STE pin is configurable to
//! either active high or active low (UCMODEx, SLAU445I Table 23-1, p. 606).
//! If the bus is used exclusively by this device then the [`exclusive_bus()`](SpiConfig::exclusive_bus)
//! configuration method can be used, which allows the STE pin to be used for other purposes. In this mode the
//! MISO pin will remain an output pin at all times (SLAU445I 23.3.4.1, p. 609: "The UCxSTE input signal
//! is not used in 3-pin slave mode").
//!
//! [`SpiSlave`] provides non-blocking methods that can be used for polling or interrupt-based implementations.
//! It does not implement either of the embedded-hal traits.
//!
//! On the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2, a slave with UCCKPH = 1 has erratum USCI47: see
//! [`to_slave()`](SpiConfig::to_slave).
//!
//! Pins used (pins with `RemappedMapping` in brackets):
//!
//! | Device       | eUSCI | MISO          | MOSI          | SCLK          | STE           |
//! |:-------------|:-----:|:-------------:|:-------------:|:-------------:|:-------------:|
//! | MSP430FR2x5x | A0    | `P1.6`        | `P1.7`        | `P1.5`        | `P1.4`        |
//! | MSP430FR2x5x | A1    | `P4.2`        | `P4.3`        | `P4.1`        | `P4.0`        |
//! | MSP430FR2x5x | B0    | `P1.3`        | `P1.2`        | `P1.1`        | `P1.0`        |
//! | MSP430FR2x5x | B1    | `P4.7`        | `P4.6`        | `P4.5`        | `P4.4`        |
//! | MSP430FR2433 | A0    | `P1.5`        | `P1.4`        | `P1.6`        | `P1.7`        |
//! | MSP430FR2433 | A1    | `P2.5`        | `P2.6`        | `P2.4`        | `P3.1`        |
//! | MSP430FR2433 | B0    | `P1.3`        | `P1.2`        | `P1.1`        | `P1.0`        |
//! | MSP430FR247x | A0    | `P1.5` (`P5.1`) | `P1.4` (`P5.2`) | `P1.6` (`P5.0`) | `P1.7` (`P4.7`) |
//! | MSP430FR247x | A1    | `P2.5`        | `P2.6`        | `P2.4`        | `P3.1`        |
//! | MSP430FR247x | B0    | `P1.3` (`P4.5`) | `P1.2` (`P4.6`) | `P1.1` (`P5.5`) | `P1.0` (`P5.6`) |
//! | MSP430FR247x | B1    | `P3.6` (`P4.3`) | `P3.2` (`P4.4`) | `P3.5` (`P5.3`) | `P2.7` (`P5.4`) |
//! | MSP430FR25x2 | A0    | `P1.5` (`P2.1`) | `P1.4` (`P2.0`) | `P1.6`        | `P1.7`        |
//! | MSP430FR25x2 | B0    | `P1.3` (`P2.6`) | `P1.2` (`P2.5`) | `P1.1` (`P2.4`) | `P1.0` (`P2.3`) |
//!
//! The pins are those of the eUSCI pin configuration tables (SOMI is MISO, SIMO is MOSI):
//! SLASEC4D Table 6-14, p. 72 (MSP430FR2x5x), SLASE59F Table 6-10, p. 49 (MSP430FR2433),
//! SLASEO7C Table 9-11, p. 54 (MSP430FR247x) and SLASEE4C Table 6-11, p. 53 (MSP430FR25x2).
//!
//! Some packages lack pins: on the MSP430FR2433 in the DSBGA (YQW) package, eUSCI_A1 has no SCLK and STE pins,
//! and on the MSP430FR2x5x in the VQFN-32 (RSM) package eUSCI_B1 only supports I2C (data sheets, device
//! comparison and pin attributes: SLASE59F Table 3-1, p. 7 and SLASE59F Table 4-1, p. 11;
//! SLASEC4D Table 3-1, p. 8, note (4) and SLASEC4D Table 4-1, p. 19, note (7)).
#[cfg(feature = "eusci_aclk")]
use crate::clock::Aclk;
use crate::{
    clock::Smclk,
    hw_traits::eusci::{EusciSPI, SpiStatw, Ucmode, Ucssel, UcxSpiCtw0},
    pin_mapping::*,
};
use core::{convert::Infallible, marker::PhantomData};
use nb::Error::WouldBlock;

use embedded_hal::spi::{Mode, Phase, Polarity};

/// Marks a eUSCI capable of SPI communication (in this case, all euscis do: SLAU445I 23.1, p. 604)
pub trait SpiUsci<M: PinMap = DefaultMapping>: EusciSPI {
    /// Master In Slave Out (refered to as SOMI in datasheet; UCxSOMI, SLAU445I 23.3, p. 606)
    type MISO;
    /// Master Out Slave In (refered to as SIMO in datasheet; UCxSIMO, SLAU445I 23.3, p. 606)
    type MOSI;
    /// Serial Clock (UCxCLK, SLAU445I 23.3, p. 606)
    type SCLK;
    /// Slave Transmit Enable (acts like CS; UCxSTE, SLAU445I 23.3, p. 606)
    type STE: SpiPinLevel;

    /// Additional configuration
    #[inline(always)]
    fn configure_pin_mapping() {}
}

// Allows a GPIO pin to be converted into an SPI object
// The pin's alternate function defaults to Alternate1: PxSEL = 01b, the primary module function
// (SLAU445I Table 8-3, p. 314). The device_specific files cite each pin's function in its data sheet.
macro_rules! impl_spi_pin {
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl_spi_pin!($struct_name, $port, $pin, Alternate1);
    };
    ($struct_name: ident, $port: ty, $pin: ty, $alt: ident) => {
        impl<DIR> From<Pin<$port, $pin, $alt<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, $alt<DIR>>) -> Self { $struct_name }
        }
        impl $crate::spi::SpiPinLevel for $struct_name {
            // The pin's bit in PxIN (SLAU445I Table 8-9, p. 334)
            #[inline(always)]
            fn is_high() -> bool {
                use $crate::hw_traits::{gpio::GpioPeriph, Steal};
                let port = unsafe { <$port as Steal>::steal() };
                port.pxin_rd() & <$pin as $crate::gpio::PinNum>::SET_MASK != 0
            }
        }
    };
}
pub(crate) use impl_spi_pin;

/// The level of an SPI pin, read from its bit in PxIN. PxIN.x and the eUSCI's input both come from the
/// pin's Schmitt trigger, which only an analog function switches off (SLASEC4D Figure 6-4, p. 95;
/// SLASE59F Figure 6-1, p. 54; SLASEO7C Figure 9-4, p. 64; SLASEE4C Figure 6-3, p. 57), so the bit reads the
/// pin in its eUSCI function too (measured on an MSP430FR2476, with STE).
pub trait SpiPinLevel {
    /// Whether the pin is high
    fn is_high() -> bool;
}

/// Typestate for an SPI bus whose role has not yet been chosen.
pub struct RoleNotSet;
/// Typestate for an SPI bus being configured as a master device.
pub struct Master;
/// Typestate for an SPI bus being configured as a slave device.
pub struct Slave;

/// Configuration object for an eUSCI peripheral being set up for SPI mode.
pub struct SpiConfig<USCI, ROLE, M: PinMap = DefaultMapping>
where USCI: SpiUsci<M>
{
    usci: USCI,
    ctlw0: UcxSpiCtw0,
    prescaler: u16,
    _phantom: PhantomData<(ROLE, M)>,
}

impl<USCI, M> SpiConfig<USCI, RoleNotSet, M>
where
    USCI: SpiUsci<M>,
    M: PinMap,
{
    /// Begin configuring an EUSCI peripheral for SPI mode.
    pub fn new(usci: USCI, mode: Mode, msb_first: bool) -> Self {
        // UCCKPH = 1 captures data on the first clock edge, UCCKPL = 1 idles high, UCMSB = 1 sends the MSB first,
        // UC7BIT = 0 is 8-bit data, UCSTEM = 0 makes STE an input in 4-pin master mode, and UCSYNC = 1 selects
        // SPI mode (SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620)
        let ctlw0 = UcxSpiCtw0 {
            ucckph: match mode.phase {
                Phase::CaptureOnFirstTransition => true,
                Phase::CaptureOnSecondTransition => false,
            },
            ucckpl: match mode.polarity {
                Polarity::IdleLow => false,
                Polarity::IdleHigh => true,
            },
            ucmsb: msb_first,
            ucsync: true,
            ucswrst: true,
            // UCSTEM = 1 is only used by `single_slave_bus`, see there
            ucstem: false,
            uc7bit: false,
            ..Default::default()
        };

        Self { usci, ctlw0, prescaler: 0, _phantom: PhantomData }
    }
    /// This device will act as a slave on the SPI bus (UCMST = 0, SLAU445I Table 23-3, p. 613;
    /// SLAU445I Table 23-12, p. 620).
    ///
    /// Erratum USCI47, on the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2: a slave with UCCKPH = 1
    /// (`CaptureOnFirstTransition`, as in `MODE_0` and `MODE_2`) sends wrong data, and an eUSCI_A slave
    /// receives nothing, if SCLK isn't at its idle level when the eUSCI leaves reset (SLAZ695J USCI47,
    /// p. 12 to p. 13; SLAZ664S USCI47, p. 14; SLAZ705H USCI47, p. 10 to p. 11). `shared_bus()` and
    /// `exclusive_bus()` release it from reset (UCSWRST = 0) before they return. The HAL can't apply
    /// the erratum's workarounds by itself:
    /// - "Use clock phase mode UCCKPH = 0 for MSP SPI slave if allowed by the application":
    ///   `CaptureOnSecondTransition`, as in `MODE_1` and `MODE_3`.
    /// - "The SPI master must set the clock pin at the appropriate idle level (low for UCCKPL = 0, high
    ///   for UCCKPL = 1) before SPI slave is reset (UCSWRST bit is cleared)": set the slave up while the
    ///   master keeps SCLK idle.
    /// - For an eUSCI_A slave, "If UCTXIFG is set twice but UCRXIFG is not set, reset the MSP SPI slave
    ///   by setting and then clearing the UCSWRST bit, and inform the SPI master to resend the data":
    ///   [`SpiSlave::reset()`] does the reset.
    pub fn to_slave(mut self) -> SpiConfig<USCI, Slave, M> {
        self.ctlw0.ucmst = false;
        // UCSSEL is 'don't care' in slave mode (SLAU445I 23.3.6, p. 609)
        SpiConfig {
            usci: self.usci,
            prescaler: self.prescaler,
            ctlw0: self.ctlw0,
            _phantom: PhantomData,
        }
    }
    /// This device will act as a master on the SPI bus, deriving SCLK from SMCLK.
    /// (UCMST = 1 and UCSSELx = 10b, SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620. SCLK is
    /// SMCLK / `clk_div`, and 0 also divides by 1: SLAU445I 23.3.6, p. 609.)
    ///
    /// SMCLK is derived from MCLK (SLAU445I 3.2.1, p. 102), so it's synchronous to MCLK, which is the
    /// workaround of erratum USCI45 on the MSP430FR2x5x and MSP430FR2433: with an SCLK source
    /// asynchronous to MCLK, the clock high phase of the first data bit can be stretched (SLAZ695J
    /// USCI45, p. 12; SLAZ664S USCI45, p. 13 to p. 14).
    pub fn to_master_using_smclk(
        mut self,
        _smclk: &Smclk,
        clk_div: u16,
    ) -> SpiConfig<USCI, Master, M> {
        self.ctlw0.ucmst = true;
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_div;
        SpiConfig {
            usci: self.usci,
            prescaler: self.prescaler,
            ctlw0: self.ctlw0,
            _phantom: PhantomData,
        }
    }
    #[cfg(feature = "eusci_aclk")]
    /// This device will act as a master on the SPI bus, deriving SCLK from ACLK.
    /// (UCMST = 1: SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620. UCSSELx = 01b is ACLK on these
    /// devices: SLASEC4D Table 6-9, p. 68; SLASEO7C Table 9-8, p. 50; SLASEE4C Table 6-8, p. 49. SCLK is
    /// ACLK / `clk_div`, and 0 also divides by 1: SLAU445I 23.3.6, p. 609.)
    ///
    /// Erratum USCI45, on the MSP430FR2x5x: when ACLK is asynchronous to MCLK, the clock high phase of the
    /// first data bit can in rare cases be stretched significantly; no data is lost (SLAZ695J USCI45,
    /// p. 12). The erratum's workaround is an SCLK source synchronous to MCLK, such as SMCLK
    /// ([`to_master_using_smclk()`](Self::to_master_using_smclk)).
    pub fn to_master_using_aclk(
        mut self,
        _aclk: &Aclk,
        clk_div: u16,
    ) -> SpiConfig<USCI, Master, M> {
        self.ctlw0.ucmst = true;
        self.ctlw0.ucssel = Ucssel::DeviceSpecific;
        self.prescaler = clk_div;
        SpiConfig {
            usci: self.usci,
            prescaler: self.prescaler,
            ctlw0: self.ctlw0,
            _phantom: PhantomData,
        }
    }
    #[cfg(feature = "eusci_modclk")]
    /// This device will act as a master on the SPI bus, deriving SCLK from MODCLK.
    /// (UCMST = 1: SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620. UCSSELx = 01b is MODCLK on
    /// the MSP430FR2433: SLASE59F Table 6-7, p. 46. SCLK is MODCLK / `clk_div`, and 0 also divides by 1:
    /// SLAU445I 23.3.6, p. 609.)
    ///
    /// Erratum USCI45: when the SCLK source is asynchronous to MCLK, the clock high phase of the first data
    /// bit can in rare cases be stretched significantly; no data is lost (SLAZ664S USCI45, p. 13 to
    /// p. 14). MODCLK always is: it comes from its own oscillator, MODOSC, which MCLK can't run from
    /// (SLAU445I 3.2.15, p. 111; SELMS: SLAU445I Table 3-8, p. 117). The erratum's workaround is an SCLK
    /// source synchronous to MCLK, such as SMCLK ([`to_master_using_smclk()`](Self::to_master_using_smclk)).
    ///
    /// This also sets MODOSCREQEN, so that MODCLK runs for the eUSCI whichever kind of request it makes
    /// (SLAU445I 3.2.15.1, p. 111; SLAU445I Table 3-12, p. 123).
    pub fn to_master_using_modclk(mut self, clk_div: u16) -> SpiConfig<USCI, Master, M> {
        crate::clock::enable_modosc_conditional_requests();
        self.ctlw0.ucmst = true;
        self.ctlw0.ucssel = Ucssel::DeviceSpecific;
        self.prescaler = clk_div;
        SpiConfig {
            usci: self.usci,
            prescaler: self.prescaler,
            ctlw0: self.ctlw0,
            _phantom: PhantomData,
        }
    }
}

impl<USCI, M> SpiConfig<USCI, Master, M>
where
    USCI: SpiUsci<M>,
    M: PinMap,
{
    /// For an SPI bus with a single slave, whose enable signal the eUSCI generates on the STE pin (UCSTEM,
    /// SLAU445I 23.3.3.2, p. 608).
    ///
    /// The eUSCI asserts STE while it transfers: bytes written as soon as the Tx buffer is free keep it asserted
    /// (measured on an MSP430FR2476), but it's released whenever the eUSCI runs out of data. Slaves that need
    /// their enable signal held for a whole transaction need a GPIO pin instead.
    pub fn single_slave_bus<MOSI, MISO, SCLK, STE>(
        mut self,
        _miso: MISO,
        _mosi: MOSI,
        _sclk: SCLK,
        _ste: STE,
        ste_pol: StePolarity,
    ) -> Spi<USCI, M>
    where
        MOSI: Into<USCI::MOSI>,
        MISO: Into<USCI::MISO>,
        SCLK: Into<USCI::SCLK>,
        STE: Into<USCI::STE>,
    {
        // UCMODEx = 01b: STE active high, 10b: STE active low (SLAU445I Table 23-3, p. 613;
        // SLAU445I Table 23-12, p. 620)
        self.ctlw0.ucmode = match ste_pol {
            StePolarity::EnabledWhenHigh => Ucmode::FourPinSPI1,
            StePolarity::EnabledWhenLow  => Ucmode::FourPinSPI0,
        };
        // UCSTEM = 1: STE is an output, the enable signal of a single slave (SLAU445I 23.3.3.2, p. 608)
        self.ctlw0.ucstem = true;
        self.configure_hw();
        Spi { usci: self.usci, ste_master_active: None, _pin_map: PhantomData }
    }

    /// For an SPI bus with more than one master (4-pin master mode with UCSTEM = 0: SLAU445I 23.3.3.1,
    /// p. 608). `ste_pol` is the STE level at which this master may use the bus: `EnabledWhenHigh` while STE
    /// is high, `EnabledWhenLow` while it's low. Another master takes the bus by driving STE to the other
    /// level, and SCLK and MOSI then stop driving the bus (SLAU445I Table 23-1, p. 606). Give STE a pull
    /// resistor to the enabled level, with `pullup()` or `pulldown()` before `to_alternate1()`, if nothing
    /// drives it while the other masters are idle; the pull resistor works in the eUSCI function too, while
    /// STE is an input (SLASEC4D Figure 6-4, p. 95; SLASE59F Figure 6-1, p. 54; SLASEO7C Figure 9-4, p. 64;
    /// SLASEE4C Figure 6-3, p. 57).
    ///
    /// Erratum USCI50: "only move data into UCxTXBUF when UCxSTE is in the active state" (SLAZ695J USCI50,
    /// p. 13; SLAZ664S USCI50, p. 14 to p. 15; SLAZ726B USCI50, p. 8 to p. 9; SLAZ705H USCI50, p. 11).
    /// Measured on an MSP430FR2476, a character written to UCxTXBUF while another master has the bus is
    /// never sent. So the writes return `WouldBlock`, and the blocking ones wait, until
    /// [`Spi::bus_available()`]; check it before [`write_unchecked()`](Spi::write_unchecked).
    ///
    /// A transfer that another master interrupts is aborted, and the next read returns
    /// [`SpiErr::BusConflict`]: "the data must be rewritten" (SLAU445I 23.3.3.1, p. 608). Repeat the
    /// transaction then. Between characters, a write waits while another master has the bus and goes on
    /// after it.
    pub fn multi_master_bus<MOSI, MISO, SCLK, STE>(
        mut self,
        _miso: MISO,
        _mosi: MOSI,
        _sclk: SCLK,
        _ste: STE,
        ste_pol: StePolarity,
    ) -> Spi<USCI, M>
    where
        MOSI: Into<USCI::MOSI>,
        MISO: Into<USCI::MISO>,
        SCLK: Into<USCI::SCLK>,
        STE: Into<USCI::STE>,
    {
        // The master is active while STE is low with UCMODEx = 01b, while it's high with 10b (SLAU445I
        // Table 23-1, p. 606). UCSTEM = 0, as `new` sets it, makes STE an input (SLAU445I 23.3.3.1, p. 608).
        let ste_master_active = match ste_pol {
            StePolarity::EnabledWhenHigh => {
                self.ctlw0.ucmode = Ucmode::FourPinSPI0;
                true
            }
            StePolarity::EnabledWhenLow => {
                self.ctlw0.ucmode = Ucmode::FourPinSPI1;
                false
            }
        };
        self.configure_hw();
        Spi { usci: self.usci, ste_master_active: Some(ste_master_active), _pin_map: PhantomData }
    }

    /// For an SPI bus with a single master.
    /// SCLK and MOSI are always outputs. The STE pin is not required
    /// (3-pin master mode, UCMODEx = 00b: SLAU445I Table 23-3, p. 613; SLAU445I 23.3.3.1, p. 608: "The
    /// UCxSTE input signal is not used in 3-pin master mode").
    pub fn single_master_bus<MOSI, MISO, SCLK>(
        mut self,
        _miso: MISO,
        _mosi: MOSI,
        _sclk: SCLK,
    ) -> Spi<USCI, M>
    where
        MOSI: Into<USCI::MOSI>,
        MISO: Into<USCI::MISO>,
        SCLK: Into<USCI::SCLK>,
    {
        // UCMODEx = 00b: 3-pin SPI (SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620)
        self.ctlw0.ucmode = Ucmode::ThreePinSPI;
        self.configure_hw();
        Spi { usci: self.usci, ste_master_active: None, _pin_map: PhantomData }
    }
}
impl<USCI, M> SpiConfig<USCI, Slave, M>
where
    USCI: SpiUsci<M>,
    M: PinMap,
{
    /// For an SPI bus with more than one slave.
    /// The STE pin is used to turn MISO high impedance, so other slaves can talk on the bus
    /// (SLAU445I 23.3.4.1, p. 609).
    pub fn shared_bus<MOSI, MISO, SCLK, STE>(
        mut self,
        _miso: MISO,
        _mosi: MOSI,
        _sclk: SCLK,
        _ste: STE,
        ste_pol: StePolarity,
    ) -> SpiSlave<USCI, M>
    where
        MOSI: Into<USCI::MOSI>,
        MISO: Into<USCI::MISO>,
        SCLK: Into<USCI::SCLK>,
        STE: Into<USCI::STE>,
    {
        // UCMODEx = 01b: STE active high, 10b: STE active low (SLAU445I Table 23-3, p. 613;
        // SLAU445I Table 23-12, p. 620)
        self.ctlw0.ucmode = match ste_pol {
            StePolarity::EnabledWhenHigh => Ucmode::FourPinSPI1,
            StePolarity::EnabledWhenLow  => Ucmode::FourPinSPI0,
        };
        self.configure_hw();
        SpiSlave { usci: self.usci, _pin_map: PhantomData }
    }
    /// For an SPI bus where this device is the only slave.
    /// MISO is always an output (a slave's UCxSOMI is its data output, and 3-pin mode doesn't use STE:
    /// SLAU445I 23.3, p. 606; SLAU445I 23.3.4.1, p. 609).
    pub fn exclusive_bus<MOSI, MISO, SCLK>(
        mut self,
        _miso: MISO,
        _mosi: MOSI,
        _sclk: SCLK,
    ) -> SpiSlave<USCI, M>
    where
        MOSI: Into<USCI::MOSI>,
        MISO: Into<USCI::MISO>,
        SCLK: Into<USCI::SCLK>,
    {
        // UCMODEx = 00b: 3-pin SPI (SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620)
        self.ctlw0.ucmode = Ucmode::ThreePinSPI;
        self.configure_hw();
        SpiSlave { usci: self.usci, _pin_map: PhantomData }
    }
}
impl<USCI, M, ROLE> SpiConfig<USCI, ROLE, M>
where
    USCI: SpiUsci<M>,
    M: PinMap,
{
    /// Transfer 7-bit characters instead of 8-bit ones (UC7BIT). The top bit of each byte is then not sent,
    /// and reads as 0 (SLAU445I 23.3.2, p. 607; UCxTXBUF, SLAU445I Table 23-7, p. 616 and
    /// SLAU445I Table 23-16, p. 623).
    #[inline]
    pub fn seven_bit_characters(mut self) -> Self {
        self.ctlw0.uc7bit = true;
        self
    }

    #[inline]
    fn configure_hw(&self) {
        // Initialization procedure of SLAU445I 23.3.1, p. 606: UCxCTLW0, UCxBRW and UCLISTEN are
        // "Modify only when UCSWRST = 1" (SLAU445I 23.4.1 to 23.4.3, p. 613 to p. 615;
        // SLAU445I 23.5.1 to 23.5.3, p. 620 to p. 622)
        // 1. Set UCSWRST
        self.usci.ctw0_set_rst();

        // 2. Initialize the registers
        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.brw_wr(self.prescaler);
        self.usci.uclisten_clear();

        // 3. Configure ports: the caller passes the pins already in their eUSCI function, and the
        // remapping bits are set here
        USCI::configure_pin_mapping();

        // 4. Clear UCSWRST. The interrupts stay off: setting UCSWRST in step 1 cleared UCTXIE and UCRXIE
        // ("When set, the UCSWRST bit resets the UCRXIE, UCTXIE, UCRXIFG, UCOE, and UCFE bits", SLAU445I
        // 23.3.1, p. 606).
        self.usci.ctw0_clear_rst();
    }
}

#[cfg(feature = "mfm")]
impl<M: PinMap> SpiConfig<crate::pac::EUsciB1, Slave, M>
where crate::pac::EUsciB1: SpiUsci<M>
{
    /// Set eUSCI_B1 up as the 4-wire SPI slave of the Manchester Function Module, see [`crate::mfm`]. The MFM
    /// connects to it internally, so its own pins aren't needed (SLASEC4D 6.10.14, p. 79: "the eUSCI_B1 must
    /// be configured in 4-wire SPI slave mode"; SLAU445I 25.6.1, p. 668).
    ///
    /// Erratum USCI47 applies to this slave as to any other, see [`SpiConfig::to_slave`]: with UCCKPH = 1
    /// its output data can be wrong (SLAZ695J USCI47, p. 12 to p. 13).
    pub fn mfm_slave(mut self, ste_pol: StePolarity) -> SpiSlave<crate::pac::EUsciB1, M> {
        // UCMODEx = 01b: STE active high, 10b: STE active low (SLAU445I Table 23-12, p. 620)
        self.ctlw0.ucmode = match ste_pol {
            StePolarity::EnabledWhenHigh => Ucmode::FourPinSPI1,
            StePolarity::EnabledWhenLow  => Ucmode::FourPinSPI0,
        };
        self.configure_hw();
        SpiSlave { usci: self.usci, _pin_map: PhantomData }
    }
}

/// The polarity of the STE pin. The values are those of UCMODEx (SLAU445I Table 23-1, p. 606;
/// SLAU445I Table 23-3, p. 613; SLAU445I Table 23-12, p. 620).
pub enum StePolarity {
    /// This device is enabled when STE is high.
    EnabledWhenHigh = 0b01,
    /// This device is enabled when STE is low.
    EnabledWhenLow = 0b10,
}

macro_rules! spi_common {
    () => {
        /// Enable Rx interrupts, which fire when a byte is ready to be read
        /// (UCRXIE, SLAU445I 23.3.8.2, p. 611)
        #[inline(always)]
        pub fn set_rx_interrupt(&mut self) { self.usci.set_receive_interrupt(); }

        /// Disable Rx interrupts, which fire when a byte is ready to be read
        /// (UCRXIE, SLAU445I 23.3.8.2, p. 611)
        #[inline(always)]
        pub fn clear_rx_interrupt(&mut self) { self.usci.clear_receive_interrupt(); }

        /// Enable Tx interrupts, which fire when the transmit buffer is empty
        /// (UCTXIE, SLAU445I 23.3.8.1, p. 611)
        #[inline(always)]
        pub fn set_tx_interrupt(&mut self) { self.usci.set_transmit_interrupt(); }

        /// Disable Tx interrupts, which fire when the transmit buffer is empty
        /// (UCTXIE, SLAU445I 23.3.8.1, p. 611)
        #[inline(always)]
        pub fn clear_tx_interrupt(&mut self) { self.usci.clear_transmit_interrupt(); }

        /// Write a byte into the Tx buffer, without checking if the Tx buffer is empty. Returns immediately.
        /// Useful if you already know the buffer is empty (e.g. a Tx interrupt was triggered)
        /// # Safety
        /// May clobber previous unsent data if the TXIFG bit is not set ("Data written to UCxTXBUF when
        /// UCTXIFG = 0 may result in erroneous data transmission", SLAU445I 23.3.8.1, p. 611).
        #[inline(always)]
        pub unsafe fn write_unchecked(&mut self, val: u8) { self.usci.txbuf_wr(val) }

        /// Read the byte in the Rx buffer, without checking if the Rx buffer is ready.
        /// Useful when you already know the buffer is ready (e.g. an Rx interrupt was triggered).
        /// # Safety
        /// May read invalid data if RXIFG bit is not ready (UCRXIFG is set "each time a character is received",
        /// SLAU445I 23.3.8.2, p. 611).
        #[inline]
        pub unsafe fn read_unchecked(&mut self) -> Result<u8, SpiErr> {
            // UCFE and UCOE first: reading UCxRXBUF clears them (UCOE: SLAU445I Table 23-5, p. 615;
            // SLAU445I Table 23-14, p. 622; UCFE measured on an MSP430FR2476)
            let statw = self.usci.statw_rd();
            if statw.ucfe() {
                return Err(self.bus_conflict());
            }
            if statw.ucoe() {
                return Err(SpiErr::Overrun(self.usci.rxbuf_rd()));
            }
            Ok(self.usci.rxbuf_rd())
        }

        fn recv_byte(&mut self) -> nb::Result<u8, SpiErr> {
            if self.usci.receive_flag() {
                // UCFE and UCOE first, as in read_unchecked
                let statw = self.usci.statw_rd();
                if statw.ucfe() {
                    Err(nb::Error::Other(self.bus_conflict()))
                } else if statw.ucoe() {
                    Err(nb::Error::Other(SpiErr::Overrun(self.usci.rxbuf_rd())))
                } else {
                    Ok(self.usci.rxbuf_rd())
                }
            } else {
                Err(WouldBlock)
            }
        }

        // UCFE: another master interrupted a transfer on a multi-master bus, which aborted it (SLAU445I
        // 23.3.3.1, p. 608); UCFE isn't used otherwise (SLAU445I Table 23-5, p. 615). Measured on an
        // MSP430FR2476, the abort sets UCRXIFG too, with the last character still in UCxRXBUF, and a character
        // waiting in UCxTXBUF stays there and isn't sent. A reset of the eUSCI drops both and clears the
        // flags.
        fn bus_conflict(&mut self) -> SpiErr {
            self.reset_keeping_interrupts();
            SpiErr::BusConflict
        }

        // Set and clear UCSWRST, which ends a transfer in progress (SLAU445I 23.3.5, p. 609), sets UCTXIFG
        // and clears UCRXIFG, UCOE and UCFE (SLAU445I 23.3.1, p. 606). It clears the interrupt enables too,
        // so they're set again.
        fn reset_keeping_interrupts(&mut self) {
            let ie = self.usci.ie_rd();
            self.usci.ctw0_set_rst();
            self.usci.ctw0_clear_rst();
            self.usci.ie_wr(ie);
        }

        /// Get the source of the interrupt currently being serviced: the highest-priority pending interrupt
        /// among the enabled ones, in the order of UCxIV, UCRXIFG before UCTXIFG (SLAU445I Table 23-10,
        /// p. 618; SLAU445I Table 23-19, p. 625; "Disabled interrupts do not affect the UCxIV value",
        /// SLAU445I 23.3.8.3, p. 611).
        ///
        /// It's worked out from UCxIFG and UCxIE instead of read from UCxIV, because "any access, read or
        /// write, of the UCxIV register automatically resets the highest-pending interrupt flag" (SLAU445I
        /// 23.3.8.3, p. 611), after which the checked read and write methods would block. The flags stay set
        /// until UCxRXBUF is read or UCxTXBUF written (SLAU445I 23.3.8.2 and 23.3.8.1, p. 611), so the
        /// Tx interrupt keeps firing until a byte is written or Tx interrupts are disabled.
        #[inline]
        pub fn interrupt_source(&mut self) -> SpiVector {
            // UCRXIE and UCTXIE (SLAU445I Table 23-8, p. 617; SLAU445I Table 23-17, p. 624)
            if self.usci.receive_interrupt_enabled() && self.usci.receive_flag() {
                SpiVector::RxBufferFull
            } else if self.usci.transmit_interrupt_enabled() && self.usci.transmit_flag() {
                SpiVector::TxBufferEmpty
            } else {
                SpiVector::None
            }
        }
    };
}

/// Possible sources for an eUSCI SPI interrupt. The values are those of UCxIV (SLAU445I Table 23-10, p. 618;
/// SLAU445I Table 23-19, p. 625).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpiVector {
    /// No interrupt is currently being serviced.
    None = 0,
    /// The interrupt was caused by the Rx buffer being full.
    RxBufferFull = 2,
    /// The interrupt was caused by the Tx buffer being empty.
    TxBufferEmpty = 4,
}

/// Represents a group of pins configured for SPI communication
pub struct Spi<USCI, M: PinMap = DefaultMapping>
where USCI: SpiUsci<M>
{
    usci: USCI,
    // On a multi-master bus, the STE level at which this master may use the bus; `None` otherwise
    ste_master_active: Option<bool>,
    _pin_map: PhantomData<M>,
}
impl<USCI, M> Spi<USCI, M>
where
    USCI: SpiUsci<M>,
    M: PinMap,
{
    spi_common!();

    /// Whether this master may use the bus: on a bus set up with
    /// [`multi_master_bus()`](SpiConfig::multi_master_bus), whether STE is at the level given there
    /// (SLAU445I Table 23-1, p. 606), and always on the other buses.
    #[inline]
    pub fn bus_available(&self) -> bool {
        match self.ste_master_active {
            Some(level) => <USCI::STE as SpiPinLevel>::is_high() == level,
            None => true,
        }
    }

    fn send_byte(&mut self, word: u8) -> nb::Result<(), Infallible> {
        // UCTXIFG = 1: UCxTXBUF can take another character (SLAU445I 23.3.8.1, p. 611), on a multi-master
        // bus only while this master may use it (erratum USCI50, see `multi_master_bus`)
        if self.usci.transmit_flag() && self.bus_available() {
            self.usci.txbuf_wr(word);
            Ok(())
        } else {
            Err(WouldBlock)
        }
    }

    #[inline(always)]
    /// Change the SPI mode. This requires resetting the peripheral, which also sets TXIFG and clears RXIFG, UCOE, and UCFE.
    /// (UCCKPH and UCCKPL are "Modify only when UCSWRST = 1", SLAU445I Table 23-3, p. 613 and
    /// SLAU445I Table 23-12, p. 620; what UCSWRST resets: SLAU445I 23.3.1, p. 606.)
    pub fn change_mode(&mut self, mode: Mode) {
        let intrs = self.usci.ie_rd();
        self.usci.ctw0_set_rst();
        self.usci.set_spi_mode(mode);
        self.usci.ctw0_clear_rst();
        // The interrupt enables are held cleared while the eUSCI is in reset (SLAU445I 23.3.1, p. 606)
        self.usci.ie_wr(intrs);
    }
}

/// An eUSCI peripheral that has been configured into an SPI slave.
pub struct SpiSlave<USCI, M: PinMap = DefaultMapping>
where USCI: SpiUsci<M>
{
    usci: USCI,
    _pin_map: PhantomData<M>,
}
impl<USCI, M> SpiSlave<USCI, M>
where
    USCI: SpiUsci<M>,
    M: PinMap,
{
    spi_common!();

    /// Try to read from the Rx buffer. Returns `nb::WouldBlock` if the buffer is empty.
    #[inline(always)]
    pub fn read(&mut self) -> nb::Result<u8, SpiErr> { self.recv_byte() }

    /// Try to write a byte into the Tx buffer. Returns `nb::WouldBlock` if the buffer is still full. Returns immediately.
    #[inline(always)]
    pub fn write(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        // UCTXIFG = 1: UCxTXBUF can take another character (SLAU445I 23.3.8.1, p. 611)
        if self.usci.transmit_flag() {
            self.usci.txbuf_wr(byte);
            Ok(())
        } else {
            Err(WouldBlock)
        }
    }

    /// Reset the eUSCI: set and clear UCSWRST, which ends a transfer in progress (SLAU445I 23.3.5,
    /// p. 609), sets UCTXIFG and clears UCRXIFG, UCOE and UCFE (SLAU445I 23.3.1, p. 606). The interrupt
    /// enables, which the reset clears too, are kept.
    ///
    /// This is the last workaround of erratum USCI47 (MSP430FR2x5x, MSP430FR2433, MSP430FR25x2), for an
    /// eUSCI_A slave with UCCKPH = 1 that left reset while SCLK wasn't idle, and since then receives
    /// nothing: "If UCTXIFG is set twice but UCRXIFG is not set, reset the MSP SPI slave by setting and
    /// then clearing the UCSWRST bit, and inform the SPI master to resend the data" (SLAZ695J USCI47,
    /// p. 12 to p. 13; SLAZ664S USCI47, p. 14; SLAZ705H USCI47, p. 10 to p. 11). Reset it while SCLK is
    /// idle, or the reset meets the erratum's condition again; see [`SpiConfig::to_slave`].
    #[inline]
    pub fn reset(&mut self) {
        self.reset_keeping_interrupts();
    }
}

/// SPI transmit/receive errors
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
pub enum SpiErr {
    /// Data in the recieve buffer was overwritten before it was read. The contained data is the new contents of the recieve buffer.
    /// (UCOE, SLAU445I Table 23-5, p. 615; SLAU445I Table 23-14, p. 622.)
    Overrun(u8),
    /// On a bus set up with [`multi_master_bus()`](SpiConfig::multi_master_bus), another master took the
    /// bus during a transfer, which aborted it: "the data must be rewritten" (UCFE: SLAU445I 23.3.3.1,
    /// p. 608). Repeat the transaction; its writes wait until this master may use the bus again.
    BusConflict,
}
impl From<Infallible> for SpiErr {
    fn from(value: Infallible) -> Self { match value {} }
}

mod ehal1 {
    use super::*;
    use embedded_hal::spi::{Error, ErrorType, SpiBus};
    use nb::block;

    impl Error for SpiErr {
        fn kind(&self) -> embedded_hal::spi::ErrorKind {
            match self {
                SpiErr::Overrun(_) => embedded_hal::spi::ErrorKind::Overrun,
                SpiErr::BusConflict => embedded_hal::spi::ErrorKind::ModeFault,
            }
        }
    }

    impl<USCI, M> ErrorType for Spi<USCI, M>
    where
        USCI: SpiUsci<M>,
        M: PinMap,
    {
        type Error = SpiErr;
    }

    impl<USCI, M> SpiBus for Spi<USCI, M>
    where
        USCI: SpiUsci<M>,
        M: PinMap,
    {
        /// Send dummy packets (`0x00`) on MOSI so the slave can respond on MISO. Store the response in `words`.
        /// (A master receives only while it transmits: SLAU445I 23.3.3, p. 607.)
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            for word in words {
                block!(self.send_byte(0x00))?;
                *word = block!(self.recv_byte())?;
            }
            Ok(())
        }

        /// Write `words` to the slave, ignoring all the incoming words.
        ///
        /// Returns once the last word has been sent: the incoming word is waited for after each one, and
        /// UCRXIFG is set when "the RX or TX operation is complete" (SLAU445I 23.3.3, p. 607). Only a
        /// [`SpiErr::BusConflict`] is returned, as it means a word wasn't sent.
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            for word in words {
                block!(self.send_byte(*word))?;
                if let Err(SpiErr::BusConflict) = block!(self.recv_byte()) {
                    return Err(SpiErr::BusConflict);
                }
            }
            Ok(())
        }

        /// Write and read simultaneously. `write` is written to the slave on MOSI and
        /// words received on MISO are stored in `read`.
        ///
        /// If `write` is longer than `read`, then after `read` is full any subsequent incoming words will be discarded.
        /// If `read` is longer than `write`, then dummy packets of `0x00` are sent until `read` is full.
        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            let mut read_bytes = read.iter_mut();
            let mut write_bytes = write.iter();
            const DUMMY_WRITE: u8 = 0x00;
            let mut dummy_read = 0;

            // Pair up read and write bytes (inserting dummy values as necessary) until everything's sent
            loop {
                let (rd, wr) = match (read_bytes.next(), write_bytes.next()) {
                    (Some(rd), Some(wr)) => (rd, wr),
                    (Some(rd), None    ) => (rd, &DUMMY_WRITE),
                    (None,     Some(wr)) => (&mut dummy_read, wr),
                    (None,     None    ) => break,
                };

                block!(self.send_byte(*wr))?;
                *rd = block!(self.recv_byte())?;
            }
            Ok(())
        }

        /// Write and read simultaneously. The contents of `words` are
        /// written to the slave, and the received words are stored into the same
        /// `words` buffer, overwriting it.
        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            for word in words {
                block!(self.send_byte(*word))?;
                *word = block!(self.recv_byte())?;
            }
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            // UCBUSY: "A transmit or receive operation is indicated by UCBUSY = 1" (SLAU445I 23.3.5, p. 609)
            while self.usci.is_busy() {}
            Ok(())
        }
    }
}

mod ehal_nb1 {
    use super::*;
    use embedded_hal_nb::{nb, spi::FullDuplex};

    impl<USCI, M> FullDuplex<u8> for Spi<USCI, M>
    where
        USCI: SpiUsci<M>,
        M: PinMap,
    {
        fn read(&mut self) -> nb::Result<u8, Self::Error> { self.recv_byte() }

        fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            self.send_byte(word).map_err(map_infallible)
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::spi::FullDuplex;

    impl<USCI, M> FullDuplex<u8> for Spi<USCI, M>
    where
        USCI: SpiUsci<M>,
        M: PinMap,
    {
        type Error = SpiErr;
        fn read(&mut self) -> nb::Result<u8, Self::Error> { self.recv_byte() }

        fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            self.send_byte(word).map_err(map_infallible)
        }
    }

    // Implementing FullDuplex above gets us a blocking write and transfer implementation for free
    impl<USCI, M> embedded_hal_02::blocking::spi::write::Default<u8> for Spi<USCI, M>
    where
        USCI: SpiUsci<M>,
        M: PinMap,
    {}
    impl<USCI, M> embedded_hal_02::blocking::spi::transfer::Default<u8> for Spi<USCI, M>
    where
        USCI: SpiUsci<M>,
        M: PinMap,
    {}
}

// Unfortunately the compiler can't always automatically infer this, even though we already have From<Infallible> for SpiErr
fn map_infallible<E>(err: nb::Error<Infallible>) -> nb::Error<E> {
    match err {
        WouldBlock => WouldBlock,
    }
}
