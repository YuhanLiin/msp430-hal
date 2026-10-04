//! Smart Analog Combo L3 (SAC-L3)
//!
//! The Smart Analog Combo (SAC) integrates an operational amplifier, a programmable gain amplifier with up to 33x
//! gain, and a 12-bit Digital to Analog Converter (DAC) core. The SAC can be used for signal
//! conditioning for either the input or output path. (SLAU445I chapter 20, p. 518)
//!
//! Only available on the MSP430FR235x (SLASEC4D 6.10.15, p. 79).
//!
//! There are four SAC modules, forming two pairs (SAC0 and SAC2, SAC1 and SAC3).
//! The amplifiers in each pair can be fed the output of the other (e.g. SAC0 may use the output of SAC2,
//! and SAC2 may use the output of SAC0). Both amplifiers can be fed into the respective
//! enhanced comparator module (eCOMP) - SAC0 and SAC2 into eCOMP0, and SAC1 and SAC3 into eCOMP1.
//! (SLASEC4D 6.10.15, p. 79, with SLASEC4D Table 6-27, p. 79, to SLASEC4D Table 6-30, p. 80; SLASEC4D
//! Figure 6-1, p. 81, and SLASEC4D Figure 6-2, p. 82)
//!
//! Each SAC can be put into one of four modes (SLAU445I 20.2.2, p. 521, to SLAU445I 20.2.2.5, p. 527):
//! - An open-loop operational amplifier (no internal feedback):
#![allow(rustdoc::bare_urls)] // SVG files trigger false positives
#![doc= include_str!("../docs/sac_open_loop.svg")]
//! - An inverting amplifier with programmable gain/feedback:
#![doc= include_str!("../docs/sac_inverting.svg")]
//! - A non-inverting amplifier with programmable gain/feedback:
#![doc= include_str!("../docs/sac_noninverting.svg")]
//! - A unity-gain buffer / voltage follower:
#![doc= include_str!("../docs/sac_buffer.svg")]
//!
//! The amplifier inputs can be connected to the external pins OA+ and OA-, the SAC's 12-bit DAC or the output of
//! the paired SAC amplifier (SLAU445I 20.2.1.1, p. 521). Each mode supports some of these sources (SLAU445I
//! Table 20-1, p. 521):
//!
//! | Mode                    | Positive input                         | Negative input                                    |
//! |:-----------------------:|:--------------------------------------:|:-------------------------------------------------:|
//! | Open-loop opamp         | OA+ or the paired amplifier            | OA- or the paired amplifier                       |
//! | Inverting amplifier     | OA+ or the DAC, which set the bias     | OA- or the paired amplifier, through the gain resistors |
//! | Non-inverting amplifier | OA+ or the paired amplifier            | The gain resistors                                |
//! | Buffer                  | OA+, the DAC or the paired amplifier   | The output                                        |
//!
//! SLAU445I Table 20-1, p. 521, lists only OA- for the negative input of the open-loop opamp. The paired
//! amplifier there comes from SLAU445I 20.2.1.1, p. 521, and the data sheet's SAC channel tables (NSEL = 10:
//! SLASEC4D Table 6-27, p. 79, to SLASEC4D Table 6-30, p. 80).
//!
//! The output of the amplifier can either be routed to the external pin OAO,
//! or used internally with the enhanced comparator module (SLASEC4D Figure 6-1, p. 81, and SLASEC4D
//! Figure 6-2, p. 82).
//!
//! To begin configuration, call [`SacConfig::begin()`]. This returns configuration objects for the DAC
//! and for the amplifier. If the DAC is not used then it need not be configured.
//!
//! Pins used (SLASEC4D Table 6-27, p. 79, to SLASEC4D Table 6-30, p. 80; their function, PxSELx = 11:
//! SLASEC4D Table 6-63, p. 96, and SLASEC4D Table 6-65, p. 100):
//!
//! |        |   OA+  |  OA--  |   OAO   |
//! |:------:|:------:|:------:|:-------:|
//! | SAC0   | `P1.3` | `P1.2` | `P1.1`  |
//! | SAC1   | `P1.7` | `P1.6` | `P1.5`  |
//! | SAC2   | `P3.3` | `P3.2` | `P3.1`  |
//! | SAC3   | `P3.7` | `P3.6` | `P3.5`  |
//!

use core::marker::PhantomData;

use crate::{
    hw_traits::sac::{MSel, NSel, SacPeriph},
    pac::Tb2,
    pmm::InternalVRef,
    pwm::{CCR1, CCR2},
    timer::SubTimer,
};

/// A builder for configuring a Smart Analog Combo (SAC) unit
pub struct SacConfig;
impl SacConfig {
    /// Begin configuration of a Smart Analog Combo (SAC) unit.
    #[inline(always)]
    pub fn begin<SAC: SacPeriph>(_reg: SAC) -> (DacConfig<SAC>, AmpConfig<NoModeSet, SAC>) {
        (DacConfig(PhantomData), AmpConfig { mode: PhantomData, reg: PhantomData })
    }
}

/// Struct representing a configuration for a DAC inside this Smart Analog Combo (SAC) unit.
pub struct DacConfig<SAC: SacPeriph>(PhantomData<SAC>);
impl<SAC: SacPeriph> DacConfig<SAC> {
    /// Initialise the DAC within this SAC with the provided values (SACxDAC: SLAU445I Table 20-8, p. 534).
    #[inline(always)]
    pub fn configure<'a>(self, vref: VRef<'a>, load_trigger: LoadTrigger<'_>) -> Dac<'a, SAC> {
        SAC::configure_dac(load_trigger.into(), vref.into(), false);
        Dac { sac: PhantomData, vref_lifetime: PhantomData }
    }

    /// Initialise the DAC like [`configure()`](Self::configure), and request an interrupt each time the DAC loads a
    /// new value: `SAC0_SAC2` for SAC0 and SAC2, `SAC1_SAC3` for SAC1 and SAC3 (DACIE: SLAU445I Table 20-8,
    /// p. 534; vectors: SLASEC4D Table 6-2, p. 64).
    /// Only the timer load triggers load values this way, so the interrupt is never requested with [`LoadTrigger::Immediate`].
    /// (SLAU445I 20.2.3.5, p. 529: "When DACLSELx = 0, the DAC12IFG flag is not set")
    /// Clear the request with [`Dac::data_loaded()`].
    #[inline(always)]
    pub fn configure_with_interrupts<'a>(self, vref: VRef<'a>, load_trigger: LoadTrigger<'_>) -> Dac<'a, SAC> {
        SAC::configure_dac(load_trigger.into(), vref.into(), true);
        Dac { sac: PhantomData, vref_lifetime: PhantomData }
    }
}

#[derive(Copy, Clone)]
/// Options for when the DAC loads in a new value placed in the DAC data register. (DACLSEL: SLAU445I
/// 20.2.3.4, p. 529, and SLAU445I Table 20-8, p. 534; the triggers: SLASEC4D Table 6-32, p. 80)
pub enum LoadTrigger<'a> {
    /// The DAC loads the new value as soon as the register is written to.
    Immediate,
    /// The DAC loads the new value when TB2.1 exhibits a rising edge.
    TB2_1(&'a SubTimer<Tb2, CCR1>),
    /// The DAC loads the new value when TB2.2 exhibits a rising edge.
    TB2_2(&'a SubTimer<Tb2, CCR2>),
}
// DACLSEL values (SLAU445I Table 20-8, p. 534; SLASEC4D Table 6-32, p. 80)
impl From<LoadTrigger<'_>> for u8 {
    #[inline(always)]
    fn from(value: LoadTrigger) -> Self {
        match value {
            LoadTrigger::Immediate => 0b00,
            //           Reserved:    0b01,
            LoadTrigger::TB2_1(_)  => 0b10,
            LoadTrigger::TB2_2(_)  => 0b11,
        }
    }
}

/// Defines which voltage reference the DAC uses (DACSREF: SLAU445I Table 20-8, p. 534; 0 is DVCC and 1 the
/// internal shared reference: SLASEC4D Table 6-31, p. 80)
#[derive(Debug, Copy, Clone)]
pub enum VRef<'a> {
    /// Use VCC as the DAC reference voltage.
    Vcc,
    /// Use the shared internal reference as the DAC reference voltage
    Internal(&'a InternalVRef),
}
impl From<VRef<'_>> for bool {
    #[inline(always)]
    fn from(value: VRef) -> Self {
        match value {
            VRef::Vcc         => false,
            VRef::Internal(_) => true,
        }
    }
}

/// The Digital to Analog Converter (DAC) inside this Smart Analog Combo (SAC) module.
#[derive(Debug)]
pub struct Dac<'a, SAC: SacPeriph> {
    sac: PhantomData<SAC>,
    vref_lifetime: PhantomData<VRef<'a>>, // If we use the internal reference, ensure it stays enabled for the life of the DAC.
}
impl<SAC: SacPeriph> Dac<'_, SAC> {
    /// Set the DAC count. This should be a value between 0 and 4095, where 0 is 0V, and 4095 is (just below) the DAC reference voltage.
    /// The value is masked with `0xFFF` before being written to the register.
    /// (SLAU445I 20.2.3.2, p. 529: Vout = Vref x DACDAT / 4096, and "A value greater than 4095 can be
    /// written to the register, but all leading bits are ignored")
    #[inline(always)]
    pub fn set_count(&mut self, count: u16) { SAC::set_dac_count(count); }

    /// Whether the DAC has loaded the value set with [`set_count()`](Self::set_count) since the last call (DACIFG),
    /// so the next value can be set. This clears the flag, and the interrupt request of
    /// [`DacConfig::configure_with_interrupts()`]. (SLAU445I 20.2.3.5, p. 529; SLAU445I Table 20-10, p. 536:
    /// "It can also be cleared by reading of SACxIV register")
    ///
    /// Only the timer load triggers set the flag: with [`LoadTrigger::Immediate`] this is always `false`
    /// (SLAU445I 20.2.3.5, p. 529).
    // SACxIV = 04h: "DAC channel update interrupt flag" (SLAU445I Table 20-11, p. 537)
    #[inline(always)]
    pub fn data_loaded(&mut self) -> bool { SAC::dac_iv() == 0x04 }
}

/// A builder for configuring a Smart Analog Combo (SAC) unit's amplifier
pub struct AmpConfig<MODE, SAC> {
    mode: PhantomData<MODE>,
    reg: PhantomData<SAC>,
}
impl<SAC: SacPeriph> AmpConfig<NoModeSet, SAC> {
    /// Begin configuring this SAC as an open-loop operational amplifier (no internal feedback). (GP mode:
    /// SLAU445I 20.2.2.1, p. 522)
    #[inline(always)]
    pub fn opamp(
        self,
        pos_in: PositiveInput<SAC>,
        // SLAU445I Table 20-1, p. 521, lists only OA- for this mode, but the data sheet's SAC channel tables
        // (SLASEC4D Table 6-27, p. 79, to SLASEC4D Table 6-30, p. 80: NSEL = 10) and SLAU445I 20.2.1.1,
        // p. 521, also connect the paired amplifier to the negative input, which no other mode can select.
        neg_in: NegativeInput<SAC>,
        power_mode: PowerMode,
    ) -> AmpConfig<ModeSet, SAC> {
        SAC::configure_sacoa(pos_in.psel(), neg_in.nsel(), power_mode.into());
        AmpConfig { mode: PhantomData, reg: PhantomData }
    }

    /// Begin configuring this SAC as an inverting amplifier. The positive input sets the bias of the output.
    /// (SLAU445I 20.2.2.4, p. 525: "The OA noninverting input can select from the external pin OAx+ or the
    /// 12-bit DAC as bias")
    #[inline(always)]
    pub fn inverting_amplifier(
        self,
        bias: BiasInput<SAC>,
        neg_in: NegativeInput<SAC>,
        gain: InvertingGain,
        power_mode: PowerMode,
    ) -> AmpConfig<ModeSet, SAC> {
        // MSEL = 00 (OA-) or 11 (paired OA) and NSEL = 01 (SLAU445I 20.2.2.4, p. 525, and SLAU445I
        // Table 20-1, p. 521)
        SAC::configure_sacpga(gain as u8, neg_in.msel());
        SAC::configure_sacoa(bias.psel(), NSel::Feedback, power_mode.into());
        AmpConfig { mode: PhantomData, reg: PhantomData }
    }

    /// Begin configuring this SAC as a non-inverting amplifier.
    #[inline(always)]
    pub fn noninverting_amplifier(
        self,
        pos_in: PositiveInput<SAC>,
        gain: NoninvertingGain,
        power_mode: PowerMode,
    ) -> AmpConfig<ModeSet, SAC> {
        // MSEL = 10 and NSEL = 01 (SLAU445I 20.2.2.5, p. 527)
        SAC::configure_sacpga(gain as u8, MSel::NonInverting);
        SAC::configure_sacoa(pos_in.psel(), NSel::Feedback, power_mode.into());
        AmpConfig { mode: PhantomData, reg: PhantomData }
    }

    /// Begin configuring this SAC as a unity-gain buffer / voltage follower.
    #[inline(always)]
    pub fn buffer(
        self,
        source: BufferInput<SAC>,
        power_mode: PowerMode,
    ) -> AmpConfig<ModeSet, SAC> {
        // MSEL = 01 and NSEL = 01, GAIN unused (SLAU445I 20.2.2.3, p. 524, and SLAU445I Table 20-2, p. 523)
        SAC::configure_sacpga(0, MSel::Follower);
        SAC::configure_sacoa(source.psel(), NSel::Feedback, power_mode.into());
        AmpConfig { mode: PhantomData, reg: PhantomData }
    }
}
impl<SAC: SacPeriph> AmpConfig<ModeSet, SAC> {
    /// Route the output of the amplifier to the GPIO pin (OAO in the module documentation's pin table)
    #[inline(always)]
    pub fn output_pin(self, _output_pin: impl Into<SAC::OutputPin>) -> Amplifier<SAC> {
        Amplifier(PhantomData)
    }
    /// Do not route the amplifier output to a GPIO pin.
    /// Useful if you only need the signal internally and don't want to give up a GPIO pin.
    #[inline(always)]
    pub fn no_output_pin(self) -> Amplifier<SAC> { Amplifier(PhantomData) }
}

/// List of possible sources for the amplifier's non-inverting input in the open-loop and non-inverting amplifier
/// modes. The user's guide doesn't support the DAC in these modes (SLAU445I Table 20-1, p. 521).
#[derive(Debug)]
pub enum PositiveInput<SAC: SacPeriph> {
    /// Use the GPIO pin labelled as OA+ as this amplifier's non-inverting input
    ExtPin(SAC::PosInputPin),
    /// Use the output of the paired SAC amplifier as this amplifier's non-inverting input.
    /// It is your responsibility to ensure this amplifier has been configured.
    // We can't require a reference to this Amplifier, as they could both refer to the other which would be impossible to instantiate
    PairedOpamp,
}
impl<SAC: SacPeriph> PositiveInput<SAC> {
    // PSEL (SLAU445I Table 20-6, p. 532)
    #[inline(always)]
    fn psel(&self) -> u8 {
        match self {
            PositiveInput::ExtPin(_)   => 0b00,
            PositiveInput::PairedOpamp => 0b10,
        }
    }
}

/// List of possible sources for the amplifier's non-inverting input in the inverting amplifier mode, which set the bias
/// of the output. The user's guide doesn't support the paired amplifier in this mode (SLAU445I Table 20-1,
/// p. 521).
#[derive(Debug)]
pub enum BiasInput<'a, SAC: SacPeriph> {
    /// Use the GPIO pin labelled as OA+ as this amplifier's non-inverting input
    ExtPin(SAC::PosInputPin),
    /// Use the SAC's Internal DAC as the amplifier's non-inverting input
    Dac(&'a Dac<'a, SAC>),
}
impl<SAC: SacPeriph> BiasInput<'_, SAC> {
    // PSEL (SLAU445I Table 20-6, p. 532)
    #[inline(always)]
    fn psel(&self) -> u8 {
        match self {
            BiasInput::ExtPin(_) => 0b00,
            BiasInput::Dac(_)    => 0b01,
        }
    }
}

/// List of possible sources for the amplifier's input in the buffer mode (SLAU445I 20.2.2.3, p. 524: "the
/// noninverting input from the external OAx+, DAC, or the output of paired OA")
#[derive(Debug)]
pub enum BufferInput<'a, SAC: SacPeriph> {
    /// Use the GPIO pin labelled as OA+ as the buffer input
    ExtPin(SAC::PosInputPin),
    /// Use the SAC's Internal DAC as the buffer input
    Dac(&'a Dac<'a, SAC>),
    /// Use the output of the paired SAC amplifier as the buffer input.
    /// It is your responsibility to ensure this amplifier has been configured.
    PairedOpamp,
}
impl<SAC: SacPeriph> BufferInput<'_, SAC> {
    // PSEL (SLAU445I Table 20-6, p. 532)
    #[inline(always)]
    fn psel(&self) -> u8 {
        match self {
            BufferInput::ExtPin(_)   => 0b00,
            BufferInput::Dac(_)      => 0b01,
            BufferInput::PairedOpamp => 0b10,
        }
    }
}

/// List of possible sources for the SAC amplifier's inverting input
// Note that this corresponds to a combination of NSEL and MSEL. In modes with feedback NSEL is always 0b01, so MSEL varies the negative input.
// (SLAU445I Table 20-1, p. 521)
#[derive(Debug)]
pub enum NegativeInput<SAC: SacPeriph> {
    /// Use the GPIO pin labelled as OA- as the amplifier's inverting input
    ExtPin(SAC::NegInputPin),
    /// Use the output of the paired SAC amplifier as this amplifier's inverting input
    PairedOpamp,
}
impl<SAC: SacPeriph> NegativeInput<SAC> {
    /// This corresponds to the input to the inverting opamp input when in open-loop mode (NSEL: SLAU445I
    /// Table 20-6, p. 532)
    #[inline(always)]
    fn nsel(&self) -> NSel {
        match self {
            NegativeInput::ExtPin(_)   => NSel::ExtPinMinus,
            NegativeInput::PairedOpamp => NSel::PairedOpamp,
        }
    }
    /// In modes with feedback this corresponds to whether the feedback divider is connected to OA- or the paired opamp output
    /// (MSEL: SLAU445I Table 20-7, p. 533: "00b = Inverting PGA mode (external pad OAx- is selected)", "11b =
    /// Cascade OA inverting mode")
    #[inline(always)]
    fn msel(&self) -> MSel {
        match self {
            NegativeInput::ExtPin(_)   => MSel::Inverting,
            NegativeInput::PairedOpamp => MSel::Cascade,
        }
    }
}

/// List of possible gain values when the SAC is in the inverting amplifier mode (GAIN: SLAU445I Table 20-2,
/// p. 523)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvertingGain {
    //    0b000 is not a valid value (SLAU445I Table 20-1, p. 521: GAIN 001-111 in the inverting mode)
    /// 1x gain
    _1  = 0b001,
    /// 2x gain
    _2  = 0b010,
    /// 4x gain
    _4  = 0b011,
    /// 8x gain
    _8  = 0b100,
    /// 16x gain
    _16 = 0b101,
    /// 25x gain
    _25 = 0b110,
    /// 32x gain
    _32 = 0b111,
}

/// List of possible gain values when the SAC is in the non-inverting amplifier mode (GAIN: SLAU445I
/// Table 20-2, p. 523)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoninvertingGain {
    /// 1x gain
    _1  = 0b000,
    /// 2x gain
    _2  = 0b001,
    /// 3x gain
    _3  = 0b010,
    /// 5x gain
    _5  = 0b011,
    /// 9x gain
    _9  = 0b100,
    /// 17x gain
    _17 = 0b101,
    /// 26x gain
    _26 = 0b110,
    /// 33x gain
    _33 = 0b111,
}

/// Power mode setting for the SAC. Controls power consumption and opamp slew rate (OAPM: SLAU445I Table 20-6,
/// p. 532)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PowerMode {
    /// High slew rate, high power consumption - 3 V/us @ 350 uA (slew rate and quiescent current: SLASEC4D
    /// Table 5-25, p. 55). The OA + DAC output load current is 1 mA typical (SLASEC4D Table 5-26, p. 56).
    HighPerformance,
    /// Low slew rate, low power consumption - 1 V/us @ 120 uA (slew rate and quiescent current: SLASEC4D
    /// Table 5-25, p. 55). The OA + DAC output load current is 0.2 mA typical (SLASEC4D Table 5-26, p. 56).
    LowPower,
}
impl From<PowerMode> for bool {
    #[inline(always)]
    fn from(value: PowerMode) -> Self {
        match value {
            PowerMode::HighPerformance => false,
            PowerMode::LowPower => true,
        }
    }
}

/// Represents an amplifier inside a Smart Analog Combo (SAC) that has been configured
pub struct Amplifier<SAC: SacPeriph>(PhantomData<SAC>);

/// Typestate for a SacConfig that has not been configured yet
pub struct NoModeSet;
/// Typestate for a SacConfig that has been configured for a particular mode
pub struct ModeSet;
