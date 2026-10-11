//! Enhanced Comparator (eCOMP)
//!
//! The enhanced comparator peripheral consists of a comparator with configurable inputs - including
//! GPIO pins, a low power 1.2V reference, on the MSP430FR235x the outputs of two Smart Analog Combo
//! (SAC) amplifiers, and a 6-bit DAC. The comparator output can be read by software and/or routed
//! to a GPIO pin (SLAU445I 18.1, p. 504).
//!
//! The comparator has a pair of inputs. In normal operation, when the positive input is larger than
//! the negative input the output is high, otherwise it is low. This behaviour can be inverted by
//! selecting the inverted output polarity mode. The comparator also features a selectable power mode
//! ('high speed' or 'low power'), configurable hysteresis levels and an optional, variable-strength
//! analog low pass filter on the output.
//!
//! The internal DAC contains dual input buffers, where either buffer can be set as the input value
//! to the DAC. The buffer selection can either be done by software (software mode),
//! or selected automatically by the output value of the comparator (hardware mode).
//! The voltage reference of the DAC can be chosen as either VCC or the internal shared reference
//! (provided it has been previously configured).
//!
//! Interrupts can be triggered on rising, falling, or both edges of the comparator output. Each edge also sets a
//! flag, which [`Comparator::interrupt_source()`] reads and clears in the `ECOMP0` (MSP430FR247x) or
//! `ECOMP0_ECOMP1` (MSP430FR2x5x) interrupt handler.
//!
//! See the simplified diagram below:
//!
#![doc= include_str!("../docs/ecomp.svg")]
//!
//! Begin configuration by calling [`ECompConfig::begin()`], which returns two configuration objects: One for the
//! eCOMP's internal DAC: [`ComparatorDacConfig`], and the other for the comparator itself: [`ComparatorConfig`].
//! If the DAC is not used then it need not be configured.
//!
//! Linked pins and peripherals:
//!
//! | Device       |        | SAC (+) | SAC (-) | COMPx.0 | COMPx.1 | COMPx.2 | COMPx.3 | COMPxOut |
//! |:------------:|:------:|:-------:|:-------:|:-------:|:-------:|:-------:|:-------:|:--------:|
//! | MSP430FR2x5x | eCOMP0 | SAC0    | SAC2    | `P1.0`  | `P1.1`  |         |         | `P2.0`   |
//! | MSP430FR2x5x | eCOMP1 | SAC1    | SAC3    | `P2.5`  | `P2.4`  |         |         | `P2.1`   |
//! | MSP430FR247x | eCOMP0 |         |         | `P1.1`  | `P2.2`  | `P5.7`  | `P6.0`  | `P3.4`   |
//!
//! After reset, a high comparator output also switches Timer_B outputs to high impedance: eCOMP0 those of TB0
//! and TB1, eCOMP1 those of TB2 and TB3 (on the MSP430FR247x, eCOMP0 those of TB0).

pub use crate::device_specific::ecomp::{NegativeInput, PositiveInput};
use crate::{
    hw_traits::ecomp::{DacBufferMode, ECompInputs},
    pmm::InternalVRef,
};
use core::marker::PhantomData;

/// Struct representing a configuration for an enhanced comparator (eCOMP) module.
pub struct ECompConfig<COMP: ECompInputs>(PhantomData<COMP>);
impl<COMP: ECompInputs> ECompConfig<COMP> {
    /// Begin configuration of an enhanced comparator (eCOMP) module.
    #[inline(always)]
    pub fn begin(_reg: COMP) -> (ComparatorDacConfig<COMP>, ComparatorConfig<COMP, NoModeSet>) {
        (ComparatorDacConfig(PhantomData), ComparatorConfig(PhantomData, PhantomData))
    }
}

/// A configuration for the comparator in an eCOMP module
pub struct ComparatorConfig<COMP: ECompInputs, MODE>(PhantomData<COMP>, PhantomData<MODE>);
impl<COMP: ECompInputs> ComparatorConfig<COMP, NoModeSet> {
    /// Configure the comparator with the provided settings and turn it on
    #[inline(always)]
    pub fn configure(
        self,
        pos_in: PositiveInput<COMP>,
        neg_in: NegativeInput<COMP>,
        pol: OutputPolarity,
        pwr: PowerMode,
        hstr: Hysteresis,
        fltr: FilterStrength,
    ) -> ComparatorConfig<COMP, ModeSet> {
        COMP::cpxctl0(pos_in.cppsel(), neg_in.cpnsel());
        COMP::configure_comparator(pol, pwr, hstr, fltr);
        ComparatorConfig(PhantomData, PhantomData)
    }
}
impl<COMP: ECompInputs> ComparatorConfig<COMP, ModeSet> {
    /// Route the comparator output to its GPIO pin (see the table in the module documentation).
    #[inline(always)]
    pub fn with_output_pin(self, _pin: COMP::COMPx_Out) -> Comparator<COMP> {
        Comparator(PhantomData)
    }
    /// Do not route the comparator output to its GPIO pin
    #[inline(always)]
    pub fn no_output_pin(self) -> Comparator<COMP> { Comparator(PhantomData) }
}

/// Struct representing a configured eCOMP comparator.
pub struct Comparator<COMP: ECompInputs>(PhantomData<COMP>);
impl<COMP: ECompInputs> Comparator<COMP> {
    /// The current value of the comparator output
    #[inline(always)]
    pub fn value(&mut self) -> bool { COMP::value() }

    /// Whether the current value of the comparator output is high
    #[inline(always)]
    pub fn is_high(&mut self) -> bool { COMP::value() }

    /// Whether the current value of the comparator output is low
    #[inline(always)]
    pub fn is_low(&mut self) -> bool { !COMP::value() }

    /// Enable rising-edge interrupts (CPIFG).
    #[inline(always)]
    pub fn enable_rising_interrupts(&mut self) { COMP::en_cpie(); }

    /// Disable rising-edge interrupts (CPIFG).
    #[inline(always)]
    pub fn disable_rising_interrupts(&mut self) { COMP::dis_cpie(); }

    /// Enable falling-edge interrupts (CPIIFG).
    #[inline(always)]
    pub fn enable_falling_interrupts(&mut self) { COMP::en_cpiie(); }

    /// Disable falling-edge interrupts (CPIIFG).
    #[inline(always)]
    pub fn disable_falling_interrupts(&mut self) { COMP::dis_cpiie(); }

    /// Whether the output rose since the flag was last cleared (CPIFG), whether or not its interrupt is enabled.
    #[inline(always)]
    pub fn rising_edge_flag(&self) -> bool { COMP::int_flags() & 0b01 != 0 }

    /// Whether the output fell since the flag was last cleared (CPIIFG), whether or not its interrupt is enabled.
    #[inline(always)]
    pub fn falling_edge_flag(&self) -> bool { COMP::int_flags() & 0b10 != 0 }

    /// Clear both edge flags. Clear them before enabling interrupts, or an edge from before requests one.
    /// Configuring the comparator can set them too.
    #[inline(always)]
    pub fn clear_edge_flags(&mut self) { COMP::clear_int_flags(0b11); }

    /// The highest-priority pending interrupt among the enabled ones (CPxIV). Reading it clears its flag.
    #[inline(always)]
    pub fn interrupt_source(&mut self) -> ComparatorVector {
        match COMP::iv() {
            0x02 => ComparatorVector::RisingEdge,
            0x04 => ComparatorVector::FallingEdge,
            _ => ComparatorVector::None,
        }
    }
}

/// The highest-priority pending comparator interrupt, as read from CPxIV by [`Comparator::interrupt_source()`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComparatorVector {
    /// No interrupt pending
    None,
    /// The output rose (CPIFG)
    RisingEdge,
    /// The output fell (CPIIFG)
    FallingEdge,
}

/// Represents a configuration for the DAC in an eCOMP peripheral
pub struct ComparatorDacConfig<COMP>(PhantomData<COMP>);
impl<COMP: ECompInputs> ComparatorDacConfig<COMP> {
    /// Initialise the DAC in this eCOMP peripheral in software dual buffering mode.
    ///
    /// The DAC value is determined by one of two buffers. In software mode this is selectable at will.
    #[inline(always)]
    pub fn new_sw_dac(self, vref: DacVRef, buf: BufferSel) -> ComparatorDac<COMP, SwDualBuffer> {
        COMP::cpxdacctl(true, vref, DacBufferMode::Software, buf);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
    /// Initialise the DAC in this eCOMP peripheral in hardware dual buffering mode.
    ///
    /// The DAC value is determined by one of two buffers. In hardware mode the comparator output value selects the buffer.
    #[inline(always)]
    pub fn new_hw_dac(self, vref: DacVRef) -> ComparatorDac<COMP, HwDualBuffer> {
        COMP::cpxdacctl(true, vref, DacBufferMode::Hardware, BufferSel::_1);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
}

/// Represents an eCOMP DAC that has been configured
pub struct ComparatorDac<'a, COMP: ECompInputs, MODE> {
    reg: PhantomData<COMP>,
    mode: PhantomData<MODE>,
    vref_lifetime: PhantomData<DacVRef<'a>>, // If we are using internal vref ensure it stays on for the lifetime of the DAC
}
impl<COMP: ECompInputs, MODE> ComparatorDac<'_, COMP, MODE> {
    /// Set the value in buffer 1 (CPDACBUF1)
    #[inline(always)]
    pub fn write_buffer_1(&mut self, count: u8) { COMP::set_buf1_val(count); }
    /// Set the value in buffer 2 (CPDACBUF2)
    #[inline(always)]
    pub fn write_buffer_2(&mut self, count: u8) { COMP::set_buf2_val(count); }
}
impl<'a, COMP: ECompInputs> ComparatorDac<'a, COMP, SwDualBuffer> {
    /// Consume this DAC and return a DAC in the hardware dual buffer mode
    #[inline(always)]
    pub fn into_hw_buffer_mode(self) -> ComparatorDac<'a, COMP, HwDualBuffer> {
        COMP::set_dac_buffer_mode(DacBufferMode::Hardware);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
    /// Select which buffer is passed to the DAC
    #[inline(always)]
    pub fn select_buffer(&mut self, buf: BufferSel) { COMP::select_buffer(buf); }
}
impl<'a, COMP: ECompInputs> ComparatorDac<'a, COMP, HwDualBuffer> {
    /// Consume this DAC and return a DAC in the software dual buffer mode
    #[inline(always)]
    pub fn into_sw_buffer_mode(self) -> ComparatorDac<'a, COMP, SwDualBuffer> {
        COMP::set_dac_buffer_mode(DacBufferMode::Software);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
}

/// List of possible reference voltages for eCOMP DACs
#[derive(Debug, Copy, Clone)]
pub enum DacVRef<'a> {
    /// Use VCC as the reference voltage for this eCOMP DAC
    Vcc,
    /// Use the internal shared voltage reference for this eCOMP DAC
    Internal(&'a InternalVRef),
}
impl From<DacVRef<'_>> for bool {
    #[inline(always)]
    fn from(value: DacVRef) -> Self {
        match value {
            DacVRef::Vcc         => false,
            DacVRef::Internal(_) => true,
        }
    }
}

/// Possible buffers used by the eCOMP DAC
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BufferSel {
    /// CPDACBUF1
    _1,
    /// CPDACBUF2
    _2,
}
impl From<BufferSel> for bool {
    #[inline(always)]
    fn from(value: BufferSel) -> Self {
        match value {
            BufferSel::_1 => false,
            BufferSel::_2 => true,
        }
    }
}

/// Possible hysteresis value for an eCOMP comparator. Larger hysteresis values require a larger voltage
/// difference between the input signals before a comparator output transition occurs.
///
/// Larger values reduce spurious output transitions when the two inputs are very close together, effectively
/// making the comparator less sensitive - both to noise but also to the input signals themselves.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Hysteresis {
    /// No hysteresis.
    Off   = 0b00,
    /// 10mV of hysteresis.
    _10mV = 0b01,
    /// 20mV of hysteresis.
    _20mV = 0b10,
    /// 30mV of hysteresis.
    _30mV = 0b11,
}

/// Possible comparator power modes. Controls the power consumption and propogation delay of the comparator.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PowerMode {
    /// eCOMP0: 1us @ 24 uA.
    ///
    /// eCOMP1 (MSP430FR2x5x only): 100ns @ 162 uA
    HighSpeed,
    /// eCOMP0: 3.2us @ 1.6uA.
    ///
    /// eCOMP1 (MSP430FR2x5x only): 320ns @ 20uA
    LowPower,
}
impl From<PowerMode> for bool {
    #[inline(always)]
    fn from(value: PowerMode) -> Self {
        match value {
            PowerMode::HighSpeed => false,
            PowerMode::LowPower => true,
        }
    }
}

/// The possible values for the strength of the low pass filter in the eCOMP module.
///
/// Higher values will more aggressively filter out high-frequency components from the output,
/// but also delays the output signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FilterStrength {
    /// Typical delay of 450 ns (in high speed mode).
    Low      = 0b00,
    /// Typical delay of 900 ns (in high speed mode).
    Medium   = 0b01,
    /// Typical delay of 1800 ns (in high speed mode).
    High     = 0b10,
    /// Typical delay of 3600 ns (in high speed mode).
    VeryHigh = 0b11,
    /// Do not use the low pass filter.
    Off      = 0b100,
}

/// Output polarity of the comparator
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputPolarity {
    /// Non-inverted output: When V+ is larger than V- the output is high.
    Noninverted,
    /// Inverted output: When V+ is larger than V- the output is low.
    Inverted,
}
impl From<OutputPolarity> for bool {
    #[inline(always)]
    fn from(value: OutputPolarity) -> Self {
        match value {
            OutputPolarity::Noninverted => false,
            OutputPolarity::Inverted => true,
        }
    }
}

/// Marker struct for a Comparator that has not been configured
pub struct NoModeSet;
/// Marker struct for a Comparator that is being configured
pub struct ModeSet;

/// Typestate for a eCOMP DAC that is set in the hardware dual buffering mode.
pub struct HwDualBuffer;
/// Typestate for a eCOMP DAC that is set in the software dual buffering mode.
pub struct SwDualBuffer;
