//! I2C
//!
//! Peripherals eUSCI_B0 and eUSCI_B1 can be used for I2C communication (SLAU445I 24.1, p. 627).
//!
//! Begin by calling [`I2cConfig::new()`]. Depending on configuration, one of [`I2cSlave`], [`I2cSingleMaster`], [`I2cMultiMaster`],
//! or [`I2cMasterSlave`] will be returned.
//!
//! [`I2cSlave`] acts as a slave device on the bus. If the MSP430 is to be the only master on the bus then [`I2cSingleMaster`]
//! offers simplified error handling. If more than one master is on the bus then [`I2cMultiMaster`] should be used instead.
//! [`I2cMasterSlave`] offers a multi-role implementation that can act as a master but automatically downgrades to a slave
//! upon being addressed by another device.
//!
//! In all modes interrupts can be set and cleared using the `set_interrupts()` and `clear_interrupts()` methods alongside
//! [`I2cInterruptFlags`], which provides a user-friendly way to set the register flags.
//!
//! ## [`I2cSlave`]
//! In slave mode the peripheral responds to requests from master devices. The 'own address' is treated as 7-bit should a `u8`
//! be provided, and 10-bit if a `u16` is provided (UCA10, SLAU445I Table 24-4, p. 649).
//! Both polling and interrupt-based methods are available, though interrupt-based is recommended for slave devices, as the slave
//! can 'fall behind' and lose information if polling is not done frequently enough.
//!
//! The interrupt-based interface relies on using [`interrupt_source()`](I2cRoleCommon::interrupt_source()) to determine which event
//! caused the interrupt. The polling-based implementation instead uses calls to [`poll()`](I2cRoleSlave::poll()) to listen for events.
//! In either case methods such as [`write_tx_buf()`](I2cSlave::write_tx_buf()) and
//! [`read_rx_buf()`](I2cSlave::read_rx_buf()) can be used to respond accordingly.
//!
//! ## [`I2cSingleMaster`]
//! Single master mode provides simplified error handling and ergonomics at the cost of being unsuitable for buses with more than one
//! master - single master mode does not handle bus arbitration, so even if the device is not expected to be addressed as a slave it is not
//! suitable for use on a multi-master bus (UCMM = 0 means "There is no other master in the system",
//! SLAU445I Table 24-4, p. 649).
//!
//! An easy-to-use blocking implementation is available through [`embedded_hal::i2c::I2c`], which provides methods for read, write,
//! write-read, and generic transactions. Additionally, slave detection is provided through [`is_slave_present()`](I2cRoleMaster::is_slave_present()).
//!
//! A non-blocking or interrupt-based implementation is possible using [`I2cSingleMaster::send_start()`],
//! [`write_tx_buf()`](I2cSingleMaster::write_tx_buf), [`read_rx_buf()`](I2cSingleMaster::read_rx_buf),
//! [`tx_buf_empty()`](I2cRoleMaster::tx_buf_empty), [`schedule_stop()`](I2cRoleMaster::schedule_stop) and
//! [`stop_sent()`](I2cRoleMaster::stop_sent).
//!
//! ## [`I2cMultiMaster`]
//! [`I2cMultiMaster`] acts similarly to [`I2cSingleMaster`], but with the addition of bus arbitration logic.
//! The MSP430 hardware automatically fails over from master to slave mode when arbitration is lost
//! (SLAU445I 24.3.5.3, p. 641), so the methods check for this before performing operations. After losing
//! arbitration [`return_to_master()`](I2cRoleMulti::return_to_master) must be called.
//!
//! ## [`I2cMasterSlave`]
//! [`I2cMasterSlave`] can act as either a master or slave device. It is multi-master capable by necessity.
//! It broadly combines the functionality of [`I2cSlave`] and [`I2cMultiMaster`], providing a blocking master implementation via
//! [`embedded_hal::i2c::I2c`], and a non-blocking or interrupt-based interface via methods similar to [`I2cMultiMaster`]:
//! [`I2cMasterSlave::send_start()`], [`write_tx_buf_as_master()`](I2cMasterSlave::write_tx_buf_as_master),
//! [`read_rx_buf_as_master()`](I2cMasterSlave::read_rx_buf_as_master),
//! [`tx_buf_empty()`](I2cRoleMaster::tx_buf_empty), [`schedule_stop()`](I2cRoleMaster::schedule_stop) and
//! [`stop_sent()`](I2cRoleMaster::stop_sent).
//!
//! The MSP430 hardware automatically fails over from master to slave mode when arbitration is lost or the device is
//! addressed as a slave (UCALIFG, SLAU445I Table 24-2, p. 646), so the master-related methods check for this
//! before attempting master-related operations, returning an error if so.
//! The device can be restored to master mode via [`return_to_master()`](I2cRoleMulti::return_to_master). If arbitration is lost this
//! method may be called immediately, however if the device is addressed as a slave then this slave transaction must be resolved
//! before the device can be returned to master mode. The arbitration lost flag, UCALIFG, stays set until
//! then, and `return_to_master()` clears it.
//!
//! The device's own STOP as a master sets the same flag as the STOP that ends a slave transaction (UCSTPIFG,
//! SLAU445I Table 24-2, p. 646). The blocking methods clear it once the STOP is on the bus, as
//! [`stop_sent()`](I2cRoleMaster::stop_sent) does in the non-blocking interface, and in master mode
//! [`poll()`](I2cRoleSlave::poll) reports no events.
//!
//! The slave interface is much the same as what is provided by [`I2cSlave`]: Bus events can be discovered using
//! [`interrupt_source()`](I2cRoleCommon::interrupt_source()) for an interrupt-based implementation, or [`poll()`](I2cRoleSlave::poll())
//! for a polling-based one. [`write_tx_buf_as_slave()`](I2cMasterSlave::write_tx_buf_as_slave) and
//! [`read_rx_buf_as_slave()`](I2cMasterSlave::read_rx_buf_as_slave) allow for writing to the Rx and Tx buffers. These methods don't have the
//! bus arbitration and slave addressing checks that the `_as_master` variants do, so these should only be called in slave mode.
//!
//! Pins used (pins with `RemappedMapping` in brackets). The external clock pin can optionally clock the bus in
//! master modes: it's UCLKI, "the eUSCI_B SPI clock input pin" (SLAU445I Figure 24-1, p. 628), selected with
//! UCSSELx = 00b, which slave mode ignores (SLAU445I Table 24-4, p. 649).
//!
//! | Device       | eUSCI | SCL             | SDA             | External clock  |
//! |:-------------|:-----:|:---------------:|:---------------:|:---------------:|
//! | MSP430FR2x5x | B0    | `P1.3`          | `P1.2`          | `P1.1`          |
//! | MSP430FR2x5x | B1    | `P4.7`          | `P4.6`          | `P4.5`          |
//! | MSP430FR2433 | B0    | `P1.3`          | `P1.2`          | `P1.1`          |
//! | MSP430FR247x | B0    | `P1.3` (`P4.5`) | `P1.2` (`P4.6`) | `P1.1` (`P5.5`) |
//! | MSP430FR247x | B1    | `P3.6` (`P4.3`) | `P3.2` (`P4.4`) | `P3.5` (`P5.3`) |
//! | MSP430FR25x2 | B0    | `P1.3` (`P2.6`) | `P1.2` (`P2.5`) | `P1.1` (`P2.4`) |
//!
//! The pins are those of the eUSCI pin configuration tables: SLASEC4D Table 6-14, p. 72 (MSP430FR2x5x),
//! SLASE59F Table 6-10, p. 49 (MSP430FR2433), SLASEO7C Table 9-11, p. 54 (MSP430FR247x) and
//! SLASEE4C Table 6-11, p. 53 (MSP430FR25x2). The external clock pin is the one listed as SCLK in their
//! SPI column.
//!

use core::convert::Infallible;

#[cfg(feature = "eusci_aclk")]
use crate::clock::Aclk;
use crate::clock::Smclk;
use crate::hw_traits::eusci::{
    EUsciI2C, I2CUcbIfgOut, UcbCtlw0, UcbCtlw1, UcbI2coa, Ucmode, Ucssel,
};
use crate::pin_mapping::*;

use core::marker::PhantomData;
use embedded_hal::i2c::{AddressMode, SevenBitAddress, TenBitAddress};
use msp430::asm;
use nb::Error::{Other, WouldBlock};

/// Enumerates the two I2C addressing modes: 7-bit and 10-bit.
///
/// Used internally by the HAL. The values are those of UCA10 and UCSLA10 (SLAU445I Table 24-4, p. 649).
#[derive(Clone, Copy)]
pub enum AddressingMode {
    /// 7-bit addressing mode
    SevenBit = 0,
    /// 10-bit addressing mode
    TenBit = 1,
}
impl From<AddressingMode> for bool {
    #[inline(always)]
    fn from(f: AddressingMode) -> bool {
        match f {
            AddressingMode::SevenBit => false,
            AddressingMode::TenBit => true,
        }
    }
}

/// I2C transmission modes. The values are those of UCTR (SLAU445I Table 24-4, p. 650).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransmissionMode {
    /// Receiver mode
    Receive = 0,
    /// Transmitter mode
    Transmit = 1,
}
impl From<TransmissionMode> for bool {
    #[inline(always)]
    fn from(f: TransmissionMode) -> bool {
        match f {
            TransmissionMode::Receive => false,
            TransmissionMode::Transmit => true,
        }
    }
}

// The UCGLITx deglitch time of SDA and SCL (SLAU445I Table 24-1, p. 642)
pub use crate::hw_traits::eusci::Ucglit as GlitchFilter;

/// How long SCL may be held low before the clock low timeout flag is set (UCCLTO), counted in MODCLK cycles
/// (SLAU445I 24.3.7.3, p. 643; SLAU445I Table 24-5, p. 651).
///
/// The cycle counts and times are the user's guide's. The time depends on MODCLK: the data sheets give
/// typical times (tTIMEOUT) of 27, 30 and 33 ms (SLASE59F Table 5-19, p. 34; SLASEO7C 8.12.7.6, p. 39;
/// SLASEE4C Table 5-19, p. 37), and of 36, 40 and 44 ms on the MSP430FR2x5x (SLASEC4D Table 5-19, p. 50).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum ClockLowTimeout {
    /// No timeout, as after reset
    #[default]
    Disabled,
    /// 135000 MODCLK cycles, about 28 ms
    _28ms,
    /// 150000 MODCLK cycles, about 31 ms
    _31ms,
    /// 165000 MODCLK cycles, about 34 ms
    _34ms,
}

/// One of the three additional own addresses of a slave, UCBxI2COA1 to UCBxI2COA3.
///
/// Each own address has its own receive and transmit flags (user's guide, multiple slave addresses:
/// SLAU445I 24.3.9.1, p. 644).
/// [`poll()`](I2cRoleSlave::poll), [`read_rx_buf()`](I2cSlave::read_rx_buf) and
/// [`write_tx_buf()`](I2cSlave::write_tx_buf) only check those of the first one, so serve the others with
/// [`interrupt_source()`](I2cRoleCommon::interrupt_source) (`Slave1RxBufFull` to `Slave3TxBufEmpty`) and the
/// `_unchecked` buffer methods.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OwnAddressSlot {
    /// UCBxI2COA1
    _1,
    /// UCBxI2COA2
    _2,
    /// UCBxI2COA3
    _3,
}

///Struct used to configure a I2C bus
pub struct I2cConfig<USCI, CLKSRC, ROLE, M: PinMap = DefaultMapping>
where USCI: I2cUsci<M>
{
    usci: USCI,
    divisor: u16,

    // Register configs
    ctlw0: UcbCtlw0,
    ctlw1: UcbCtlw1,
    i2coa0: UcbI2coa,
    i2coa1: UcbI2coa,
    i2coa2: UcbI2coa,
    i2coa3: UcbI2coa,
    addmask: u16,
    tbcnt: u8,
    clk_src: PhantomData<CLKSRC>,
    role: PhantomData<ROLE>,
    _pin_map: PhantomData<M>,
}

/// Marks a usci capable of I2C communication
pub trait I2cUsci<M: PinMap = DefaultMapping>: EUsciI2C {
    /// I2C SCL pin
    type ClockPin;
    /// I2C SDA pin
    type DataPin;
    /// I2C external clock source pin. Only necessary if UCLKI is selected as a clock source (UCLKI is the
    /// eUSCI_B SPI clock pin: SLAU445I Figure 24-1, p. 628).
    type ExternalClockPin;

    /// Additional configuration
    #[inline(always)]
    fn configure_pin_mapping() {}
}

// Allows a GPIO pin to be converted into an I2C object
// The pin's alternate function defaults to Alternate1: PxSEL = 01b, the primary module function
// (SLAU445I Table 8-3, p. 314). The device_specific files cite each pin's function in its data sheet.
macro_rules! impl_i2c_pin {
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl_i2c_pin!($struct_name, $port, $pin, Alternate1);
    };
    ($struct_name: ident, $port: ty, $pin: ty, $alt: ident) => {
        impl<DIR> From<Pin<$port, $pin, $alt<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, $alt<DIR>>) -> Self { $struct_name }
        }
    };
}
pub(crate) use impl_i2c_pin;

/// Typestate for an I2C bus configuration with no clock source selected
pub struct NoClockSet;
/// Typestate for an I2C bus configuration with a clock source selected
pub struct ClockSet;

/// Typestate for an I2C bus that has not yet been assigned a role.
pub struct NoRoleSet;

/// Marker trait for typestates that correspond to I2C bus roles.
pub trait I2cMarker {}
/// Typestate for an I2C bus being configured as a master on a bus with no other master devices present.
pub struct SingleMaster;
impl I2cMarker for SingleMaster {}

/// Typestate for an I2C bus being configured as a slave.
pub struct Slave;
impl I2cMarker for Slave {}

/// Typestate for an I2C bus being configured as a master on a bus that has other master devices present.
pub struct MultiMaster;
impl I2cMarker for MultiMaster {}

/// Typestate for an I2C bus being configured as a master on a bus that has other master devices present that may address this device.
pub struct MasterSlave;
impl I2cMarker for MasterSlave {}

/// The smallest clock divider for a master role: the bit clock can be at most BRCLK/4 for a single master, and BRCLK/8
/// with several masters on the bus (user's guide, I2C clock generation: SLAU445I 24.3.7, p. 642)
trait MinClkDivisor {
    const MIN_CLK_DIVISOR: u16;
}
impl MinClkDivisor for SingleMaster {
    const MIN_CLK_DIVISOR: u16 = 4;
}
impl MinClkDivisor for MultiMaster {
    const MIN_CLK_DIVISOR: u16 = 8;
}
impl MinClkDivisor for MasterSlave {
    const MIN_CLK_DIVISOR: u16 = 8;
}

macro_rules! return_self_config {
    ($self: ident) => {
        I2cConfig {
            usci:    $self.usci,
            divisor: $self.divisor,
            ctlw0:   $self.ctlw0,
            ctlw1:   $self.ctlw1,
            i2coa0:  $self.i2coa0,
            i2coa1:  $self.i2coa1,
            i2coa2:  $self.i2coa2,
            i2coa3:  $self.i2coa3,
            addmask: $self.addmask,
            tbcnt:   $self.tbcnt,
            clk_src: PhantomData,
            role: PhantomData,
            _pin_map: PhantomData,
        }
    };
}

impl<USCI, M> I2cConfig<USCI, NoClockSet, NoRoleSet, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Begin configuration of an eUSCI peripheral as an I2C device.
    pub fn new(usci: USCI, deglitch_time: GlitchFilter) -> I2cConfig<USCI, NoClockSet, NoRoleSet, M> {
        // I2C mode is UCMODEx = 11b with UCSYNC = 1 (SLAU445I 24.3.1, p. 629; SLAU445I 24.3.5.1, p. 633), set
        // up while UCSWRST = 1
        let ctlw0 = UcbCtlw0 {
            ucsync: true,
            ucswrst: true,
            ucmode: Ucmode::I2CMode,
            ..Default::default()
        };

        let ctlw1 = UcbCtlw1 { ucglit: deglitch_time, ..Default::default() };

        let i2coa0 = UcbI2coa::default();
        let i2coa1 = UcbI2coa::default();
        let i2coa2 = UcbI2coa::default();
        let i2coa3 = UcbI2coa::default();

        I2cConfig {
            usci,
            divisor: 1,
            ctlw0,
            ctlw1,
            i2coa0,
            i2coa1,
            i2coa2,
            i2coa3,
            // All address bits compared, as after reset (03FFh, mask off: SLAU445I Table 24-16, p. 659)
            addmask: 0x03FF,
            tbcnt: 0,
            clk_src: PhantomData,
            role: PhantomData,
            _pin_map: PhantomData,
        }
    }
    /// Configure this eUSCI peripheral as an I2C master on a bus with no other master devices.
    /// (UCMST = 1 with UCMM = 0: SLAU445I 24.3.5.2, p. 636; SLAU445I Table 24-4, p. 649.)
    pub fn as_single_master(mut self) -> I2cConfig<USCI, NoClockSet, SingleMaster, M> {
        self.ctlw0.ucmst = true;

        return_self_config!(self)
    }

    /// Configure this eUSCI peripheral as an I2C slave.
    /// (UCMST = 0, the own address in UCBxI2COA0 and its size in UCA10: SLAU445I 24.3.5.1, p. 633.)
    pub fn as_slave<TenOrSevenBit>(
        mut self,
        own_address: TenOrSevenBit,
    ) -> I2cConfig<USCI, ClockSet, Slave, M>
    where
        TenOrSevenBit: AddressType,
    {
        self.ctlw0.uca10 = TenOrSevenBit::addr_type().into();

        // UCOAEN enables the own address, UCGCEN the general call (SLAU445I Table 24-11, p. 656)
        self.i2coa0 = UcbI2coa {
            ucgcen: false, // Set by general_call()
            ucoaen: true,
            i2coa0: own_address.into(),
        };

        return_self_config!(self)
    }

    /// Configure this eUSCI peripheral as an I2C master on a bus with other master devices.
    /// (UCMST = 1 with UCMM = 1: SLAU445I 24.3.5.2, p. 636; SLAU445I Table 24-4, p. 649.)
    ///
    /// No own address is enabled (UCOAEN = 0, SLAU445I Table 24-11, p. 656), so this device can't be
    /// addressed as a slave, though the other masters may still contest the bus. The user's guide asks
    /// multi-master devices to program their own address (SLAU445I 24.3.5.2, p. 636); use
    /// [`as_master_slave`](Self::as_master_slave) for a device that other masters can address.
    pub fn as_multi_master(mut self) -> I2cConfig<USCI, NoClockSet, MultiMaster, M> {
        self.ctlw0 = UcbCtlw0 { ucmst: true, ucmm: true, ..self.ctlw0 };

        return_self_config!(self)
    }

    /// Configure this EUSCI peripheral as an I2C master-slave on a bus with other master devices.
    /// The other masters may contest the bus and/or address this device as a slave.
    /// (UCMM = 1 with the own address in UCBxI2COA0, as multi-master systems need: SLAU445I 24.3.5.2, p. 636.)
    pub fn as_master_slave<TenOrSevenBit>(
        mut self,
        own_address: TenOrSevenBit,
    ) -> I2cConfig<USCI, NoClockSet, MasterSlave, M>
    where TenOrSevenBit: AddressType,
    {
        self.ctlw0 = UcbCtlw0 {
            uca10: TenOrSevenBit::addr_type().into(),
            ucmst: true,
            ucmm: true,
            ..self.ctlw0
        };

        // Note: If you add support for the other 3 own addresses (or the mask) you will also have to upgrade the logic for checking
        // that the peripheral isn't addressing itself, i.e. I2cMasterSlaveErr::TriedAddressingSelf
        // (UCBxI2CSA = UCBxI2COAx is not allowed: SLAU445I 24.3.5.2, p. 636; the mask:
        // SLAU445I 24.3.9.2, p. 644)
        self.i2coa0 = UcbI2coa {
            ucgcen: false, // Set by general_call()
            ucoaen: true,
            i2coa0: own_address.into(),
        };

        return_self_config!(self)
    }
}

#[allow(private_bounds)]
impl<USCI, M, ROLE: I2cMarker + MinClkDivisor> I2cConfig<USCI, NoClockSet, ROLE, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    #[inline(always)]
    fn set_clock(&mut self, ucssel: Ucssel, clk_divisor: u16) {
        assert!(clk_divisor >= ROLE::MIN_CLK_DIVISOR, "I2C clock divisor too small for this role");
        self.ctlw0.ucssel = ucssel;
        self.divisor = clk_divisor;
    }

    /// Configures this peripheral to use SMCLK (UCSSELx = 10b, SLAU445I Table 24-4, p. 649)
    ///
    /// # Panics
    ///
    /// If `clk_divisor` is below the user's guide minimum (SLAU445I 24.3.7, p. 642): 4 for a single master,
    /// 8 with several masters.
    #[inline]
    pub fn use_smclk(
        mut self,
        _smclk: &Smclk,
        clk_divisor: u16,
    ) -> I2cConfig<USCI, ClockSet, ROLE, M> {
        self.set_clock(Ucssel::Smclk, clk_divisor);
        return_self_config!(self)
    }

    #[cfg(feature = "eusci_aclk")]
    /// Configures this peripheral to use ACLK (UCSSELx = 01b, which is ACLK on these devices:
    /// SLASEC4D Table 6-9, p. 68; SLASEO7C Table 9-8, p. 50; SLASEE4C Table 6-8, p. 49)
    ///
    /// # Panics
    ///
    /// If `clk_divisor` is below the user's guide minimum (SLAU445I 24.3.7, p. 642): 4 for a single master,
    /// 8 with several masters.
    #[inline]
    pub fn use_aclk(
        mut self,
        _aclk: &Aclk,
        clk_divisor: u16,
    ) -> I2cConfig<USCI, ClockSet, ROLE, M> {
        self.set_clock(Ucssel::DeviceSpecific, clk_divisor);
        return_self_config!(self)
    }

    #[cfg(feature = "eusci_modclk")]
    /// Configures this peripheral to use MODCLK (UCSSELx = 01b, which is MODCLK on the MSP430FR2433:
    /// SLASE59F Table 6-7, p. 46)
    ///
    /// This also sets MODOSCREQEN, so that MODCLK runs for the eUSCI whichever kind of request it makes
    /// (SLAU445I 3.2.15.1, p. 111; SLAU445I Table 3-12, p. 123).
    ///
    /// # Panics
    ///
    /// If `clk_divisor` is below the user's guide minimum (SLAU445I 24.3.7, p. 642): 4 for a single master,
    /// 8 with several masters.
    #[inline]
    pub fn use_modclk(mut self, clk_divisor: u16) -> I2cConfig<USCI, ClockSet, ROLE, M> {
        crate::clock::enable_modosc_conditional_requests();
        self.set_clock(Ucssel::DeviceSpecific, clk_divisor);
        return_self_config!(self)
    }

    /// Configures this peripheral to use UCLK (UCSSELx = 00b, UCLKI: SLAU445I Table 24-4, p. 649)
    ///
    /// # Panics
    ///
    /// If `clk_divisor` is below the user's guide minimum (SLAU445I 24.3.7, p. 642): 4 for a single master,
    /// 8 with several masters.
    #[inline]
    pub fn use_uclk<Pin>(
        mut self,
        _uclk: Pin,
        clk_divisor: u16,
    ) -> I2cConfig<USCI, ClockSet, ROLE, M>
    where Pin: Into<USCI::ExternalClockPin>,
    {
        self.set_clock(Ucssel::Uclk, clk_divisor);
        return_self_config!(self)
    }
}

#[allow(private_bounds)]
impl<USCI, M, RoleSet: I2cMarker> I2cConfig<USCI, ClockSet, RoleSet, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Performs hardware configuration
    #[inline]
    fn configure_regs(&self) {
        // Initialization procedure of SLAU445I 24.3.1, p. 629: the registers are written with UCSWRST = 1
        // ("Modify only when UCSWRST = 1": SLAU445I 24.4.1 to 24.4.13, p. 649 to p. 659)
        // 1. Set UCSWRST
        self.usci.ctw0_set_rst();

        // 2. Initialize the registers
        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.ctw1_wr(&self.ctlw1);
        self.usci.i2coa_wr(0, &self.i2coa0);
        self.usci.i2coa_wr(1, &self.i2coa1);
        self.usci.i2coa_wr(2, &self.i2coa2);
        self.usci.i2coa_wr(3, &self.i2coa3);
        self.usci.ie_wr(0);
        self.usci.ifg_rst();

        self.usci.brw_wr(self.divisor);
        self.usci.tbcnt_wr(self.tbcnt as u16);
        self.usci.addmask_wr(self.addmask);

        // 3. Configure ports: the caller passes the pins already in their eUSCI function, and the
        // remapping bits are set here
        USCI::configure_pin_mapping();

        // 4. Clear UCSWRST
        self.usci.ctw0_clear_rst();
    }
}

impl<USCI, CLKSRC, ROLE, M> I2cConfig<USCI, CLKSRC, ROLE, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Set the clock low timeout: if SCL is held low that long, the `ClockLowTimeout` interrupt flag is set
    /// (UCCLTO, SLAU445I 24.3.7.3, p. 643; SLAU445I Table 24-5, p. 651).
    pub fn clock_low_timeout(mut self, timeout: ClockLowTimeout) -> Self {
        use crate::hw_traits::eusci::Ucclto;
        self.ctlw1.ucclto = match timeout {
            ClockLowTimeout::Disabled => Ucclto::Ucclto00b,
            ClockLowTimeout::_28ms => Ucclto::Ucclto01b,
            ClockLowTimeout::_31ms => Ucclto::Ucclto10b,
            ClockLowTimeout::_34ms => Ucclto::Ucclto11b,
        };
        self
    }

}

impl<USCI, CLKSRC, M> I2cConfig<USCI, CLKSRC, SingleMaster, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Count data bytes: after `count` bytes the `ByteCounterZero` interrupt flag is set (UCASTPx = 01b,
    /// UCBxTBCNT: SLAU445I 24.3.8, p. 643; SLAU445I Table 24-5, p. 651).
    /// With `auto_stop`, the master then also sends the STOP condition itself (UCASTPx = 10b, SLAU445I
    /// 24.3.8.2, p. 644); only use that with the non-blocking interface and fixed-length transactions,
    /// without `schedule_stop()`.
    /// The count can only change while the eUSCI is configured (UCBxTBCNT: "Modify only when UCSWRST = 1",
    /// SLAU445I Table 24-8, p. 654).
    pub fn byte_counter(mut self, count: u8, auto_stop: bool) -> Self {
        use crate::hw_traits::eusci::Ucastp;
        self.ctlw1.ucastp = if auto_stop { Ucastp::Ucastp10b } else { Ucastp::Ucastp01b };
        self.tbcnt = count;
        self
    }
}

// The roles that can be in slave mode: addressed as a slave, or, with UCMM = 1, after losing arbitration
// ("the UCMST bit is automatically cleared and the module acts as slave", SLAU445I Table 24-4, p. 649). The
// automatic STOP isn't available to them (UCASTPx: "In slave mode, only settings 00b and 01b are available",
// SLAU445I Table 24-5, p. 651).
macro_rules! byte_counter_no_stop {
    ($($role: ty),+) => {$(
        impl<USCI, CLKSRC, M> I2cConfig<USCI, CLKSRC, $role, M>
        where
            USCI: I2cUsci<M>,
            M: PinMap,
        {
            /// Count data bytes: after `count` bytes the `ByteCounterZero` interrupt flag is set (UCASTPx = 01b,
            /// UCBxTBCNT: SLAU445I 24.3.8, p. 643; SLAU445I Table 24-5, p. 651). The automatic STOP isn't
            /// offered, because this role can be in slave mode, and "In slave mode, only settings 00b and 01b
            /// are available" (SLAU445I Table 24-5, p. 651).
            /// The count can only change while the eUSCI is configured (UCBxTBCNT: "Modify only when UCSWRST = 1",
            /// SLAU445I Table 24-8, p. 654).
            pub fn byte_counter(mut self, count: u8) -> Self {
                self.ctlw1.ucastp = crate::hw_traits::eusci::Ucastp::Ucastp01b;
                self.tbcnt = count;
                self
            }
        }
    )+};
}
byte_counter_no_stop!(Slave, MultiMaster, MasterSlave);

macro_rules! slave_config {
    ($role: ty) => {
        impl<USCI, CLKSRC, M> I2cConfig<USCI, CLKSRC, $role, M>
        where
            USCI: I2cUsci<M>,
            M: PinMap,
        {
            /// Also respond to the general call address, 0 (UCGCEN, SLAU445I Table 24-11, p. 656).
            pub fn general_call(mut self) -> Self {
                self.i2coa0.ucgcen = true;
                self
            }
        }
    };
}
slave_config!(Slave);
slave_config!(MasterSlave);

impl<USCI, CLKSRC, M> I2cConfig<USCI, CLKSRC, Slave, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Respond to another own address too (UCBxI2COA1 to UCBxI2COA3, SLAU445I 24.3.9.1, p. 644), in the same
    /// addressing mode (7 or 10 bits) as the first one (one UCA10 bit for all: SLAU445I Table 24-4, p. 649).
    /// Its data has flags of its own, see [`OwnAddressSlot`].
    pub fn own_address<TenOrSevenBit: AddressType>(mut self, slot: OwnAddressSlot, address: TenOrSevenBit) -> Self {
        let oa = UcbI2coa { ucgcen: false, ucoaen: true, i2coa0: address.into() };
        match slot {
            OwnAddressSlot::_1 => self.i2coa1 = oa,
            OwnAddressSlot::_2 => self.i2coa2 = oa,
            OwnAddressSlot::_3 => self.i2coa3 = oa,
        }
        self
    }

    /// Ignore the address bits that are 0 in `mask` when comparing a received address with the first own
    /// address (UCBxADDMASK, SLAU445I 24.3.9.2, p. 644; SLAU445I Table 24-16, p. 659).
    /// [`I2cRoleSlave::received_address`] tells which address was received.
    pub fn address_mask(mut self, mask: u16) -> Self {
        // ADDMASKx is 10 bits wide, and writing the field drops the others (SLAU445I Table 24-16, p. 659)
        self.addmask = mask;
        self
    }

    /// Acknowledge addresses matching through the address mask from software, with
    /// [`I2cRoleSlave::acknowledge_address`], instead of automatically (UCSWACK, SLAU445I 24.3.9.2, p. 644;
    /// SLAU445I Table 24-5, p. 651).
    pub fn software_address_ack(mut self) -> Self {
        self.ctlw1.ucswack = true;
        self
    }

    /// Request the Tx buffer (UCTXIFG0) at each START condition, before the address is known, to have the
    /// first byte ready earlier (UCETXINT, SLAU445I 24.3.11.2, p. 645). Only with the first own address: don't
    /// use [`own_address`](Self::own_address) (UCETXINT, SLAU445I Table 24-5, p. 651).
    pub fn early_tx_interrupt(mut self) -> Self {
        self.ctlw1.ucetxint = true;
        self
    }
}

macro_rules! master_config {
    ($role: ty) => {
        impl<USCI, CLKSRC, M> I2cConfig<USCI, CLKSRC, $role, M>
        where
            USCI: I2cUsci<M>,
            M: PinMap,
        {
            /// Acknowledge the last received byte as a master receiver too, instead of sending the NACK before the
            /// STOP that the I2C specification requires (UCSTPNACK, SLAU445I Table 24-5, p. 651). Only for
            /// slaves that release SDA after a fixed number of bytes.
            pub fn ack_last_byte(mut self) -> Self {
                self.ctlw1.ucstpnack = true;
                self
            }
        }
    };
}
master_config!(SingleMaster);
master_config!(MultiMaster);
master_config!(MasterSlave);

macro_rules! configure {
    ($role: ty, $out_type: path) => {
        impl<USCI, M> I2cConfig<USCI, ClockSet, $role, M>
        where
            USCI: I2cUsci<M>,
            M: PinMap,
        {
            /// Performs hardware configuration and creates the I2C bus
            #[inline(always)]
            pub fn configure<SCL, SDA>(self, _scl: SCL, _sda: SDA) -> $out_type
            where
                SCL: Into<USCI::ClockPin>,
                SDA: Into<USCI::DataPin>,
            {
                self.configure_regs();
                $out_type { usci: self.usci, _pin_map: PhantomData }
            }
        }
    };
}

configure!(SingleMaster, I2cSingleMaster<USCI, M>);
configure!(MultiMaster,  I2cMultiMaster<USCI, M>);
configure!(Slave,        I2cSlave<USCI, M>);
configure!(MasterSlave,  I2cMasterSlave<USCI, M>);

mod sealed {
    use super::*;

    pub trait I2cRoleBase<M>
    where M: PinMap
    {
        type USCI: I2cUsci<M>;
        fn usci(&self) -> &Self::USCI;
    }

    pub trait I2cError {
        fn nack(variant: NackType) -> Self;
        fn nack_type(&self) -> Option<NackType>;
    }

    /// Internal methods common to all I2C roles capable of master operations
    pub trait I2cRoleMasterPrivate<M>: I2cRoleBase<M>
    where M: PinMap
    {
        type ErrorType: I2cError;
        fn set_addressing_mode(&mut self, mode: AddressingMode) {
            // UCSLA10, the size of the slave address (SLAU445I Table 24-4, p. 649)
            self.usci().set_ucsla10(mode.into())
        }

        /// Send a START, or a repeated START (UCTXSTT: SLAU445I 24.3.5.2.1, p. 637; SLAU445I 24.3.5.2.2,
        /// p. 639). The flags of earlier transactions are cleared first, as the eUSCI doesn't clear them
        /// (SLAU445I 24.3.11, p. 645): the START discards UCBxTXBUF and then sets UCTXIFG0 (SLAU445I
        /// Figure 24-12, p. 638), so a byte written on an old UCTXIFG0 would be lost, and an old UCSTPIFG
        /// would end the wait for this transaction's STOP. The slave's flags and UCALIFG stay
        /// (`EUsciI2C::clear_master_flags`).
        fn generate_start(&mut self) {
            self.usci().clear_master_flags();
            self.usci().transmit_start();
        }

        /// Whether this transaction's STOP is on the bus. "UCTXSTP is automatically cleared after STOP is
        /// generated" (SLAU445I Table 24-4, p. 650), and the next START has to wait for that (SLAU445I
        /// 24.3.5.2.2, p. 639). The STOP then sets UCSTPIFG ("If a STOP condition was generated by the
        /// eUSCI_B module, the UCSTPIFG is set": SLAU445I 24.3.5.2.2, p. 639) and clears UCBBUSY (SLAU445I
        /// 24.3.2, p. 630). UCBBUSY shows it when an interrupt handler has cleared UCSTPIFG already, by
        /// reading the vector (SLAU445I 24.3.11.5, p. 646).
        fn stop_done(&self, ifg: &<Self::USCI as EUsciI2C>::IfgOut) -> bool {
            !self.usci().uctxstp_rd() && (ifg.ucstpifg() || !self.usci().is_bus_busy())
        }

        /// Wait until the STOP of this transaction is on the bus (`stop_done`), then clear the
        /// transaction's flags: UCSTPIFG is the flag of a slave's STOP too (SLAU445I Table 24-2, p. 646), so
        /// a master-slave's `poll()` would report it later. A NACK discards the STOP instead ("Any set
        /// UCTXSTT or UCTXSTP is also discarded": SLAU445I 24.3.5.2.1, p. 637), and `handle_errs` sends it
        /// then.
        fn wait_for_stop(&mut self, idx: usize) -> Result<(), Self::ErrorType> {
            loop {
                let ifg = self.usci().ifg_rd();
                self.handle_errs(&ifg, idx)?;
                if self.stop_done(&ifg) {
                    break;
                }
            }
            self.usci().clear_master_flags_keep_rx();
            Ok(())
        }

        /// After a NACK "The master must react with either a STOP condition or a repeated START condition"
        /// (SLAU445I 24.3.5.2.1, p. 637): send the STOP, wait until it's on the bus, and clear the
        /// transaction's flags, the NACK's too (as `wait_for_stop`).
        fn stop_after_nack(&mut self) {
            self.usci().transmit_stop();
            while !self.stop_done(&self.usci().ifg_rd()) {
                asm::nop();
            }
            self.usci().clear_master_flags_keep_rx();
        }

        fn blocking_read_unchecked(
            &mut self,
            address: u16,
            buffer: &mut [u8],
            send_start: bool,
            send_stop: bool,
        ) -> Result<(), Self::ErrorType> {
            // Hardware doesn't support zero byte reads: a master receiver sends its STOP after NACKing a
            // received byte (SLAU445I 24.3.5.2.2, p. 639; UCTXSTP, SLAU445I Table 24-4, p. 650).
            if buffer.is_empty() { return Ok(()) }

            // Master receiver: UCBxI2CSA and UCTR = 0, then UCTXSTT (SLAU445I 24.3.5.2.2, p. 639)
            self.usci().i2csa_wr(address);
            self.usci().set_uctr(TransmissionMode::Receive.into());

            if send_start {
                self.generate_start();
                // Wait for initial address byte and (N)ACK to complete. ("The UCTXSTT flag is cleared as
                // soon as the complete address is sent", SLAU445I 24.3.5.2.2, p. 639.) A lost arbitration
                // ends the wait too.
                while self.usci().uctxstt_rd() {
                    self.handle_errs(&self.usci().ifg_rd(), 0)?;
                }
            }

            let len = buffer.len();
            for (idx, byte) in buffer.iter_mut().enumerate() {
                // "The next byte received from the slave is followed by a NACK and a STOP condition"
                // (SLAU445I 24.3.5.2.2, p. 639)
                if send_stop && (idx == len - 1) {
                    self.usci().transmit_stop();
                }
                loop {
                    let ifg = self.usci().ifg_rd();
                    self.handle_errs(&ifg, idx)?;
                    if ifg.ucrxifg0() {
                        break;
                    }
                }
                *byte = self.usci().ucrxbuf_rd();
            }

            if send_stop {
                self.wait_for_stop(len)?;
            }

            Ok(())
        }

        fn blocking_write_unchecked(
            &mut self,
            address: u16,
            bytes: &[u8],
            send_start: bool,
            send_stop: bool,
        ) -> Result<(), Self::ErrorType> {
            // Master transmitter: UCBxI2CSA and UCTR = 1, then UCTXSTT (SLAU445I 24.3.5.2.1, p. 637)
            self.usci().i2csa_wr(address);
            self.usci().set_uctr(TransmissionMode::Transmit.into());

            if bytes.is_empty() {
                return self.zero_byte_write();
            }

            if send_start {
                self.generate_start();
            }

            // UCTXIFG0 is set when the START is generated, so the first byte goes into the buffer before the
            // address is acknowledged (SLAU445I 24.3.5.2.1, p. 637). Without a START the write before left
            // UCTXIFG0 set, and the bus is held "until data is written into UCBxTXBUF" (SLAU445I 24.3.5.2.1,
            // p. 637).
            for (idx, &byte) in bytes.iter().enumerate() {
                loop {
                    let ifg = self.usci().ifg_rd();
                    // Subtract index because buffer fills before any NACKs come through
                    self.handle_errs(&ifg, idx.saturating_sub(1))?;
                    if ifg.uctxifg0() {
                        break;
                    }
                }
                self.usci().uctxbuf_wr(byte);
            }
            // UCTXIFG0 is set again as the last byte moves to the shift register (SLAU445I 24.3.5.2.1, p. 637)
            while !self.usci().ifg_rd().uctxifg0() {
                self.handle_errs(&self.usci().ifg_rd(), bytes.len().saturating_sub(1))?;
            }

            if send_stop {
                // The STOP follows the next acknowledge (SLAU445I 24.3.5.2.1, p. 637)
                self.usci().transmit_stop();
                self.wait_for_stop(bytes.len())?;
            }

            Ok(())
        }

        fn zero_byte_write(&mut self) -> Result<(), Self::ErrorType> {
            // To send only the address, set UCTXSTT and UCTXSTP at the same time (user's guide:
            // SLAU445I 24.3.8.2, p. 644), after clearing the flags of earlier transactions, as
            // `generate_start` does
            self.usci().clear_master_flags();
            self.usci().transmit_start_stop();
            // An earlier note here said that the bus stalls with nothing in Tx, even with a stop scheduled,
            // but SLAU445I 24.3.5.2.1, p. 637 says a STOP set while the eUSCI waits for data comes "even if
            // no data was transmitted", and with UCTXSTP set before the data starts "only the address is
            // transmitted", so this byte isn't sent.
            self.usci().uctxbuf_wr(0);
            // "In this case, the UCSTPIFG is set" (SLAU445I 24.3.5.2.1, p. 637)
            self.wait_for_stop(0)
        }

        #[inline]
        fn send_start_unchecked<SevenOrTenBit: AddressType>(
            &mut self,
            address: SevenOrTenBit,
            mode: TransmissionMode,
        ) {
            // UCSLA10, UCTR and UCBxI2CSA, then UCTXSTT (SLAU445I 24.3.5.2.1, p. 637;
            // SLAU445I 24.3.5.2.2, p. 639)
            self.set_addressing_mode(SevenOrTenBit::addr_type());
            self.usci().set_uctr(mode.into());
            self.usci().i2csa_wr(address.into());
            self.generate_start();
        }

        /// In multi-operation transactions update the NACK byte error count to match *total* bytes sent
        /// (wrapping, as in release builds: it only numbers the byte)
        #[inline]
        fn add_nack_count(err: Self::ErrorType, bytes_already_sent: usize) -> Self::ErrorType {
            let total = |n: usize| n.wrapping_add(bytes_already_sent);
            match err.nack_type() {
                None => err,
                Some(NackType::Address(n)) => Self::ErrorType::nack(NackType::Address(total(n))),
                Some(NackType::Data(n))    => Self::ErrorType::nack(NackType::Data(total(n))),
            }
        }

        // The blocking transfers clear only the flags of their own transaction, at its START and after its
        // STOP: after a lost arbitration the flags are the slave role's (UCSTTIFG, UCRXIFG0, UCTXIFG0:
        // SLAU445I Figure 24-12, p. 638; SLAU445I Figure 24-13, p. 640), and `return_to_master()` clears
        // UCALIFG.
        #[inline]
        fn blocking_write(
            &mut self,
            address: u16,
            bytes: &[u8],
            send_start: bool,
            send_stop: bool,
        ) -> Result<(), Self::ErrorType> {
            self.can_proceed(address)?;
            self.blocking_write_unchecked(address, bytes, send_start, send_stop)
        }

        #[inline]
        fn blocking_read(
            &mut self,
            address: u16,
            buffer: &mut [u8],
            send_start: bool,
            send_stop: bool,
        ) -> Result<(), Self::ErrorType> {
            self.can_proceed(address)?;
            self.blocking_read_unchecked(address, buffer, send_start, send_stop)
        }

        /// blocking write then blocking read. A read without bytes puts nothing on the bus (see
        /// `blocking_read_unchecked`), so with an empty `buffer` the write sends the STOP.
        #[inline]
        fn blocking_write_read(
            &mut self,
            address: u16,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::ErrorType> {
            self.blocking_write(address, bytes, true, buffer.is_empty())?;
            self.blocking_read(address, buffer, true, true)
                .map_err(|e| Self::add_nack_count(e, bytes.len()))
        }

        /// The checks of the non-blocking master methods, before their buffer flag: a lost arbitration (see
        /// `arbitration_err`), then a NACK
        fn nb_errs(&mut self, ifg: &<Self::USCI as EUsciI2C>::IfgOut) -> nb::Result<(), Self::ErrorType> {
            if let Some(err) = self.arbitration_err(ifg) {
                return Err(Other(err));
            }
            if ifg.ucnackifg() {
                // The byte counter restarts at each START and skips address bytes (SLAU445I 24.3.8, p. 643)
                let nack_type = match self.usci().byte_count() {
                    0 => NackType::Address(0),
                    n => NackType::Data(n as usize),
                };
                return Err(Other(Self::ErrorType::nack(nack_type)));
            }
            Ok(())
        }

        fn mst_write_tx_buf(
            &mut self,
            byte: u8,
            ifg: &<Self::USCI as EUsciI2C>::IfgOut,
        ) -> nb::Result<(), Self::ErrorType> {
            self.nb_errs(ifg)?;
            // UCTXIFG0: the transmitter can take a new byte (SLAU445I 24.3.11.1, p. 645)
            if !ifg.uctxifg0() {
                return Err(WouldBlock);
            }
            self.usci().uctxbuf_wr(byte);
            Ok(())
        }

        fn mst_read_rx_buf(
            &mut self,
            ifg: &<Self::USCI as EUsciI2C>::IfgOut,
        ) -> nb::Result<u8, Self::ErrorType> {
            self.nb_errs(ifg)?;
            // UCRXIFG0: a byte was received into UCBxRXBUF (SLAU445I 24.3.11.3, p. 645)
            if !ifg.ucrxifg0() {
                return Err(WouldBlock);
            }
            Ok(self.usci().ucrxbuf_rd())
        }

        /// Error handling during blocking read/writes: a NACK, which ends the transaction with a STOP, or a
        /// lost arbitration (see `arbitration_err`)
        fn handle_errs(
            &mut self,
            ifg: &<Self::USCI as EUsciI2C>::IfgOut,
            idx: usize,
        ) -> Result<(), Self::ErrorType> {
            if ifg.ucnackifg() {
                self.stop_after_nack();
                let nack = if idx == 0 { NackType::Address(idx) } else { NackType::Data(idx) };
                return Err(Self::ErrorType::nack(nack));
            }
            match self.arbitration_err(ifg) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }

        /// A lost arbitration, which only a role with other masters on the bus (UCMM = 1) can see: UCALIFG is
        /// set and UCMST cleared (SLAU445I Table 24-2, p. 646). The flags stay as they are, for the slave
        /// role and `return_to_master()`.
        fn arbitration_err(&self, ifg: &<Self::USCI as EUsciI2C>::IfgOut) -> Option<Self::ErrorType>;

        /// Whether a master operation can occur at the moment
        fn can_proceed(&mut self, _address: u16) -> Result<(), Self::ErrorType>;
    }

    /// Internal methods common to all I2C roles capable of slave operations
    pub trait I2cRoleSlavePrivate<M>: I2cRoleBase<M>
    where M: PinMap
    {
        #[inline]
        fn sl_write_tx_buf(&mut self, byte: u8) -> nb::Result<(), Infallible> {
            // UCTXIFG0 is the flag of the first own address (SLAU445I 24.3.11.1, p. 645)
            if !self.usci().ifg_rd().uctxifg0() {
                return Err(WouldBlock);
            }
            self.usci().uctxbuf_wr(byte);
            Ok(())
        }
        #[inline]
        fn sl_read_rx_buf(&mut self) -> nb::Result<u8, Infallible> {
            // UCRXIFG0 is the flag of the first own address (SLAU445I 24.3.11.3, p. 645)
            if !self.usci().ifg_rd().ucrxifg0() {
                return Err(WouldBlock);
            }
            Ok(self.usci().ucrxbuf_rd())
        }
    }
}
use sealed::*;

/// Common methods available to all I2C roles.
pub trait I2cRoleCommon<M>: I2cRoleBase<M>
where M: PinMap
{
    /// Get the number of bytes received/transmitted since the last Start or Repeated Start condition
    /// (UCBCNTx, SLAU445I Table 24-7, p. 653).
    #[inline(always)]
    fn byte_count(&mut self) -> u8 { self.usci().byte_count() }

    /// Get the event that triggered the current interrupt. Used as part of the interrupt-based interface.
    fn interrupt_source(&mut self) -> I2cVector { self.usci().iv_rd() }

    /// Set the bits in the interrupt enable register that correspond to the bits set in `intrs`
    /// (UCBxIE, SLAU445I Table 24-18, p. 660).
    #[inline(always)]
    fn set_interrupts(&mut self, intrs: I2cInterruptFlags) { self.usci().ie_set(intrs.bits()) }
    /// Clear the bits in the interrupt enable register that correspond to the bits *set* in `intrs`
    /// (UCBxIE, SLAU445I Table 24-18, p. 660).
    #[inline(always)]
    fn clear_interrupts(&mut self, intrs: I2cInterruptFlags) { self.usci().ie_clr(!(intrs.bits())) }
}

/// Common methods available to all I2C roles that can perform master operations.
pub trait I2cRoleMaster<M>: I2cRoleMasterPrivate<M>
where M: PinMap
{
    /// Manually schedule a stop condition to be sent. Used as part of the non-blocking interface.
    ///
    /// The stop will be sent after the current byte operation: after the next acknowledge as a transmitter,
    /// after the next byte, which is NACKed, as a receiver. If the bus is stalled waiting for the Tx buffer,
    /// the STOP is sent "even if no data was transmitted" (SLAU445I 24.3.5.2.1, p. 637); if it's stalled
    /// waiting for the Rx buffer to be read, the NACK "occurs immediately", followed by the STOP
    /// (SLAU445I 24.3.5.2.2, p. 639).
    ///
    /// As a transmitter, wait with [`tx_buf_empty()`](Self::tx_buf_empty) until the last byte has left the Tx
    /// buffer first: "When transmitting a single byte of data, the UCTXSTP bit must be set while the byte is
    /// being transmitted or any time after transmission begins, without writing new data into UCBxTXBUF.
    /// Otherwise, only the address is transmitted" (SLAU445I 24.3.5.2.1, p. 637). Then
    /// [`stop_sent()`](Self::stop_sent) tells when the transaction has ended.
    #[inline(always)]
    fn schedule_stop(&mut self) {
        // The flags aren't cleared automatically (SLAU445I 24.3.11, p. 645). UCSTPIFG is cleared first, so
        // that `stop_sent()` sees this STOP's own flag; UCTXIFG0, so that no byte goes into the Tx buffer
        // after the STOP is requested; and UCNACKIFG, which the STOP answers (SLAU445I 24.3.5.2.1, p. 637). A
        // byte already in UCBxRXBUF, the last one of a master receive say, stays readable, and the slave's
        // flags stay too (`EUsciI2C::clear_master_flags`).
        self.usci().clear_master_flags_keep_rx();
        self.usci().transmit_stop();
    }

    /// Check whether the Tx buffer is empty, without writing to it. Used as part of the non-blocking
    /// interface: after the last byte of a write, a STOP ([`schedule_stop()`](Self::schedule_stop)) or a
    /// repeated START (`send_start()`) has to wait until that byte has moved on to the shift register. "When
    /// the data is transferred from the buffer to the shift register, UCTXIFG0 is set, indicating data
    /// transmission has begun, and the UCTXSTP bit may be set", and data is only transmitted while "The
    /// UCTXSTT bit is not set" (SLAU445I 24.3.5.2.1, p. 637).
    ///
    /// Returns `Err(WouldBlock)` while the byte is still in the Tx buffer, and the errors of the non-blocking
    /// buffer methods: a NACK, and in the multi-master roles a lost arbitration.
    #[inline]
    fn tx_buf_empty(&mut self) -> nb::Result<(), Self::ErrorType> {
        let ifg = self.usci().ifg_rd();
        self.nb_errs(&ifg)?;
        // UCTXIFG0 (SLAU445I 24.3.11.1, p. 645)
        if !ifg.uctxifg0() {
            return Err(WouldBlock);
        }
        Ok(())
    }

    /// Check whether the STOP has been sent: the one of [`schedule_stop()`](Self::schedule_stop), or the
    /// automatic one of the byte counter. Used as part of the non-blocking interface, at the end of a
    /// transaction: "the current transaction must be completed before the next one is initiated", which
    /// UCTXSTP shows (SLAU445I 24.3.5.2.2, p. 639).
    ///
    /// The STOP sets the STOP flag, UCSTPIFG, of this eUSCI too (SLAU445I Table 24-2, p. 646). This clears
    /// it, with the transaction's other flags, so that a master-slave doesn't report this STOP later as the
    /// end of a slave transaction. UCRXIFG0 stays, so that a last received byte can still be read.
    ///
    /// Returns `Err(WouldBlock)` until then, and the errors of the non-blocking buffer methods: a NACK, and
    /// in the multi-master roles a lost arbitration. A NACK discards a set UCTXSTP ("Any set UCTXSTT or
    /// UCTXSTP is also discarded", SLAU445I 24.3.5.2.1, p. 637): call `schedule_stop()` again.
    #[inline]
    fn stop_sent(&mut self) -> nb::Result<(), Self::ErrorType> {
        let ifg = self.usci().ifg_rd();
        self.nb_errs(&ifg)?;
        if !self.stop_done(&ifg) {
            return Err(WouldBlock);
        }
        self.usci().clear_master_flags_keep_rx();
        Ok(())
    }

    /// Checks whether a slave with the specified address is present on the I2C bus.
    /// Sends a zero-byte write and records whether the slave sends an ACK or not (only the address is sent:
    /// SLAU445I 24.3.8.2, p. 644; a missing ACK sets UCNACKIFG: SLAU445I Table 24-2, p. 646).
    ///
    /// A `u8` address will use the 7-bit addressing mode, a `u16` address uses 10-bit addressing.
    #[inline]
    fn is_slave_present<TenOrSevenBit>(
        &mut self,
        address: TenOrSevenBit,
    ) -> Result<bool, Self::ErrorType>
    where TenOrSevenBit: AddressType,
    {
        self.set_addressing_mode(TenOrSevenBit::addr_type());
        match self.blocking_write(address.into(), &[], true, true) {
            Ok(_) => Ok(true),
            Err(e) if e.nack_type().is_some() => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Common methods available to all I2C roles that can perform slave operations.
pub trait I2cRoleSlave<M>: I2cRoleSlavePrivate<M>
where M: PinMap
{
    /// Returns whether the device is currently in receive mode or transmit mode.
    /// (UCTR, which a slave sets from the R/W bit it receives: SLAU445I 24.3.5.1, p. 633.)
    #[inline(always)]
    fn transmission_mode(&mut self) -> TransmissionMode {
        match self.usci().is_transmitter() {
            true  => TransmissionMode::Transmit,
            false => TransmissionMode::Receive,
        }
    }
    /// Check the I2C bus flags for any events that should be dealt with. Returns `Err(WouldBlock)` if no events have occurred yet, otherwise `Ok(I2cEvent)`.
    ///
    /// A master-slave in master mode isn't a slave, so it gets `Err(WouldBlock)` then: being addressed
    /// makes it a slave (SLAU445I Table 24-2, p. 646), and in master mode the flags are a master's, such as
    /// the STOP flag of its own STOP.
    fn poll(&mut self) -> nb::Result<I2cEvent, Infallible> {
        // UCMST (SLAU445I Table 24-4, p. 649)
        if self.usci().is_master() {
            return Err(WouldBlock);
        }
        // UCSTPIFG, UCSTTIFG, UCRXIFG0, UCTXIFG0 with UCTR (SLAU445I Table 24-2, p. 646;
        // SLAU445I Table 24-19, p. 662)
        if self.usci().stop_received() {
            self.usci().clear_start_stop_flags();
            return Ok(I2cEvent::Stop);
        }

        match (self.usci().start_received(), self.usci().rxifg0_rd(), self.usci().is_transmitter() & self.usci().txifg0_rd()) {
            (true,  true,  false) => {
                self.usci().clear_start_flag();
                Ok(I2cEvent::WriteStart)
            },
            (true,  false, true ) => {
                self.usci().clear_start_flag();
                Ok(I2cEvent::ReadStart)
            },
            (false, true,  false) => Ok(I2cEvent::Write),
            (false, false, true ) => Ok(I2cEvent::Read),
            // Rx buffer filled, then repeated start then Tx buffer empty. (Can't be reverse because empty Tx buf stalls the bus).
            // (A slave transmitter holds SCL low until its Tx buffer is written: SLAU445I 24.3.5.1.1, p. 633.)
            (true,  true,  true ) => Ok(I2cEvent::OverrunWrite), // Don't clear the start flag yet.
            // The same, with the start flag already cleared, by reading the interrupt vector say
            // (reading UCBxIV resets the highest pending flag: SLAU445I 24.3.11.5, p. 646)
            (false, true,  true ) => Ok(I2cEvent::OverrunWrite),
            // Start flag but no Rx / Tx events yet. Don't clear the flag yet.
            (_,     false, false) => Err(WouldBlock),
        }
    }

    /// Check whether the device is currently being addressed as a slave.
    /// (UCSTTIFG is set by a START "together with its own address": SLAU445I Table 24-2, p. 646.)
    #[inline(always)]
    fn is_being_addressed(&mut self) -> bool {
        !self.usci().is_master() && self.usci().ifg_rd().ucsttifg()
    }

    /// Queue a NACK to be sent on the I2C bus. If this is called in response to a packet being received the NACK will be sent on the following byte.
    /// (SLAU445I 24.3.5.1.2, p. 634: "during the next acknowledgment cycle".)
    ///
    /// Used as part of the non-blocking / interrupt-based interface. NACKs can only be sent as a slave receiver (user's
    /// guide, UCTXNACK: SLAU445I Table 24-4, p. 650), so only use this while receiving as a slave.
    #[inline(always)]
    fn send_nack(&mut self) { self.usci().transmit_nack(); }

    /// The address this device was last addressed with (UCBxADDRX, SLAU445I Table 24-15, p. 658), useful with
    /// several own addresses or an address mask.
    #[inline(always)]
    fn received_address(&mut self) -> u16 { self.usci().addrx_rd() }

    /// With [`software_address_ack`](I2cConfig::software_address_ack), acknowledge the received address or not,
    /// after the start flag is set (UCTXACK, SLAU445I Table 24-4, p. 649). SCL is held low until this is
    /// called. When not acknowledging a read, this also clears the Tx buffer flag, as the user's guide requires
    /// (SLAU445I 24.3.9.2, p. 644: "TXIFG0 must be reset").
    #[inline]
    fn acknowledge_address(&mut self, ack: bool) {
        if ack {
            self.usci().transmit_ack();
        } else {
            // Any write to the low byte of UCBxCTLW0 with UCTXACK = 0 continues without an ACK ("The clock is
            // stretched until the UCBxCTL1 register has been written", SLAU445I Table 24-4, p. 649)
            self.usci().clear_txack();
            self.usci().clear_txifg0();
        }
    }
}

/// Common methods available to all multi-master-aware I2C roles.
pub trait I2cRoleMulti<M>: I2cRoleMaster<M>
where M: PinMap
{
    /// Manually send a start condition and address byte. Used as part of the non-blocking interface.
    /// Passing a `u8` address uses 7-bit addressing, a `u16` address uses 10-bit addressing.
    ///
    /// It first clears the flags the transaction before left, as the START discards the Tx buffer and sets
    /// UCTXIFG0 again (SLAU445I Figure 24-12, p. 638): read a last received byte before calling it.
    #[inline]
    fn send_start<SevenOrTenBit: AddressType>(
        &mut self,
        address: SevenOrTenBit,
        mode: TransmissionMode,
    ) -> Result<(), Self::ErrorType> {
        self.can_proceed(address.into())?;
        self.send_start_unchecked(address, mode);
        Ok(())
    }

    /// After losing arbitration (or after being addressed as a slave) call this method to return the peripheral to master mode.
    /// (The eUSCI clears UCMST in both cases: UCALIFG, SLAU445I Table 24-2, p. 646.)
    ///
    /// This also clears UCALIFG, which the eUSCI doesn't clear itself (SLAU445I 24.3.11, p. 645), so that the
    /// master methods don't report the old arbitration loss again. A pending arbitration lost interrupt is
    /// also one of the conditions in which the eUSCI_B stretches SCL (SLAU445I 24.3.7.2, p. 643). Clearing
    /// UCALIFG resets UCTXIFGx too, and the next START sets UCTXIFG0 again (SLAU445I 24.3.11.1, p. 645).
    #[inline(always)]
    fn return_to_master(&mut self) {
        self.usci().clear_alifg();
        self.usci().set_master();
    }

    /// Check whether the device is currently in master mode (UCMST, SLAU445I Table 24-4, p. 649).
    #[inline(always)]
    fn is_master(&mut self) -> bool { self.usci().is_master() }
}

/// An eUSCI peripheral that has been configured as an I2C master.
/// This variant offers simplified error handling and ease of use, but is not suitable for use on a multi-master bus.
/// (UCMM = 0: "There is no other master in the system", SLAU445I Table 24-4, p. 649.)
pub struct I2cSingleMaster<USCI, M = DefaultMapping> {
    usci: USCI,
    _pin_map: PhantomData<M>,
}
impl<USCI, M> I2cRoleBase<M> for I2cSingleMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI { &self.usci }
}
impl<USCI, M> I2cRoleCommon<M> for I2cSingleMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleMasterPrivate<M> for I2cSingleMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type ErrorType = I2cSingleMasterErr;
    // No arbitration: with UCMM = 0 "There is no other master in the system" (SLAU445I Table 24-4, p. 649)
    fn arbitration_err(&self, _ifg: &<Self::USCI as EUsciI2C>::IfgOut) -> Option<Self::ErrorType> { None }

    fn can_proceed(&mut self, _address: u16) -> Result<(), Self::ErrorType> { Ok(()) }
}
impl<USCI, M> I2cRoleMaster<M> for I2cSingleMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cSingleMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Manually send a start condition and address byte. Used as part of the non-blocking interface.
    /// Passing a `u8` address uses 7-bit addressing, a `u16` address uses 10-bit addressing.
    ///
    /// It first clears the flags the transaction before left, as the START discards the Tx buffer and sets
    /// UCTXIFG0 again (SLAU445I Figure 24-12, p. 638): read a last received byte before calling it.
    #[inline(always)]
    pub fn send_start<SevenOrTenBit: AddressType>(
        &mut self,
        address: SevenOrTenBit,
        mode: TransmissionMode,
    ) {
        self.send_start_unchecked(address, mode);
    }

    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the Rx buffer is empty, `Err(GotNACK(n))` if a NACK was received from a previous byte
    /// (will prevent the Rx buffer from filling), where `n` is the number of
    /// bytes since the latest Start or Repeated Start condition (UCBCNTx, SLAU445I Table 24-7, p. 653).
    /// otherwise `Ok(n)`.
    #[inline(always)]
    pub fn read_rx_buf(&mut self) -> nb::Result<u8, I2cSingleMasterErr> {
        self.mst_read_rx_buf(&self.usci.ifg_rd())
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the Tx buffer is still full, `Err(GotNACK(n))` if a NACK was received from a previous byte
    /// (will prevent the Tx buffer from emptying), where `n` is the number of
    /// bytes since the latest Start or Repeated Start condition (UCBCNTx, SLAU445I Table 24-7, p. 653).
    /// Otherwise returns `Ok(())`.
    #[inline(always)]
    pub fn write_tx_buf(&mut self, byte: u8) -> nb::Result<(), I2cSingleMasterErr> {
        self.mst_write_tx_buf(byte, &self.usci.ifg_rd())
    }
}

/// An eUSCI peripheral that has been configured as an I2C multi-master.
/// Multi-masters are capable of sharing an I2C bus with other multi-masters, and may also optionally act as a slave device (depending on configuration).
pub struct I2cMultiMaster<USCI, M = DefaultMapping> {
    usci: USCI,
    _pin_map: PhantomData<M>,
}
impl<USCI, M> I2cRoleBase<M> for I2cMultiMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI { &self.usci }
}
impl<USCI, M> I2cRoleCommon<M> for I2cMultiMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleMasterPrivate<M> for I2cMultiMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type ErrorType = I2cMultiMasterErr;
    fn can_proceed(&mut self, _address: u16) -> Result<(), I2cMultiMasterErr> {
        // Multimaster doesn't need to check anything with the address, but it keeps the interface the same so we can abstract it
        // (UCMST is cleared when arbitration is lost: SLAU445I Table 24-4, p. 649)
        if !self.usci.is_master() {
            return Err(I2cMultiMasterErr::ArbitrationLost);
        }
        Ok(())
    }

    // UCALIFG (SLAU445I Table 24-2, p. 646)
    fn arbitration_err(&self, ifg: &USCI::IfgOut) -> Option<I2cMultiMasterErr> {
        if ifg.ucalifg() { Some(I2cMultiMasterErr::ArbitrationLost) } else { None }
    }
}
impl<USCI, M> I2cRoleMaster<M> for I2cMultiMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleMulti<M> for I2cMultiMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cMultiMaster<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is empty,
    /// `Err(Other(I2cMultiMasterErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(n)` if data was successfully retreived from the Rx buffer.
    #[inline]
    pub fn read_rx_buf(&mut self) -> nb::Result<u8, I2cMultiMasterErr> {
        // UCALIFG, UCNACKIFG and UCRXIFG0 (SLAU445I Table 24-19, p. 663)
        self.mst_read_rx_buf(&self.usci.ifg_rd())
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// First checks if the peripheral is still in master mode, if not returns an error.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is still full,
    /// `Err(Other(I2cMultiMasterErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(())` if data was successfully loaded into the Tx buffer.
    #[inline]
    pub fn write_tx_buf(&mut self, byte: u8) -> nb::Result<(), I2cMultiMasterErr> {
        // UCALIFG, UCNACKIFG and UCTXIFG0 (SLAU445I Table 24-19, p. 663)
        self.mst_write_tx_buf(byte, &self.usci.ifg_rd())
    }
}

/// An eUSCI peripheral that has been configured as an I2C slave.
pub struct I2cSlave<USCI, M = DefaultMapping> {
    usci: USCI,
    _pin_map: PhantomData<M>,
}
impl<USCI, M> I2cRoleBase<M> for I2cSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI { &self.usci }
}
impl<USCI, M> I2cRoleCommon<M> for I2cSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleSlavePrivate<M> for I2cSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleSlave<M> for I2cSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Read the Rx buffer without checking if it's ready.
    /// Useful in cases where you already know the Rx buffer is ready (e.g. an Rx interrupt occurred).
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then the data will be invalid (UCBxRXBUF holds "the last received
    /// character": SLAU445I Table 24-9, p. 655).
    #[inline(always)]
    pub unsafe fn read_rx_buf_unchecked(&mut self) -> u8 { self.usci.ucrxbuf_rd() }

    /// Write to the Tx buffer without checking if it's ready.
    /// Useful in cases where you already know the Tx buffer is ready (e.g. a Tx interrupt occurred).
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then previous data may be clobbered (UCBxTXBUF "holds the data waiting to be
    /// moved into the transmit shift register": SLAU445I Table 24-10, p. 655).
    #[inline(always)]
    pub unsafe fn write_tx_buf_unchecked(&mut self, byte: u8) { self.usci.uctxbuf_wr(byte); }

    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    /// Returns `Err(WouldBlock)` if the Rx buffer is empty, otherwise `Ok(n)`.
    #[inline(always)]
    pub fn read_rx_buf(&mut self) -> nb::Result<u8, Infallible> { self.sl_read_rx_buf() }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// Returns `Err(WouldBlock)` if the Tx buffer is still full, otherwise `Ok(())`.
    #[inline(always)]
    pub fn write_tx_buf(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        self.sl_write_tx_buf(byte)
    }
}

/// An eUSCI peripheral that has been configured as an I2C multi-master.
/// Multi-masters are capable of sharing an I2C bus with other multi-masters, and may also optionally act as a slave device (depending on configuration).
pub struct I2cMasterSlave<USCI, M = DefaultMapping> {
    usci: USCI,
    _pin_map: PhantomData<M>,
}
impl<USCI, M> I2cRoleBase<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI { &self.usci }
}
impl<USCI, M> I2cRoleCommon<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleMasterPrivate<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    type ErrorType = I2cMasterSlaveErr;
    fn can_proceed(&mut self, address: u16) -> Result<(), I2cMasterSlaveErr> {
        // Are we a master? If not, why? (Addressed after losing arbitration: UCALIFG with UCSTTIFG,
        // SLAU445I Figure 24-12, p. 638; SLAU445I Figure 24-13, p. 640)
        if !self.usci.is_master() {
            return match self.usci.ifg_rd().ucsttifg() {
                false => Err(I2cMasterSlaveErr::ArbitrationLost),
                true  => Err(I2cMasterSlaveErr::AddressedAsSlave),
            };
        }
        // Check if the eUSCI is addressing itself. The hardware isn't capable of this.
        // (SLAU445I 24.3.5.2, p. 636: "There is no hardware detection for this case")
        let own_addr_reg = self.usci.i2coa_rd(0);
        if own_addr_reg.ucoaen && own_addr_reg.i2coa0 == address {
            return Err(I2cMasterSlaveErr::TriedAddressingSelf);
        }
        Ok(())
    }

    // UCALIFG with UCSTTIFG: addressed as a slave after losing arbitration (SLAU445I Figure 24-12, p. 638;
    // SLAU445I Figure 24-13, p. 640)
    fn arbitration_err(&self, ifg: &USCI::IfgOut) -> Option<I2cMasterSlaveErr> {
        if !ifg.ucalifg() {
            return None;
        }
        Some(match ifg.ucsttifg() {
            false => I2cMasterSlaveErr::ArbitrationLost,  // Lost arbitration
            true  => I2cMasterSlaveErr::AddressedAsSlave, // Lost arbitration and the slave address was us
        })
    }
}
impl<USCI, M> I2cRoleMaster<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleSlavePrivate<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleSlave<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}
impl<USCI, M> I2cRoleMulti<M> for I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{}

impl<USCI, M> I2cMasterSlave<USCI, M>
where
    USCI: I2cUsci<M>,
    M: PinMap,
{
    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is empty,
    /// `Err(Other(I2cMasterSlaveErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(n)` if data was successfully retreived from the Rx buffer.
    #[inline]
    pub fn read_rx_buf_as_master(&mut self) -> nb::Result<u8, I2cMasterSlaveErr> {
        // UCALIFG with UCSTTIFG, UCNACKIFG and UCRXIFG0 (SLAU445I Table 24-19, p. 663)
        self.mst_read_rx_buf(&self.usci.ifg_rd())
    }

    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is empty, or
    /// `Ok(n)` if data was successfully retreived from the Rx buffer.
    #[inline(always)]
    pub fn read_rx_buf_as_slave(&mut self) -> nb::Result<u8, Infallible> { self.sl_read_rx_buf() }

    /// Read the Rx buffer without checking if it's ready. Should only be used if the peripheral is in slave mode.
    ///
    /// Useful in cases where you already know the Rx buffer is ready (e.g. an Rx interrupt occurred).
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then the data will be invalid (UCBxRXBUF holds "the last received
    /// character": SLAU445I Table 24-9, p. 655).
    #[inline(always)]
    pub unsafe fn read_rx_buf_as_slave_unchecked(&mut self) -> u8 { self.usci.ucrxbuf_rd() }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// First checks if the peripheral is still in master mode, if not returns an error.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is still full,
    /// `Err(Other(I2cMasterSlaveErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(())` if data was successfully loaded into the Tx buffer.
    #[inline]
    pub fn write_tx_buf_as_master(&mut self, byte: u8) -> nb::Result<(), I2cMasterSlaveErr> {
        // UCALIFG with UCSTTIFG, UCNACKIFG and UCTXIFG0 (SLAU445I Table 24-19, p. 663)
        self.mst_write_tx_buf(byte, &self.usci.ifg_rd())
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// Does not check if the peripheral is in master mode.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is still full, or
    /// `Ok(())` if data was successfully loaded into the Tx buffer.
    #[inline(always)]
    pub fn write_tx_buf_as_slave(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        self.sl_write_tx_buf(byte)
    }

    /// Write to the Tx buffer without checking if it's ready. Should only be used if the peripheral is in slave mode.
    /// Useful in cases where you already know the Tx buffer is ready (e.g. a Tx interrupt occurred).
    ///
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then previous data may be clobbered (UCBxTXBUF "holds the data waiting to be
    /// moved into the transmit shift register": SLAU445I Table 24-10, p. 655).
    #[inline(always)]
    pub unsafe fn write_tx_buf_as_slave_unchecked(&mut self, byte: u8) {
        self.usci.uctxbuf_wr(byte);
    }
}

macro_rules! impl_i2c_error {
    ($err_type: ty) => {
        impl I2cError for $err_type {
            #[inline(always)]
            fn nack(variant: NackType) -> Self { Self::GotNACK(variant) }

            #[inline(always)]
            fn nack_type(&self) -> Option<NackType> {
                match self {
                    Self::GotNACK(nack_type) => Some(*nack_type),
                    #[allow(unreachable_patterns)] // I2cSingleMasterErr has only one variant
                    _ => None,
                }
            }
        }
    };
}

/// NACK information enum. The contained value is the byte number when the error occurred.
///
/// If this originated from a blocking method the byte number counts up from the beginning of the transaction
/// (i.e. the initial start condition) where byte 0 is the address byte, byte 1 is the first data byte, etc..
/// If it originated from a non-blocking method it counts up from the most recent Start or Repeated Start condition
/// (the byte counter: SLAU445I 24.3.8, p. 643).
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
pub enum NackType {
    /// Received a NACK during an address byte. No device with the specified address is on the bus.
    Address(usize),
    /// Received a NACK during a data byte. This could be caused by a number of reasons -
    /// the receiver is not ready, it received invalid data or commands, it cannot receive any more data, etc.
    Data(usize),
}

/// I2C transmit/receive errors on a single master I2C bus.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2cSingleMasterErr {
    /// Received a NACK. The contained value denotes the byte where the NACK occurred.
    GotNACK(NackType),
    // Other errors like the 'clock low timeout' UCCLTOIFG may appear here in future.
}
impl_i2c_error!(I2cSingleMasterErr);

/// I2C transmit/receive errors on a multi-master I2C bus.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2cMultiMasterErr {
    /// Received a NACK. The contained value denotes the byte where the NACK occurred.
    GotNACK(NackType),
    /// Another master on the bus talked over us, so the transaction was aborted.
    /// The peripheral has been forced into slave mode (SLAU445I 24.3.5.3, p. 641).
    /// Call [`return_to_master()`](I2cRoleMulti::return_to_master) to resume the master role.
    ArbitrationLost,
    // Other errors like the 'clock low timeout' UCCLTOIFG may appear here in future.
}
impl_i2c_error!(I2cMultiMasterErr);

/// I2C transmit/receive errors on a master-slave I2C device.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2cMasterSlaveErr {
    /// Received a NACK. The contained value denotes the byte where the NACK occurred.
    GotNACK(NackType),
    /// Another master on the bus talked over us, so the transaction was aborted.
    /// The peripheral has been forced into slave mode (SLAU445I 24.3.5.3, p. 641).
    /// Call [`return_to_master()`](I2cRoleMulti::return_to_master) to resume the master role.
    ArbitrationLost,
    /// Another master on the bus addressed us as a slave device. The peripheral has been forced into slave mode
    /// (UCALIFG, SLAU445I Table 24-2, p. 646).
    /// The slave transaction *must* be completed before master operations can be resumed with
    /// [`return_to_master()`](I2cRoleMulti::return_to_master), which also clears the arbitration lost flag.
    /// The master methods leave the flags of the slave transaction as they are.
    AddressedAsSlave,
    /// The eUSCI peripheral attempted to address itself. The hardware does not support this operation
    /// (SLAU445I 24.3.5.2, p. 636).
    TriedAddressingSelf,
    // Other errors like the 'clock low timeout' UCCLTOIFG may appear here in future.
}
impl_i2c_error!(I2cMasterSlaveErr);

/// A list of events that may occur on the I2C bus.
///
/// Writing the Tx buffer clears UCTXIFGx and reading the Rx buffer clears UCRXIFGx
/// (SLAU445I Table 24-9, p. 655; SLAU445I Table 24-10, p. 655).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum I2cEvent {
    /// The master sent a (repeated) start and wants to read from us. Write to the Tx buffer to clear this event.
    ReadStart,
    /// The master continues to read from us. Write to the Tx buffer to clear this event.
    Read,
    /// The master sent a (repeated) start and wants to write to us. Read from the Rx buffer to clear this event.
    WriteStart,
    /// The master continues to write to us. Read from the Rx buffer to clear this event.
    Write,
    /// We have fallen behind. The master sent a write (filled the Rx buffer), then a repeated start and a read (currently stalled).
    ///
    /// The repeated start after the write means we can no longer tell if the initial write was a `WriteStart` or a `Write`.
    ///
    /// If you have more information about the expected format of the transaction you may be able to deduce which of the two it was.
    ///
    /// Read from the Rx buffer to clear this event.
    OverrunWrite,
    /// The master has ended the transaction. This event is automatically cleared.
    Stop,
}

/// List of possible I2C interrupt sources.
///
/// Used when reading from the I2C interrupt vector register via [`interrupt_source()`](I2cRoleCommon::interrupt_source())
///
/// The values are those of UCBxIV (SLAU445I Table 24-20, p. 664); the flags behind them are described in
/// SLAU445I Table 24-2, p. 646 and SLAU445I Table 24-19, p. 662.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum I2cVector {
    /// No interrupt.
    None             = 0x00,
    /// Arbitration was lost during an attempted transmission.
    ArbitrationLost  = 0x02,
    /// Received a NACK.
    NackReceived     = 0x04,
    /// Received a Start condition on the I2C bus along with one of our own addresses.
    StartReceived    = 0x06,
    /// Received a Stop condition on the I2C bus (UCSTPIFG: "set when the I2C module detects a STOP
    /// condition on the bus", SLAU445I Table 24-2, p. 646).
    /// This is usually set when acting as an I2C slave, but this can also occur as an I2C master: during a
    /// zero byte write (SLAU445I 24.3.5.2.1, p. 637), and as a master receiver "If a STOP condition was
    /// generated by the eUSCI_B module" (SLAU445I 24.3.5.2.2, p. 639). The blocking master methods and
    /// [`stop_sent()`](I2cRoleMaster::stop_sent) clear the flag of the master's own STOP.
    StopReceived     = 0x08,
    /// Slave address 3 received a data byte.
    Slave3RxBufFull  = 0x0A,
    /// The Tx buffer is empty and slave address 3 was on the I2C bus when this occurred.
    Slave3TxBufEmpty = 0x0C,
    /// Slave address 2 received a data byte.
    Slave2RxBufFull  = 0x0E,
    /// The Tx buffer is empty and slave address 2 was on the I2C bus when this occurred.
    Slave2TxBufEmpty = 0x10,
    /// Slave address 1 received a data byte.
    Slave1RxBufFull  = 0x12,
    /// The Tx buffer is empty and slave address 1 was on the I2C bus when this occurred.
    Slave1TxBufEmpty = 0x14,
    /// Data is waiting in the Rx buffer. In slave mode slave address 0 was on the I2C bus when this occurred.
    /// (UCRXIFG0, SLAU445I Table 24-19, p. 663; vector 16h, SLAU445I Table 24-20, p. 664)
    RxBufFull        = 0x16,
    /// The Tx buffer is empty. In slave mode slave address 0 was on the I2C bus when this occurred.
    /// (UCTXIFG0, SLAU445I Table 24-19, p. 663; vector 18h, SLAU445I Table 24-20, p. 664)
    TxBufEmpty       = 0x18,
    /// The target byte count has been reached.
    /// (UCBCNTIFG, SLAU445I Table 24-19, p. 662; vector 1Ah, SLAU445I Table 24-20, p. 664)
    ByteCounterZero  = 0x1A,
    /// The SCL line has been held low longer than the Clock Low Timeout value.
    /// (UCCLTOIFG, SLAU445I Table 24-19, p. 662; vector 1Ch, SLAU445I Table 24-20, p. 664)
    ClockLowTimeout  = 0x1C,
    /// The 9th bit of an I2C data packet has been completed.
    /// (UCBIT9IFG, SLAU445I Table 24-19, p. 662; vector 1Eh, SLAU445I Table 24-20, p. 664)
    NinthBitReceived = 0x1E,
}

bitflags::bitflags! {
    /// Human-friendly list of possible I2C interrupt source flags.
    ///
    /// Used for writing to the I2C interrupt enable register e.g. via the [`set_interrupts()`](I2cSingleMaster::set_interrupts()) method.
    ///
    /// The bits are those of UCBxIE (SLAU445I Figure 24-31, p. 660; SLAU445I Table 24-18, p. 660); the flags
    /// they enable are described in SLAU445I Table 24-2, p. 646 and SLAU445I Table 24-19, p. 662.
    pub struct I2cInterruptFlags: u16 {
        /// UCRXIE0. Trigger an interrupt when data is waiting in the Rx buffer. In slave mode slave address 0 must be on the I2C bus when this occurred.
        const RxBufFull           = 1 << 0;
        /// UCTXIE0. Trigger an interrupt when the Tx buffer is empty. In slave mode slave address 0 must be on the I2C bus when this occurred.
        const TxBufEmpty          = 1 << 1;
        /// UCSTTIE. Trigger an interrupt when a Start condition is received on the I2C bus along with one of our own addresses.
        const StartReceived       = 1 << 2;
        /// UCSTPIE. Trigger an interrupt when a Stop condition is detected on the I2C bus: UCSTPIFG is
        /// "set when the I2C module detects a STOP condition on the bus" (SLAU445I Table 24-2, p. 646), and
        /// the state change flags "are independent of the address comparison result"
        /// (SLAU445I 24.3.9.1, p. 644). Typically this triggers when acting as an I2C slave, but this also
        /// triggers as an I2C master during a zero byte write, and when a master receiver sends a STOP (see
        /// `I2cVector::StopReceived`).
        const StopReceived        = 1 << 3;
        /// UCALIE. Trigger an interrupt when arbitration was lost during an attempted transmission.
        /// (Bit 4, SLAU445I Table 24-18, p. 660)
        const ArbitrationLost     = 1 << 4;
        /// UCNACKIE. Trigger an interrupt a NACK is received. (Bit 5, SLAU445I Table 24-18, p. 660)
        const NackReceived        = 1 << 5;
        /// UCBCNTIE. Trigger an interrupt when the target byte count has been reached.
        /// (Bit 6, SLAU445I Table 24-18, p. 660)
        const ByteCounterZero     = 1 << 6;
        /// UCCLTOIE. Trigger an interrupt when the SCL line has been held low longer than the Clock Low Timeout value.
        /// (Bit 7, SLAU445I Table 24-18, p. 660)
        const ClockLowTimeout     = 1 << 7;
        /// UCRXIE1. Trigger an interrupt when slave address 1 receives a data byte.
        /// (Bit 8, SLAU445I Table 24-18, p. 660)
        const Slave1RxBufFull     = 1 << 8;
        /// UCTXIE1. Trigger an interrupt when the Tx buffer is empty and slave address 1 was on the I2C bus when this occurred.
        /// (Bit 9, SLAU445I Table 24-18, p. 660)
        const Slave1TxBufEmpty    = 1 << 9;
        /// UCRXIE2. Trigger an interrupt when slave address 2 receives a data byte.
        /// (Bit 10, SLAU445I Table 24-18, p. 660)
        const Slave2RxBufFull     = 1 << 10;
        /// UCTXIE2. Trigger an interrupt when the Tx buffer is empty and slave address 2 was on the I2C bus when this occurred.
        /// (Bit 11, SLAU445I Table 24-18, p. 660)
        const Slave2TxBufEmpty    = 1 << 11;
        /// UCRXIE3. Trigger an interrupt when slave address 3 receives a data byte.
        /// (Bit 12, SLAU445I Table 24-18, p. 660)
        const Slave3RxBufFull     = 1 << 12;
        /// UCTXIE3. Trigger an interrupt when the Tx buffer is empty and slave address 3 was on the I2C bus when this occurred.
        /// (Bit 13, SLAU445I Table 24-18, p. 660)
        const Slave3TxBufEmpty    = 1 << 13;
        /// UCBIT9IE. Trigger an interrupt when the 9th bit of an I2C data packet we are involved in has been completed.
        /// (Bit 14, SLAU445I Table 24-18, p. 660)
        const NinthBitReceived    = 1 << 14;
    }
}

// Trait to link embedded-hal types to our addressing mode enum.
// Since SevenBitAddress and TenBitAddress are just aliases for u8 and u16 in both ehal 1.0 and 0.2.7, this works for both!
/// A trait marking types that can be used as I2C addresses. Namely `u8` for 7-bit addresses and `u16` for 10-bit addresses.
///
/// Used internally by the HAL.
pub trait AddressType: AddressMode + Into<u16> + Copy {
    /// Return the `AddressingMode` that relates to this type: `SevenBit` for `u8`, `TenBit` for `u16`.
    fn addr_type() -> AddressingMode;
}
impl AddressType for SevenBitAddress {
    #[inline(always)]
    fn addr_type() -> AddressingMode { AddressingMode::SevenBit }
}
impl AddressType for TenBitAddress {
    #[inline(always)]
    fn addr_type() -> AddressingMode { AddressingMode::TenBit }
}

mod ehal1 {
    use super::*;
    use embedded_hal::i2c::{Error, ErrorKind, ErrorType, I2c, NoAcknowledgeSource, Operation};

    /// Implement embedded-hal's [`I2c`](embedded_hal::i2c::I2c) trait
    macro_rules! impl_ehal_i2c {
        ($type: ty, $err_type: ty) => {
            impl<USCI, M, TenOrSevenBit> I2c<TenOrSevenBit> for $type
            where
                USCI: I2cUsci<M>,
                M: PinMap,
                TenOrSevenBit: AddressType,
            {
                fn transaction(
                    &mut self,
                    address: TenOrSevenBit,
                    mut ops: &mut [Operation<'_>],
                ) -> Result<(), Self::Error> {
                    self.set_addressing_mode(TenOrSevenBit::addr_type());

                    // A read without bytes puts nothing on the bus, as the eUSCI can't receive zero bytes
                    // (see `blocking_read_unchecked`), so the STOP follows the last operation that isn't
                    // one. A write without bytes sends the address with its own STOP (`zero_byte_write`),
                    // so the operation after it starts with a START.
                    fn empty_read(op: &Operation<'_>) -> bool {
                        matches!(op, Operation::Read(items) if items.is_empty())
                    }
                    // Whether the previous operation was a read; `None` before the first one, and after a
                    // write without bytes
                    let mut prev_read = None;
                    let mut bytes_sent: usize = 0;
                    while let Some((op, rest)) = core::mem::take(&mut ops).split_first_mut() {
                        ops = rest;
                        if empty_read(op) {
                            continue;
                        }
                        let read = matches!(op, Operation::Read(_));
                        // Send a start if this is the first operation,
                        // or if the previous operation was a different type (e.g. Read and Write)
                        let send_start = prev_read != Some(read);
                        // Send a stop only after the last operation that isn't an empty read
                        let send_stop = ops.iter().all(empty_read);

                        let len = match op {
                            Operation::Read(items) => {
                                self.blocking_read(address.into(), items, send_start, send_stop)
                                    .map_err(|e| Self::add_nack_count(e, bytes_sent))?;
                                items.len()
                            }
                            Operation::Write(items) => {
                                self.blocking_write(address.into(), items, send_start, send_stop)
                                    .map_err(|e| Self::add_nack_count(e, bytes_sent))?;
                                items.len()
                            }
                        };
                        // Only numbers the byte of a NACK, so it wraps, as it does in release builds
                        bytes_sent = bytes_sent.wrapping_add(len);
                        prev_read = if len == 0 { None } else { Some(read) };
                    }
                    Ok(())
                }
            }
            impl<USCI, M> ErrorType for $type
            where
                USCI: I2cUsci<M>,
                M: PinMap,
            {
                type Error = $err_type;
            }
        };
    }

    use NackType::*;
    impl_ehal_i2c!(I2cSingleMaster<USCI, M>, I2cSingleMasterErr);
    impl Error for I2cSingleMasterErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2cSingleMasterErr::GotNACK(Address(_))  => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
                I2cSingleMasterErr::GotNACK(Data(_))     => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
            }
        }
    }

    impl_ehal_i2c!(I2cMultiMaster<USCI, M>, I2cMultiMasterErr);
    impl Error for I2cMultiMasterErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2cMultiMasterErr::GotNACK(Address(_))  => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
                I2cMultiMasterErr::GotNACK(Data(_))     => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
                I2cMultiMasterErr::ArbitrationLost      => ErrorKind::ArbitrationLoss,
            }
        }
    }

    impl_ehal_i2c!(I2cMasterSlave<USCI, M>, I2cMasterSlaveErr);
    impl Error for I2cMasterSlaveErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2cMasterSlaveErr::GotNACK(Address(_))  => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
                I2cMasterSlaveErr::GotNACK(Data(_))     => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
                I2cMasterSlaveErr::ArbitrationLost      => ErrorKind::ArbitrationLoss,
                I2cMasterSlaveErr::AddressedAsSlave     => ErrorKind::ArbitrationLoss,
                I2cMasterSlaveErr::TriedAddressingSelf  => ErrorKind::Other,
            }
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::blocking::i2c::{AddressMode, Read, Write, WriteRead};

    macro_rules! impl_ehal02_i2c {
        ($type: ty, $err_type: ty) => {
            impl<USCI, M, SevenOrTenBit> Read<SevenOrTenBit> for $type
            where
                USCI: I2cUsci<M>,
                M: PinMap,
                SevenOrTenBit: AddressMode + AddressType,
            {
                type Error = $err_type;
                #[inline]
                fn read(&mut self, address: SevenOrTenBit, buffer: &mut [u8]) -> Result<(), Self::Error> {
                    self.set_addressing_mode(SevenOrTenBit::addr_type());
                    self.blocking_read(address.into(), buffer, true, true)
                }
            }
            impl<USCI, M, SevenOrTenBit> Write<SevenOrTenBit> for $type
            where
                USCI: I2cUsci<M>,
                M: PinMap,
                SevenOrTenBit: AddressMode + AddressType,
            {
                type Error = $err_type;
                #[inline]
                fn write(&mut self, address: SevenOrTenBit, bytes: &[u8]) -> Result<(), Self::Error> {
                    self.set_addressing_mode(SevenOrTenBit::addr_type());
                    self.blocking_write(address.into(), bytes, true, true)
                }
            }
            impl<USCI, M, SevenOrTenBit> WriteRead<SevenOrTenBit> for $type
            where
                USCI: I2cUsci<M>,
                M: PinMap,
                SevenOrTenBit: AddressMode + AddressType,
            {
                type Error = $err_type;
                #[inline]
                fn write_read(
                    &mut self,
                    address: SevenOrTenBit,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Self::Error> {
                    self.set_addressing_mode(SevenOrTenBit::addr_type());
                    self.blocking_write_read(address.into(), bytes, buffer)
                }
            }
        };
    }

    impl_ehal02_i2c!(I2cSingleMaster<USCI, M>, I2cSingleMasterErr);
    impl_ehal02_i2c!(I2cMultiMaster<USCI, M>,  I2cMultiMasterErr);
    impl_ehal02_i2c!(I2cMasterSlave<USCI, M>,  I2cMasterSlaveErr);
}
