//! Enhanced Comparator (eCOMP)
//!
//! The enhanced comparator peripheral consists of a comparator with configurable inputs - including
//! GPIO pins, a low power 1.2V reference, on the MSP430FR235x the outputs of two Smart Analog Combo
//! (SAC) amplifiers, and a 6-bit DAC. The comparator output can be read by software and/or routed
//! to a GPIO pin. (SLAU445I 18.1, p. 504; SLASEC4D 6.10.13, p. 78; SLASEO7C 9.10.13, p. 62 to p. 63)
//!
//! The comparator has a pair of inputs. In normal operation, when the positive input is larger than
//! the negative input the output is high, otherwise it is low. This behaviour can be inverted by
//! selecting the inverted output polarity mode. The comparator also features a selectable power mode
//! ('high speed' or 'low power'), configurable hysteresis levels and an optional, variable-strength
//! analog low pass filter on the output. (SLAU445I 18.2.1, p. 505, and SLAU445I 18.2.3, p. 505; CPINV,
//! CPMSEL, CPHSEL, CPFLT and CPFLTDLY: SLAU445I Table 18-3, p. 510)
//!
//! The internal DAC contains dual input buffers, where either buffer can be set as the input value
//! to the DAC. The buffer selection can either be done by software (software mode),
//! or selected automatically by the output value of the comparator (hardware mode).
//! The voltage reference of the DAC can be chosen as either VCC or the internal shared reference
//! (provided it has been previously configured). (SLAU445I 18.2.4, p. 506, and SLAU445I Table 18-6, p. 512)
//!
//! Interrupts can be triggered on rising, falling, or both edges of the comparator output. Each edge also sets a
//! flag, which [`Comparator::interrupt_source()`] reads and clears in the `ECOMP0` (MSP430FR247x) or
//! `ECOMP0_ECOMP1` (MSP430FR2x5x) interrupt handler. (SLAU445I 18.3, p. 507; interrupt vectors: SLASEO7C
//! Table 9-2, p. 47; SLASEC4D Table 6-2, p. 64)
//!
//! See the simplified diagram below:
//!
#![doc= include_str!("../docs/ecomp.svg")]
//!
//! Begin configuration by calling [`ECompConfig::begin()`], which returns two configuration objects: One for the
//! eCOMP's internal DAC: [`ComparatorDacConfig`], and the other for the comparator itself: [`ComparatorConfig`].
//! If the DAC is not used then it need not be configured.
//!
//! Linked pins and peripherals (data sheets, eCOMP channel connection tables, listed below the table):
//!
//! | Device       |        | SAC (+) | SAC (-) | COMPx.0 | COMPx.1 | COMPx.2 | COMPx.3 | COMPxOut |
//! |:------------:|:------:|:-------:|:-------:|:-------:|:-------:|:-------:|:-------:|:--------:|
//! | MSP430FR2x5x | eCOMP0 | SAC0    | SAC2    | `P1.0`  | `P1.1`  |         |         | `P2.0`   |
//! | MSP430FR2x5x | eCOMP1 | SAC1    | SAC3    | `P2.5`  | `P2.4`  |         |         | `P2.1`   |
//! | MSP430FR247x | eCOMP0 |         |         | `P1.1`  | `P2.2`  | `P5.7`  | `P6.0`  | `P3.4`   |
//!
//! - MSP430FR2x5x inputs: SLASEC4D Table 6-23, p. 78, and SLASEC4D Table 6-24, p. 78 (the SACs at CPPSEL and
//!   CPNSEL = 101b); outputs: SLASEC4D Table 6-25, p. 78, and SLASEC4D Table 6-26, p. 79
//! - MSP430FR247x inputs: SLASEO7C Table 9-21, p. 63; output: SLASEO7C Table 9-22, p. 63
//!
//! Only the MSP430FR2355 and MSP430FR2353 have SACs (SLASEC4D 6.10.15, p. 79: "Only MSP430FR235x devices
//! implement the SAC modules").
//!
//! After reset, a high comparator output also switches the outputs of some Timer_B peripherals to high impedance:
//! eCOMP0 those of TB0 and TB1, eCOMP1 those of TB2 and TB3. See
//! [`TimerConfig::high_impedance_trigger`](crate::timer::TimerConfig::high_impedance_trigger) to turn that off.
//! (TBxTRGSEL resets to the internal source: SLAU445I Table 1-26, p. 77, and SLAU445I Table 1-31, p. 82;
//! which is the eCOMP output: SLASEC4D Table 6-20, p. 76; SLASEO7C Table 9-17, p. 61, which lists only TB0,
//! the MSP430FR247x's only Timer_B. A high level makes "all Timer_B outputs ... in a high-impedance state",
//! as SLAU445I 14.2.5, p. 401, says for the TBOUTH pin.)
//!
//! The comparator output is also input B of a timer's capture pin 1, so a capture can time its edges
//! (SLAU445I 18.1, p. 504: "Output provided to timer capture input"; see [`capture`](crate::capture)). On
//! the MSP430FR2x5x eCOMP0 drives TB0's and eCOMP1 TB2's (SLASEC4D Table 6-16, p. 73; SLASEC4D
//! Table 6-18, p. 74). On the MSP430FR247x eCOMP0 doesn't reach TB0's, erratum COMP12: "eCOMP0 output
//! can not be selected internally to the Timer0_B7 CCI1B input (TB0CCTL1.CCIS = 01b)". Its workaround is
//! to "Connect eCOMP0 output and Timer B capture input externally through GPIOs" (SLAZ726B COMP12, p. 5):
//! route the output to P3.4 with [`ComparatorConfig::with_output_pin`], and wire that pin to a capture
//! pin's input A, such as TB0.CCI1A on P4.7 (SLASEO7C Table 9-22, p. 63; SLASEO7C Table 9-15, p. 59).

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
    /// Configure the comparator with the provided settings and turn it on (CPxCTL0 and CPxCTL1: SLAU445I
    /// Table 18-2, p. 509, and SLAU445I Table 18-3, p. 510)
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
    /// The current value of the comparator output (CPOUT: SLAU445I Table 18-3, p. 511)
    #[inline(always)]
    pub fn value(&mut self) -> bool { COMP::value() }

    /// Whether the current value of the comparator output is high
    #[inline(always)]
    pub fn is_high(&mut self) -> bool { COMP::value() }

    /// Whether the current value of the comparator output is low
    #[inline(always)]
    pub fn is_low(&mut self) -> bool { !COMP::value() }

    /// Enable rising-edge interrupts (CPIFG). (CPIE: SLAU445I Table 18-3, p. 510; CPIFG is set on rising
    /// edges while CPIES = 0, its reset value: SLAU445I Table 18-3, p. 510)
    #[inline(always)]
    pub fn enable_rising_interrupts(&mut self) { COMP::en_cpie(); }

    /// Disable rising-edge interrupts (CPIFG). (CPIE: SLAU445I Table 18-3, p. 510)
    #[inline(always)]
    pub fn disable_rising_interrupts(&mut self) { COMP::dis_cpie(); }

    /// Enable falling-edge interrupts (CPIIFG). (CPIIE: SLAU445I Table 18-3, p. 510; CPIIFG is set on falling
    /// edges while CPIES = 0, its reset value: SLAU445I Table 18-3, p. 510)
    #[inline(always)]
    pub fn enable_falling_interrupts(&mut self) { COMP::en_cpiie(); }

    /// Disable falling-edge interrupts (CPIIFG). (CPIIE: SLAU445I Table 18-3, p. 510)
    #[inline(always)]
    pub fn disable_falling_interrupts(&mut self) { COMP::dis_cpiie(); }

    /// Whether the output rose since the flag was last cleared (CPIFG), whether or not its interrupt is enabled.
    /// (CPIFG, CPxINT bit 0: SLAU445I Table 18-4, p. 511; set on each edge: SLAU445I 18.3, p. 507)
    #[inline(always)]
    pub fn rising_edge_flag(&self) -> bool { COMP::rising_flag() }

    /// Whether the output fell since the flag was last cleared (CPIIFG), whether or not its interrupt is enabled.
    /// (CPIIFG, CPxINT bit 1: SLAU445I Table 18-4, p. 511; set on each edge: SLAU445I 18.3, p. 507)
    #[inline(always)]
    pub fn falling_edge_flag(&self) -> bool { COMP::falling_flag() }

    /// Clear both edge flags. Clear them before enabling interrupts, or an edge from before requests one.
    /// (SLAU445I Table 18-4, p. 511: "Write 1 to clear this bit"; configuring can set them too, SLAU445I
    /// Table 18-3, p. 510: "Changing CPFLT might set interrupt flag")
    #[inline(always)]
    pub fn clear_edge_flags(&mut self) { COMP::clear_edge_flags(); }

    /// The highest-priority pending interrupt among the enabled ones (CPxIV). Reading it clears its flag.
    /// (SLAU445I Table 18-5, p. 512)
    #[inline(always)]
    pub fn interrupt_source(&mut self) -> ComparatorVector { COMP::iv() }
}

/// The highest-priority pending comparator interrupt, as read from CPxIV by [`Comparator::interrupt_source()`]
/// (SLAU445I Table 18-5, p. 512)
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
    /// (SLAU445I 18.2.4, p. 506; CPDACEN, CPDACBUFS = 1 and CPDACSW: SLAU445I Table 18-6, p. 512)
    #[inline(always)]
    pub fn new_sw_dac(self, vref: DacVRef, buf: BufferSel) -> ComparatorDac<COMP, SwDualBuffer> {
        COMP::cpxdacctl(true, vref, DacBufferMode::Software, buf);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
    /// Initialise the DAC in this eCOMP peripheral in hardware dual buffering mode.
    ///
    /// The DAC value is determined by one of two buffers. In hardware mode the comparator output value selects the buffer.
    /// (SLAU445I 18.2.4, p. 506; CPDACEN and CPDACBUFS = 0: SLAU445I Table 18-6, p. 512)
    #[inline(always)]
    pub fn new_hw_dac(self, vref: DacVRef) -> ComparatorDac<COMP, HwDualBuffer> {
        // CPDACSW only counts when CPDACBUFS = 1 (SLAU445I Table 18-6, p. 512)
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
    /// Set the value in buffer 1 (CPDACBUF1: 6 bits, the reference voltage x count / 64, SLAU445I Table 18-7,
    /// p. 513)
    #[inline(always)]
    pub fn write_buffer_1(&mut self, count: u8) { COMP::set_buf1_val(count); }
    /// Set the value in buffer 2 (CPDACBUF2: 6 bits, the reference voltage x count / 64, SLAU445I Table 18-7,
    /// p. 513)
    #[inline(always)]
    pub fn write_buffer_2(&mut self, count: u8) { COMP::set_buf2_val(count); }
}
impl<'a, COMP: ECompInputs> ComparatorDac<'a, COMP, SwDualBuffer> {
    /// Consume this DAC and return a DAC in the hardware dual buffer mode (CPDACBUFS = 0: SLAU445I
    /// Table 18-6, p. 512)
    #[inline(always)]
    pub fn into_hw_buffer_mode(self) -> ComparatorDac<'a, COMP, HwDualBuffer> {
        COMP::set_dac_buffer_mode(DacBufferMode::Hardware);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
    /// Select which buffer is passed to the DAC (CPDACSW: SLAU445I Table 18-6, p. 512)
    #[inline(always)]
    pub fn select_buffer(&mut self, buf: BufferSel) { COMP::select_buffer(buf); }
}
impl<'a, COMP: ECompInputs> ComparatorDac<'a, COMP, HwDualBuffer> {
    /// Consume this DAC and return a DAC in the software dual buffer mode (CPDACBUFS = 1: SLAU445I
    /// Table 18-6, p. 512)
    #[inline(always)]
    pub fn into_sw_buffer_mode(self) -> ComparatorDac<'a, COMP, SwDualBuffer> {
        COMP::set_dac_buffer_mode(DacBufferMode::Software);
        ComparatorDac { reg: PhantomData, mode: PhantomData, vref_lifetime: PhantomData }
    }
}

/// List of possible reference voltages for eCOMP DACs (CPDACREFS: SLAU445I Table 18-6, p. 512; the internal
/// shared reference: SLASEC4D 6.10.13, p. 78, and SLASEO7C 9.10.13, p. 62)
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

/// Possible buffers used by the eCOMP DAC (CPDACSW: SLAU445I Table 18-6, p. 512)
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
///
/// (CPHSEL: SLAU445I Table 18-3, p. 510; typical VHYS: SLASEC4D Table 5-23, p. 53, SLASEC4D Table 5-24,
/// p. 54, and SLASEO7C 8.12.9.1, p. 42)
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
/// (CPMSEL: SLAU445I Table 18-3, p. 510; delays tPD and currents ICOMP: SLASEC4D Table 5-23, p. 53, and
/// SLASEO7C 8.12.9.1, p. 42, for eCOMP0; SLASEC4D Table 5-24, p. 54, for eCOMP1)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PowerMode {
    /// eCOMP0: 1us @ 24 uA.
    ///
    /// eCOMP1 (MSP430FR2x5x only): 100ns @ 162 uA
    HighSpeed,
    /// eCOMP0: 3.2us @ 1.6uA.
    ///
    /// eCOMP1 (MSP430FR2x5x only): 320ns @ 20uA (SLASEC4D Table 5-24, p. 54; SLASEC4D 6.10.13, p. 78, gives
    /// 10 uA of leakage instead)
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
///
/// (CPFLT and CPFLTDLY: SLAU445I 18.2.3, p. 505, and SLAU445I Table 18-3, p. 510, which gives the typical
/// delays below and says they are only valid in high speed mode.) The data sheets give the propagation
/// delay with the filter instead, tFDLY: 0.7 us, 1.1 us, 1.9 us and 3.4 us for eCOMP0 (SLASEC4D
/// Table 5-23, p. 53; SLASEO7C 8.12.9.1, p. 42) and 150 ns, 350 ns, 1000 ns and 1900 ns for eCOMP1
/// (SLASEC4D Table 5-24, p. 54).
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

/// Output polarity of the comparator (CPINV: SLAU445I Table 18-3, p. 510)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputPolarity {
    /// Non-inverted output: When V+ is larger than V- the output is high (SLAU445I 18.2.1, p. 505).
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
