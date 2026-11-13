//! Analog to Digital Converter (ADC)
//!
//! Begin configuration by calling [`AdcConfig::new()`] or [`::default()`](AdcConfig::default()).
//! Once fully configured an [`Adc`] will be returned.
//!
//! [`Adc`] can read from a channel by calling [`read_count()`](Adc::read_count()) to return an ADC count.
//!
//! The [`count_to_mv()`](Adc::count_to_mv()) method is available to convert an ADC count to a voltage in millivolts,
//! given a reference voltage.
//!
//! As a convenience, [`read_voltage_mv()`](Adc::read_voltage_mv()) combines [`read_count()`](Adc::read_count()) and
//! [`count_to_mv()`](Adc::count_to_mv()).
//!
//! The ADC measures against AVCC, the operating voltage of the MSP430, unless
//! [`with_reference()`](Adc::with_reference()) selects the internal shared reference or an external one on the
//! VeREF+ (P1.0) and VeREF- (P1.2) pins (ADCSREFx, reset to AVCC and AVSS: SLAU445I Table 21-8, p. 567; the
//! pins are A0/Veref+ and A2/Veref- in the ADC channel tables listed below).
//!
//! Besides single conversions with [`read_count()`](Adc::read_count()), [`start()`](Adc::start()) converts a
//! sequence of channels and repeats conversions, started by software, the RTC, a timer or the comparator. The
//! window comparator ([`set_window()`](Adc::set_window())) flags results outside or inside a range, and
//! [`enable_interrupts()`](Adc::enable_interrupts()) requests the `ADC` interrupt for these events (SLAU445I
//! 21.2.7, p. 546: conversion modes; SLAU445I Table 21-4, p. 563, and the trigger tables cited at
//! `TriggerSource`: triggers; SLAU445I 21.2.7.7, p. 555: window comparator; SLAU445I 21.2.7.10, p. 558:
//! interrupts).
//!
//! [`read_count()`](Adc::read_count()) takes a reference to the GPIO pin corresponding to the relevant ADC channel
//! to ensure it's been correctly configured. The ADC inputs are (data sheets, pin function tables):
//!
//! | Device       | Resolution | Channels 0 to 7 | Channels 8 to 11            | Pin mode          |
//! |--------------|------------|-----------------|-----------------------------|-------------------|
//! | MSP430FR2x5x | 12-bit     | P1.0 to P1.7    | P5.0 to P5.3                | `to_alternate3()` |
//! | MSP430FR247x | 12-bit     | P1.0 to P1.7    | P4.3, P4.4, P5.3, P5.4      | `to_alternate3()` |
//! | MSP430FR2433 | 10-bit     | P1.0 to P1.7    |                             | `to_adc_mode()`   |
//! | MSP430FR25x2 | 10-bit     | P1.0 to P1.3, P2.2 to P2.5 |                  | `to_adc_mode()`   |
//!
//! The table follows each data sheet's ADC section with its ADC Channel Connections table, and its pin
//! function tables (`to_alternate3()` is PxSELx = 11 there, the tertiary module function: SLAU445I
//! Table 8-3, p. 314):
//! - MSP430FR2x5x: SLASEC4D 6.10.12, p. 77; SLASEC4D Table 6-21, p. 77; SLASEC4D Table 6-63, p. 96;
//!   SLASEC4D Table 6-67, p. 104
//! - MSP430FR247x: SLASEO7C 9.10.12, p. 62; SLASEO7C Table 9-19, p. 62; SLASEO7C Table 9-23, p. 65;
//!   SLASEO7C Table 9-26, p. 68; SLASEO7C Table 9-27, p. 69
//! - MSP430FR2433: SLASE59F 6.10.12, p. 53; SLASE59F Table 6-15, p. 53; SLASE59F Table 6-17, p. 55
//! - MSP430FR25x2: SLASEE4C 6.10.12, p. 55; SLASEE4C Table 6-13, p. 55 to p. 56; SLASEE4C Table 6-15,
//!   p. 58; SLASEE4C Table 6-16, p. 60
//!
//! On the MSP430FR2433 and MSP430FR25x2 the analog inputs are enabled through SYSCFG2.ADCPCTLx instead of the
//! pin's function select bits, which is what `to_adc_mode()` does (SLASE59F Table 6-17 note 2, p. 55;
//! SLASEE4C Table 6-15, p. 58; SLAU445I Table 1-31, p. 82).
//!
//! ADC channels 12 to 15 are not associated with external pins, so instead channels 12 and 13 can be read by passing a
//! reference to [`InternalTempSensor`] or [`InternalVRef`] respectively. Channels 14 and 15 require no prior
//! configuration, so the two functions below provide a reference that can be used to read from these channels.
//! The ADC channel tables listed above give channel 12 as the temperature sensor, 13 as the internal
//! reference, 14 as DVSS and 15 as DVCC.

use crate::_pac;
use crate::{
    clock::{Aclk, Smclk},
    pmm::{InternalTempSensor, InternalVRef, VrefOutput},
};
use core::{convert::Infallible, marker::PhantomData};

#[cfg(feature = "embedded-hal-02")]
pub use embedded_hal_02::adc::Channel;

#[cfg(not(feature = "embedded-hal-02"))]
/// A marker trait to identify MCU pins that can be used as inputs to an ADC channel.
///
/// This marker trait denotes an object, i.e. a GPIO pin, that is ready for use as an input to the
/// ADC. As ADCs channels can be supplied by multiple pins, this trait defines the relationship
/// between the physical interface and the ADC sampling buffer.
///
/// ```
/// # use std::marker::PhantomData;
/// # use embedded_hal::adc::Channel;
///
/// struct Adc1; // Example ADC with single bank of 8 channels
/// struct Gpio1Pin1<MODE>(PhantomData<MODE>);
/// struct Analog(()); // marker type to denote a pin in "analog" mode
///
/// // GPIO 1 pin 1 can supply an ADC channel when it is configured in Analog mode
/// impl Channel<Adc1> for Gpio1Pin1<Analog> {
///     fn channel() -> u8 { 7 } // GPIO pin 1 is connected to ADC channel 7
/// }
/// ```
pub trait Channel<ADC> {
    /// Type denoting the method used to identify ADC channels. This may be an integer (e.g. single ADC bank), or a tuple (multiple ADC banks), etc.
    /// On the MSP430 this is a `u8`, but this type remains generic for compatibility reasons with embedded-hal v0.2.7.
    type ID;
    /// Channel ID type
    ///
    /// A type used to identify this ADC channel. For example, if the ADC has eight channels, this
    /// might be a `u8`. If the ADC has multiple banks of channels, it could be a tuple, like
    /// `(u8: bank_id, u8: channel_id)`.
    /// Get the specific ID that identifies this channel, for example `0_u8` for the first ADC channel
    fn channel() -> u8;
}
/// Marker trait that marks a pin as being capable of being an ADC input via ADCPCTLx (SYSCFG2: SLAU445I
/// Table 1-31, p. 82).
// This trait is used to mark a pin as being capable of moving between PxSEL modes and ADCPCTLx mode.
pub trait AdcPctlCapable {
    /// The corresponding ADCPCTL bit that represents this pin: ADCPCTLx enables input Ax (SLAU445I
    /// Table 1-31, p. 82).
    const ADCPCTLX: u8;
}

/// How many ADCCLK cycles the ADC's sample-and-hold stage will last for (ADCSHTx: SLAU445I Table 21-3,
/// p. 561).
///
/// Default: 8 cycles
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SampleTime {
    /// Sample for 4 ADCCLK cycles
    _4 = 0b0000,
    /// Sample for 8 ADCCLK cycles
    #[default]
    _8 = 0b0001,
    /// Sample for 16 ADCCLK cycles
    _16 = 0b0010,
    /// Sample for 32 ADCCLK cycles
    _32 = 0b0011,
    /// Sample for 64 ADCCLK cycles
    _64 = 0b0100,
    /// Sample for 96 ADCCLK cycles
    _96 = 0b0101,
    /// Sample for 128 ADCCLK cycles
    _128 = 0b0110,
    /// Sample for 192 ADCCLK cycles
    _192 = 0b0111,
    /// Sample for 256 ADCCLK cycles
    _256 = 0b1000,
    /// Sample for 384 ADCCLK cycles
    _384 = 0b1001,
    /// Sample for 512 ADCCLK cycles (ADCSHTx = 1010b: SLAU445I Table 21-3, p. 561)
    _512 = 0b1010,
    /// Sample for 768 ADCCLK cycles (ADCSHTx = 1011b: SLAU445I Table 21-3, p. 561)
    _768 = 0b1011,
    /// Sample for 1024 ADCCLK cycles (ADCSHTx = 1100b; 1101b to 1111b give 1024 cycles too: SLAU445I
    /// Table 21-3, p. 561)
    _1024 = 0b1100,
}

impl SampleTime {
    #[inline(always)]
    fn adcsht(self) -> u8 { self as u8 }
}

/// How much the ADC input clock will be divided by after being divided by the predivider (ADCDIVx: SLAU445I
/// Table 21-4, p. 563)
///
/// Default: Divide by 1
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum ClockDivider {
    /// Divide the input clock by 1
    #[default]
    _1 = 0b000,
    /// Divide the input clock by 2
    _2 = 0b001,
    /// Divide the input clock by 3
    _3 = 0b010,
    /// Divide the input clock by 4
    _4 = 0b011,
    /// Divide the input clock by 5
    _5 = 0b100,
    /// Divide the input clock by 6
    _6 = 0b101,
    /// Divide the input clock by 7
    _7 = 0b110,
    /// Divide the input clock by 8
    _8 = 0b111,
}

impl ClockDivider {
    #[inline(always)]
    fn adcdiv(self) -> u8 { self as u8 }
}

// ADCSSELx (SLAU445I Table 21-4, p. 564)
#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum ClockSource {
    /// Use MODCLK as the ADC input clock
    #[default]
    ModClk = 0b00,
    /// Use ACLK as the ADC input clock
    AClk = 0b01,
    /// Use SMCLK as the ADC input clock
    SmClk = 0b10,
}

impl ClockSource {
    #[inline(always)]
    fn adcssel(self) -> u8 { self as u8 }
}

/// How much the ADC input clock will be divided by prior to being divided by the ADC clock divider (ADCPDIVx:
/// SLAU445I Table 21-5, p. 565)
///
/// Default: Divide by 1
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Predivider {
    /// Divide the input clock by 1
    #[default]
    _1 = 0b00,
    /// Divide the input clock by 4
    _4 = 0b01,
    /// Divide the input clock by 64
    _64 = 0b10,
}

impl Predivider {
    #[inline(always)]
    fn adcpdiv(self) -> u8 { self as u8 }
}

/// The output resolution of the ADC conversion. Also determines how many ADCCLK cycles the conversion step takes
/// (ADCRES: SLAU445I Table 21-5, p. 565).
///
/// Default: 10-bit resolution
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// 8-bit ADC conversion result. The conversion step takes 10 ADCCLK cycles.
    _8BIT = 0b00,
    /// 10-bit ADC conversion result. The conversion step takes 12 ADCCLK cycles.
    #[default]
    _10BIT = 0b01,
    #[cfg(feature = "adc12bit")]
    /// 12-bit ADC conversion result. The conversion step takes 14 ADCCLK cycles.
    _12BIT = 0b10,
}

impl Resolution {
    #[inline(always)]
    fn adcres(self) -> u8 { self as u8 }
}

/// Selects the drive capability of the ADC reference buffer, which can increase the maximum sampling speed at the cost of increased power draw.
/// (ADCSR: SLAU445I Table 21-5, p. 565, and SLAU445I 21.2.3.1, p. 542)
///
/// Default: 200ksps
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SamplingRate {
    /// Maximum of 50 ksps. Lower power usage.
    _50KSPS,
    /// Maximum of 200 ksps. Higher power usage.
    #[default]
    _200KSPS,
}

impl SamplingRate {
    #[inline(always)]
    fn adcsr(self) -> bool {
        match self {
            SamplingRate::_200KSPS => false,
            SamplingRate::_50KSPS => true,
        }
    }
}

/// How conversion results and window comparator thresholds are formatted (ADCDF: SLAU445I Table 21-5,
/// p. 565). [`AdcConfig`] defaults to `Unsigned`.
///
/// - `Unsigned`: right-aligned, from 0 at VR- up to 255, 1023 or 4095 at VR+ (SLAU445I 21.2.1, p. 541, and
///   SLAU445I 21.3.4, p. 566).
/// - `Signed`: two's complement and left-aligned, as an `i16` would read it: from -32768 at VR- up to just
///   below 32768 at VR+. The low bits are 0: 8 bits for an 8-bit result, 6 for 10-bit and 4 for 12-bit
///   (SLAU445I 21.3.5, p. 566; ADCDF in SLAU445I Table 21-5, p. 565: "-VREF results in 8000h, and ... +VREF
///   results in 7FC0h").
///
/// (The PACs name the ADCDF values differently, and the MSP430FR2433 PAC has no enum for it at all, so
/// this enum is written as a bit.)
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum DataFormat {
    /// Right-aligned binary
    #[default]
    Unsigned,
    /// Left-aligned two's complement
    Signed,
}

// Pins corresponding to an ADC channel. Pin types can have `::channel()` called on them to get their ADC channel index.
macro_rules! impl_adc_channel_pin {
    ($port: ty, $pin: ty, $mode:tt => $channel: literal ) => {
        impl<DIR> Channel<Adc> for Pin<$port, $pin, $mode<DIR>> {
            type ID = u8;

            fn channel() -> Self::ID { $channel }
        }
        // On the MSP430FR2433 and MSP430FR25x2 ADC functionality is done via ADCPCTLx instead of
        // Alternate1/2/3 (SLASE59F Table 6-17, p. 55; SLASEE4C Table 6-15, p. 58, and SLASEE4C Table 6-16,
        // p. 60). The MSP430FR247x has no SAC either, but uses PxSELx = 11 (SLASEO7C Table 9-23, p. 65).
        // ADCPCTLx enables input Ax (SLAU445I Table 1-31, p. 82).
        // Implement this for all modes
        #[cfg(feature = "adcpctl")]
        impl<MODE> AdcPctlCapable for Pin<$port, $pin, MODE> {
            const ADCPCTLX: u8 = $channel;
        }
    };
}
pub(crate) use impl_adc_channel_pin;

// A few ADC channels don't correspond to pins.
macro_rules! impl_adc_channel_extra {
    ($type: ty, $channel: literal ) => {
        impl Channel<Adc> for $type {
            type ID = u8;

            fn channel() -> Self::ID { $channel }
        }
    };
}

// Channel 12 is the temperature sensor, 13 the internal reference (SLASEC4D Table 6-21, p. 77; SLASEO7C
// Table 9-19, p. 62; SLASE59F Table 6-15, p. 53; SLASEE4C Table 6-13, p. 55; ADCINCHx = 1100b for the
// sensor: SLAU445I 21.2.7.8, p. 556)
impl_adc_channel_extra!(InternalTempSensor<'_>, 12);
impl_adc_channel_extra!(InternalVRef, 13);

// The VREF+ output is measured through its pin's channel (SLASEC4D 6.10.1, p. 67: "ADC channel 7 can also be
// selected to monitor this voltage"; SLASEO7C Table 9-19 note 1, p. 62; SLASE59F 6.10.1, p. 45; SLASEE4C
// 6.10.1, p. 49)
impl<PIN: Channel<Adc, ID = u8>> Channel<Adc> for VrefOutput<PIN> {
    type ID = u8;

    fn channel() -> Self::ID { PIN::channel() }
}

// Users needn't deal with the structs themselves so it just adds noise to the docs. We instead document the functions below.
#[doc(hidden)]
pub struct AdcVssChannel;
impl_adc_channel_extra!(AdcVssChannel, 14);
/// ADC channel 14, tied to VSS (DVSS: SLASEC4D Table 6-21, p. 77; SLASEO7C Table 9-19, p. 62; SLASE59F
/// Table 6-15, p. 53; SLASEE4C Table 6-13, p. 56). Pass this function's output to `read_count()`.
#[inline(always)]
pub fn adc_ch14_vss() -> AdcVssChannel { AdcVssChannel }

#[doc(hidden)]
pub struct AdcVccChannel;
impl_adc_channel_extra!(AdcVccChannel, 15);
/// ADC channel 15, tied to VCC (DVCC: SLASEC4D Table 6-21, p. 77; SLASEO7C Table 9-19, p. 62; SLASE59F
/// Table 6-15, p. 53; SLASEE4C Table 6-13, p. 56). Pass this function's output to `read_count()`.
#[inline(always)]
pub fn adc_ch15_vcc() -> AdcVccChannel { AdcVccChannel }

/// Typestate for an ADC configuration with no clock source selected
pub struct NoClockSet;
/// Typestate for an ADC configuration with a clock source selected
pub struct ClockSet(ClockSource);

/// `x / (2^BITS - 1)`, rounded down, from shifts: the MSP430 has no divider, and a library division
/// takes several hundred cycles. `(x + x / 2^BITS) / 2^BITS` is never above the quotient, and at most
/// one below it for the products `count_to_mv` divides; the loop makes up the difference.
#[inline(always)]
fn div_by_full_scale<const BITS: u32>(x: u32) -> u32 {
    let full_scale = (1 << BITS) - 1;
    let mut quotient = (x + (x >> BITS)) >> BITS;
    let mut remainder = x - ((quotient << BITS) - quotient);
    while remainder >= full_scale {
        quotient += 1;
        remainder -= full_scale;
    }
    quotient
}

/// Configuration object for an ADC.
///
/// The default configuration is based on the default register values (SLAU445I Table 21-3, p. 561 to
/// p. 562; SLAU445I Table 21-4, p. 563 to p. 564; SLAU445I Table 21-5, p. 565):
/// - Predivider = 1 and clock divider = 1
/// - 10-bit resolution
/// - 8 cycle sample time, the reset value of the 12-bit ADC (the 10-bit ADC of the MSP430FR2433 and
///   MSP430FR25x2 resets to 4 cycles: SLAU445I Table 21-3 note 1, p. 561)
/// - Max 200 ksps sample rate
/// - Unsigned results
#[derive(Clone, PartialEq, Eq)]
pub struct AdcConfig<STATE> {
    state: STATE,
    /// How much the input clock is divided by, after the predivider (ADCDIVx: SLAU445I Table 21-4, p. 563).
    pub clock_divider: ClockDivider,
    /// How much the input clock is initially divided by, before the clock divider (ADCPDIVx: SLAU445I
    /// Table 21-5, p. 565).
    pub predivider: Predivider,
    /// How many bits the conversion result is. Also defines the number of ADCCLK cycles required to do the conversion step.
    /// (ADCRES: SLAU445I Table 21-5, p. 565)
    pub resolution: Resolution,
    /// Sets the maximum sampling rate of the ADC. Lower values use less power. (ADCSR: SLAU445I Table 21-5,
    /// p. 565)
    pub sampling_rate: SamplingRate,
    /// Determines the number of ADCCLK cycles the sampling time takes (ADCSHTx: SLAU445I Table 21-3, p. 561).
    pub sample_time: SampleTime,
    /// The format of conversion results and window comparator thresholds (ADCDF: SLAU445I Table 21-5,
    /// p. 565).
    pub data_format: DataFormat,
}

// Only implement Default for NoClockSet
impl Default for AdcConfig<NoClockSet> {
    fn default() -> Self {
        Self {
            state: NoClockSet,
            clock_divider: Default::default(),
            predivider: Default::default(),
            resolution: Default::default(),
            sampling_rate: Default::default(),
            sample_time: Default::default(),
            data_format: DataFormat::Unsigned,
        }
    }
}

impl AdcConfig<NoClockSet> {
    /// Creates an ADC configuration. A default implementation is also available through `::default()`
    pub fn new(
        clock_divider: ClockDivider,
        predivider: Predivider,
        resolution: Resolution,
        sampling_rate: SamplingRate,
        sample_time: SampleTime,
    ) -> AdcConfig<NoClockSet> {
        AdcConfig {
            state: NoClockSet,
            clock_divider,
            predivider,
            resolution,
            sampling_rate,
            sample_time,
            data_format: DataFormat::Unsigned,
        }
    }
    /// Configure the ADC to use SMCLK (ADCSSELx: SLAU445I Table 21-4, p. 564)
    pub fn use_smclk(self, _smclk: &Smclk) -> AdcConfig<ClockSet> {
        AdcConfig {
            state: ClockSet(ClockSource::SmClk),
            clock_divider: self.clock_divider,
            predivider: self.predivider,
            resolution: self.resolution,
            sampling_rate: self.sampling_rate,
            sample_time: self.sample_time,
            data_format: self.data_format,
        }
    }
    /// Configure the ADC to use ACLK (ADCSSELx: SLAU445I Table 21-4, p. 564)
    ///
    /// On the MSP430FR2433 and MSP430FR25x2, temperature sensor readings taken in LPM3 with ACLK as the
    /// ADC clock may be wrong: "When ACLK is used as ADC clock source and device is in LPM3 mode while
    /// sampling the on-chip temperature sensor, the ADC may generate erroneous conversion results". The
    /// erratum's workarounds are SMCLK or MODCLK as the ADC clock, with "A 100us sampling time" if the
    /// conversion is triggered from LPM3, or LPM0 or active mode (SLAZ664S ADC50; SLAZ705H ADC50).
    pub fn use_aclk(self, _aclk: &Aclk) -> AdcConfig<ClockSet> {
        AdcConfig {
            state: ClockSet(ClockSource::AClk),
            clock_divider: self.clock_divider,
            predivider: self.predivider,
            resolution: self.resolution,
            sampling_rate: self.sampling_rate,
            sample_time: self.sample_time,
            data_format: self.data_format,
        }
    }
    /// Configure the ADC to use MODCLK (ADCSSELx: SLAU445I Table 21-4, p. 564)
    pub fn use_modclk(self) -> AdcConfig<ClockSet> {
        AdcConfig {
            state: ClockSet(ClockSource::ModClk),
            clock_divider: self.clock_divider,
            predivider: self.predivider,
            resolution: self.resolution,
            sampling_rate: self.sampling_rate,
            sample_time: self.sample_time,
            data_format: self.data_format,
        }
    }
}
impl AdcConfig<ClockSet> {
    /// Applies this ADC configuration to hardware registers, and returns an ADC.
    pub fn configure(self, mut adc_reg: _pac::Adc) -> Adc {
        // Disable the ADC before we set the other bits. Some can only be set while the ADC is disabled.
        // (SLAU445I 21.2.1, p. 541: "the ADC control bits can be modified only when ADCENC = 0")
        disable_adc_reg(&mut adc_reg);

        let adcsht = self.sample_time.adcsht();
        adc_reg.adcctl0().write(|w| unsafe { w.adcsht().bits(adcsht) });
        // AVCC and AVSS as reference, as the returned `Adc` says, and channel 0 (ADCSREFx = 000b,
        // ADCINCHx = 0: SLAU445I Table 21-8, p. 567)
        adc_reg.adcmctl0().write(|w| unsafe { w.bits(0) });

        let adcssel = self.state.0.adcssel();
        let adcdiv = self.clock_divider.adcdiv();
        // ADCSHP = 1: the sampling timer sets the sample time, pulse sample mode (SLAU445I 21.2.5.2, p. 544)
        adc_reg.adcctl1().write(|w| { unsafe { w
            .adcssel().bits(adcssel)
            .adcshp().set_bit()
            .adcdiv().bits(adcdiv) 
        }});

        let adcpdiv = self.predivider.adcpdiv();
        let adcres = self.resolution.adcres();
        let adcsr = self.sampling_rate.adcsr();
        adc_reg.adcctl2().write(|w| { unsafe { w
            .adcpdiv().bits(adcpdiv)
            .adcres().bits(adcres)
            .adcdf().bit(self.data_format == DataFormat::Signed)
            .adcsr().bit(adcsr)
        }});

        Adc { adc_reg, pending: None, reference: PhantomData }
    }
}

/// Typestate for an ADC that measures against AVCC and AVSS, as after reset (ADCSREFx: SLAU445I Table 21-8,
/// p. 567)
pub struct AvccReference;
/// Typestate for an ADC with a reference selected by [`Adc::with_reference()`], which borrows the
/// internal reference or the VeREF pins for `'a`
pub struct SelectedReference<'a>(PhantomData<&'a ()>);

/// Marker trait for the VeREF+ pin (P1.0) in its analog mode, which supplies an external positive reference
/// (A0/Veref+: SLASEC4D Table 6-21, p. 77; SLASEO7C Table 9-19, p. 62; SLASE59F Table 6-15, p. 53; SLASEE4C
/// Table 6-13, p. 55)
pub trait VeRefPlusPin {}
/// Marker trait for the VeREF- pin (P1.2) in its analog mode, which supplies an external negative reference
/// (A2/Veref-: SLASEC4D Table 6-21, p. 77; SLASEO7C Table 9-19, p. 62; SLASE59F Table 6-15, p. 53; SLASEE4C
/// Table 6-13, p. 55)
pub trait VeRefMinusPin {}

/// The positive reference of the ADC, VR+: an input at or above it converts to the full-scale count (ADCSREF,
/// SLAU445I 21.2.1, p. 541, SLAU445I 21.2.3, p. 542, and SLAU445I Table 21-8, p. 567)
pub enum PositiveReference<'a> {
    /// AVCC, as after reset
    Avcc,
    /// The internal shared reference, see [`Pmm::enable_internal_reference()`](crate::pmm::Pmm::enable_internal_reference)
    Internal(&'a InternalVRef),
    /// An external reference on the VeREF+ pin, through the ADC's reference buffer
    ExternalBuffered(&'a dyn VeRefPlusPin),
    /// An external reference on the VeREF+ pin, unbuffered
    External(&'a dyn VeRefPlusPin),
}

/// The negative reference of the ADC, VR-: an input at or below it converts to 0 (ADCSREF, SLAU445I 21.2.1,
/// p. 541, SLAU445I 21.2.3, p. 542, and SLAU445I Table 21-8, p. 567)
pub enum NegativeReference<'a> {
    /// AVSS, as after reset
    Avss,
    /// An external reference on the VeREF- pin
    External(&'a dyn VeRefMinusPin),
}

/// How conversions repeat (ADCCONSEQ, SLAU445I 21.2.7, p. 546, and SLAU445I Table 21-1, p. 546).
/// [`ConversionConfig`] defaults to `Single`.
///
/// - `Single`: convert the channel once. With a hardware trigger, start again for the next conversion
///   (SLAU445I 21.2.7.1, p. 547: "When any other trigger source is used, ADCENC must be toggled between
///   each conversion").
/// - `Sequence`: convert the channels from the selected one down to channel 0, once (SLAU445I 21.2.7.2,
///   p. 549).
/// - `RepeatSingle`: convert the channel once for each trigger, until stopped (SLAU445I 21.2.7.3, p. 551).
/// - `RepeatSequence`: convert the channels from the selected one down to channel 0 for each trigger,
///   until stopped (SLAU445I 21.2.7.4, p. 553).
///
/// (The PACs name the ADCCONSEQ values differently, so this enum is written as its 2 bits.)
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ConversionMode {
    /// One channel, once
    #[default]
    Single = 0b00,
    /// A sequence of channels, once
    Sequence = 0b01,
    /// One channel, repeatedly
    RepeatSingle = 0b10,
    /// A sequence of channels, repeatedly
    RepeatSequence = 0b11,
}

/// Marker trait for the timer whose capture/compare register 1 output starts conversions with
/// [`TriggerSource::Timer`]: TB1 on the MSP430FR2x5x, TA1 on the other devices (ADC Trigger Signal
/// Connections: SLASEC4D Table 6-22, p. 77; SLASEO7C Table 9-20, p. 62; SLASE59F Table 6-16, p. 53; SLASEE4C
/// Table 6-14, p. 56)
pub trait AdcTriggerTimer {}

/// What starts conversions (ADCSHS: SLAU445I Table 21-4, p. 563; ADC Trigger Signal Connections: SLASEC4D
/// Table 6-22, p. 77; SLASEO7C Table 9-20, p. 62; SLASE59F Table 6-16, p. 53; SLASEE4C Table 6-14, p. 56)
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TriggerSource {
    /// Software, through [`Adc::start()`] (ADCSC: SLAU445I Table 21-3, p. 562)
    #[default]
    Software,
    /// RTC counter overflows (SLASEC4D 6.10.11, p. 76; SLASEO7C 9.10.11, p. 61; SLASE59F 6.10.11, p. 52;
    /// SLASEE4C 6.10.11, p. 55: "The RTC overflow events trigger ... ADC conversion trigger")
    Rtc,
    /// The output of capture/compare register 1 of TB1 on the MSP430FR2x5x, or TA1 on the other devices. Set
    /// that timer up for PWM, with a pin or with
    /// [`PwmUninit::into_adc_trigger()`](crate::pwm::PwmUninit::into_adc_trigger).
    /// (CCR1 "To ADC trigger": SLASEC4D Table 6-17, p. 74; SLASEO7C Table 9-13, p. 56; SLASE59F Table 6-12,
    /// p. 51; SLASEE4C Figure 6-2, p. 54)
    Timer,
    /// The output of eCOMP0 (eCOMP0 COUT: SLASEC4D Table 6-22, p. 77; SLASEO7C Table 9-20, p. 62)
    #[cfg(feature = "ecomp")]
    Comparator,
}

/// How a trigger controls the sampling (ADCSHP, ADCISSH, SLAU445I 21.2.5, p. 542 to p. 543, and SLAU445I
/// Table 21-4, p. 563)
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SampleMode {
    /// A rising edge starts sampling for the configured sample time (pulse sample mode, SLAU445I 21.2.5.2,
    /// p. 544)
    #[default]
    RisingEdge,
    /// A falling edge starts sampling for the configured sample time (pulse sample mode, inverted trigger)
    FallingEdge,
    /// Sample while the trigger is high, and convert when it goes low (extended sample mode). The trigger must
    /// stay high for at least 4 ADCCLK cycles. Hardware triggers only. (SLAU445I 21.2.5.1, p. 543: "The SHI
    /// signal requires at least 4 ADCCLK cycles")
    WhileHigh,
    /// Sample while the trigger is low, and convert when it goes high (extended sample mode, inverted trigger).
    /// Hardware triggers only.
    WhileLow,
}

/// Settings for [`Adc::start()`]. The default converts once, started by software.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ConversionConfig {
    /// How conversions repeat
    pub mode: ConversionMode,
    /// What starts the conversions
    pub trigger: TriggerSource,
    /// How the trigger controls sampling
    pub sample_mode: SampleMode,
    /// In the sequence and repeat modes, convert back to back after the first trigger, as fast as possible,
    /// instead of waiting for a trigger for each conversion (ADCMSC). In the repeat modes the conversions
    /// then continue until stopped. (SLAU445I 21.2.7.5, p. 555)
    pub back_to_back: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        ConversionConfig {
            mode: ConversionMode::Single,
            trigger: TriggerSource::default(),
            sample_mode: SampleMode::default(),
            back_to_back: false,
        }
    }
}

bitflags::bitflags! {
    /// ADC interrupt sources, for [`Adc::enable_interrupts()`] and [`Adc::interrupt_flags()`] (ADCIE, ADCIFG:
    /// SLAU445I Table 21-13, p. 570, and SLAU445I Table 21-14, p. 571)
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct AdcInterruptFlags: u16 {
        /// ADCIFG0. A conversion result is ready. Reading it clears this flag.
        const ResultReady  = 1 << 0;
        /// ADCINIFG. The result is inside the window: from the low threshold up to the high threshold
        /// (SLAU445I 21.2.7.7, p. 555: "between the low threshold ... and the high threshold").
        const InsideWindow = 1 << 1;
        /// ADCLOIFG. The result is below the low threshold of the window.
        const BelowWindow  = 1 << 2;
        /// ADCHIIFG. The result is above the high threshold of the window.
        const AboveWindow  = 1 << 3;
        /// ADCOVIFG. A result overwrote one that hadn't been read.
        const Overflow     = 1 << 4;
        /// ADCTOVIFG. A trigger arrived before the conversion had finished.
        const TimeOverflow = 1 << 5;
    }
}

/// The highest-priority pending ADC interrupt, as read from ADCIV by [`Adc::interrupt_source()`] (SLAU445I
/// Table 21-15, p. 572)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdcVector {
    /// No interrupt pending
    None,
    /// A result overwrote one that hadn't been read
    Overflow,
    /// A trigger arrived before the conversion had finished
    TimeOverflow,
    /// The result is above the high threshold of the window
    AboveWindow,
    /// The result is below the low threshold of the window
    BelowWindow,
    /// The result is inside the window
    InsideWindow,
    /// A conversion result is ready. This flag stays set until the result is read (SLAU445I 21.2.7.10.1,
    /// p. 558: "Only the ADCIFG0 is not reset by this ADCIV read access").
    ResultReady,
}

/// Controls the onboard ADC. The `read()` method is available through the embedded_hal `OneShot` trait.
///
/// `REF` tracks the reference selected with [`Adc::with_reference()`].
pub struct Adc<REF = AvccReference> {
    adc_reg: _pac::Adc,
    /// Channel of the conversion that was started but not read yet
    pending: Option<u8>,
    reference: PhantomData<REF>,
}

impl<REF> Adc<REF> {
    /// Whether the ADC is currently sampling or converting (ADCBUSY: SLAU445I Table 21-4, p. 564).
    pub fn adc_is_busy(&self) -> bool {
        self.adc_reg.adcctl1().read().adcbusy().bit_is_set()
    }

    /// Gets the latest ADC conversion result (ADCMEM0: SLAU445I 21.3.4, p. 566).
    pub fn adc_get_result(&self) -> u16 { self.adc_reg.adcmem0().read().bits() }

    /// Enables this ADC, ready to start conversions (ADCON: SLAU445I Table 21-3, p. 561).
    pub fn enable(&mut self) {
        unsafe {
            self.adc_reg.adcctl0().set_bits(|w| w.adcon().set_bit());
        }
    }

    /// Disables this ADC to save power (SLAU445I 21.2.1, p. 541: "The ADC can be turned off when not in
    /// use to save power").
    pub fn disable(&mut self) { disable_adc_reg(&mut self.adc_reg); }

    /// Selects which pin to sample (ADCINCHx: SLAU445I Table 21-8, p. 567).
    fn set_pin<PIN>(&mut self, _pin: &PIN)
    where PIN: Channel<Adc, ID = u8> {
        self.adc_reg.adcmctl0().modify(|_, w|
            unsafe { w.adcinch().bits(PIN::channel()) }
        );
    }

    /// Starts an ADC conversion (SLAU445I Table 21-3, p. 562: "ADCSC and ADCENC may be set together with one
    /// instruction").
    fn start_conversion(&mut self) {
        unsafe {
            self.adc_reg.adcctl0().set_bits(|w| w
                .adcenc().set_bit()
                .adcsc().set_bit());
        }
    }

    /// Begins a single ADC conversion if one isn't already underway, enabling the ADC in the process.
    ///
    /// If the result is ready it is returned as an ADC count, otherwise returns `WouldBlock`
    ///
    /// A conversion that is still pending for another channel is finished first and its result
    /// discarded.
    pub fn read_count<PIN>(&mut self, pin: &mut PIN) -> nb::Result<u16, Infallible>
    where PIN: Channel<Adc, ID = u8> {
        if let Some(pending) = self.pending {
            if self.adc_is_busy() {
                return Err(nb::Error::WouldBlock);
            }
            self.pending = None;
            if pending == PIN::channel() {
                return Ok(self.adc_get_result());
            }
        }
        self.disable();
        // A single conversion started by software, as `start()` may have set otherwise (ADCSHSx = 00,
        // ADCISSH = 0, ADCCONSEQx = 00, ADCSHP = 1: SLAU445I Table 21-4, p. 563 to p. 564; ADCMSC = 0:
        // SLAU445I Table 21-3, p. 561)
        self.adc_reg.adcctl1().modify(|_, w| w.adcshs().set(0).adcissh().clear_bit().adcconseq().set(0).adcshp().set_bit());
        self.adc_reg.adcctl0().modify(|_, w| w.adcmsc().clear_bit());
        self.set_pin(pin);
        self.enable();

        self.start_conversion();
        self.pending = Some(PIN::channel());
        Err(nb::Error::WouldBlock)
    }

    /// Convert an ADC count to a voltage value in millivolts, rounded down.
    ///
    /// `ref_voltage_mv` is the reference voltage of the ADC in millivolts. The full-scale count
    /// (255, 1023 or 4095) corresponds to the reference voltage, as in the data sheets' DVCC equation
    /// (SLASEC4D 6.10.1, p. 67: DVCC = 4095 x reference voltage / ADC result; with 1023 in
    /// SLASEO7C 9.10.1, p. 49, SLASE59F 6.10.1, p. 45, and SLASEE4C 6.10.1, p. 48) and in the ADCDF
    /// description (SLAU445I Table 21-5, p. 565: "+VREF results in 03FFh"). The ADC conversion formula
    /// of SLAU445I 21.2.1, p. 541, has 1024 or 4096 instead, a difference of at most 1 LSB. With an
    /// external negative reference, this is the voltage above VR-. A count in the signed [`DataFormat`]
    /// is converted too.
    pub fn count_to_mv(&self, count: u16, ref_voltage_mv: u16) -> u16 {
        use crate::_pac::adc::adcctl2::Adcres;
        let ctl2 = self.adc_reg.adcctl2().read();
        // ADCRES (SLAU445I Table 21-5, p. 565)
        let bits = match ctl2.adcres().variant() {
            Adcres::Adcres0 => 8,
            Adcres::Adcres1 => 10,
            Adcres::Adcres2 => 12,
            Adcres::Adcres3 => 12, // Reserved, unreachable
        };
        let count = if ctl2.adcdf().bit_is_set() {
            // Left-aligned two's complement, offset by half the range (SLAU445I 21.3.5, p. 566, and ADCDF in
            // SLAU445I Table 21-5, p. 565)
            (((count as i16) >> (16 - bits)) + (1 << (bits - 1))) as u16
        } else {
            count
        };
        let product = count as u32 * ref_voltage_mv as u32;
        let mv = match bits {
            8 => div_by_full_scale::<8>(product),
            10 => div_by_full_scale::<10>(product),
            _ => div_by_full_scale::<12>(product),
        };
        mv as u16
    }

    /// Begins a single ADC conversion if one isn't already underway, enabling the ADC in the process.
    ///
    /// If the result is ready it is returned as a voltage in millivolts based on `ref_voltage_mv`, otherwise returns `WouldBlock`.
    ///
    /// If you instead want a raw count you should use the `.read_count()` method.
    pub fn read_voltage_mv<PIN: Channel<Adc, ID = u8>>(
        &mut self,
        pin: &mut PIN,
        ref_voltage_mv: u16,
    ) -> nb::Result<u16, Infallible> {
        self.read_count(pin).map(|count| self.count_to_mv(count, ref_voltage_mv))
    }

    /// Select the reference voltages the ADC measures against (ADCSREF: SLAU445I Table 21-8, p. 567). An
    /// input at or below the negative reference converts to 0, one at or above the positive reference to the
    /// full-scale count (SLAU445I 21.2.1, p. 541).
    ///
    /// The internal reference must stay enabled, and the VeREF pins in their analog mode, while the ADC
    /// uses them, so the returned ADC borrows them (SLAU445I 21.2.3.1, p. 542: "The on-chip reference from
    /// the PMM module must be enabled by software"). Waits for a conversion in progress to finish.
    pub fn with_reference<'a>(
        mut self,
        positive: PositiveReference<'a>,
        negative: NegativeReference<'a>,
    ) -> Adc<SelectedReference<'a>> {
        // ADCSREFx: bits 1-0 of the field select VR+, bit 2 selects VEREF- as VR- (SLAU445I Table 21-8,
        // p. 567)
        let vr_plus: u8 = match positive {
            PositiveReference::Avcc => 0b00,
            PositiveReference::Internal(_) => 0b01,
            PositiveReference::ExternalBuffered(_) => 0b10,
            PositiveReference::External(_) => 0b11,
        };
        let vr_minus: u8 = match negative {
            NegativeReference::Avss => 0,
            NegativeReference::External(_) => 1,
        };
        // "It is not recommended to change this setting while a conversion is ongoing" (SLAU445I Table 21-8,
        // p. 567)
        while self.adc_is_busy() {}
        self.disable();
        self.pending = None;
        self.adc_reg.adcmctl0().modify(|_, w| w.adcsref().set(vr_minus << 2 | vr_plus));
        Adc { adc_reg: self.adc_reg, pending: None, reference: PhantomData }
    }

    /// Start conversions of `pin`'s channel, or in the sequence modes of the channels from it down to
    /// channel 0, as `config` describes (ADCINCHx: SLAU445I Table 21-8, p. 567). Read the results with
    /// [`result()`](Adc::result()).
    ///
    /// A sequence converts every channel down to 0, so their pins should be in their analog mode too.
    /// Conversions already running are stopped first, and their results discarded (SLAU445I 21.2.7.1, p. 547:
    /// resetting ADCON within a conversion returns the ADC to the 'ADC off' state).
    pub fn start<PIN>(&mut self, _pin: &mut PIN, config: ConversionConfig)
    where PIN: Channel<Adc, ID = u8> {
        self.disable();
        self.pending = None;

        // ADCSHSx (SLAU445I Table 21-4, p. 563), with the sources of the trigger tables cited at
        // `TriggerSource`
        let shs: u8 = match config.trigger {
            TriggerSource::Software => 0b00,
            TriggerSource::Rtc => 0b01,
            TriggerSource::Timer => 0b10,
            #[cfg(feature = "ecomp")]
            TriggerSource::Comparator => 0b11,
        };
        // ADCSHP and ADCISSH (SLAU445I Table 21-4, p. 563)
        let (shp, issh) = match (config.trigger, config.sample_mode) {
            // The software trigger is a pulse (SLAU445I Table 21-3, p. 562: "ADCSC is reset automatically")
            (TriggerSource::Software, _) => (true, false),
            (_, SampleMode::RisingEdge) => (true, false),
            (_, SampleMode::FallingEdge) => (true, true),
            (_, SampleMode::WhileHigh) => (false, false),
            (_, SampleMode::WhileLow) => (false, true),
        };
        // With ADCCONSEQx (SLAU445I Table 21-4, p. 564)
        self.adc_reg.adcctl1().modify(|_, w| unsafe { w.adcshs().set(shs).adcshp().bit(shp).adcissh().bit(issh).adcconseq().bits(config.mode as u8) });
        self.adc_reg.adcctl0().modify(|_, w| w.adcmsc().bit(config.back_to_back));
        self.adc_reg.adcmctl0().modify(|_, w| w.adcinch().set(PIN::channel()));
        // Discard results and flags of earlier conversions (ADCIFG: SLAU445I Table 21-14, p. 571)
        self.adc_reg.adcifg().write(|w| unsafe { w.bits(0) });

        self.enable();
        // Hardware triggers start conversions once ADCENC is set (SLAU445I Figure 21-10, p. 547); ADCSC may
        // be set with ADCENC (SLAU445I Table 21-3, p. 562)
        let software = matches!(config.trigger, TriggerSource::Software);
        self.adc_reg.adcctl0().modify(|_, w| w.adcenc().set_bit().adcsc().bit(software));
    }

    /// The next result of the conversions started with [`start()`](Adc::start()), or `WouldBlock` if
    /// none is ready (ADCIFG0). Reading a result clears the flag (SLAU445I Table 21-14, p. 571).
    ///
    /// Results that aren't read before the next one arrives are lost, see
    /// [`AdcInterruptFlags::Overflow`] (SLAU445I 21.2.7.10, p. 558).
    pub fn result(&mut self) -> nb::Result<u16, Infallible> {
        if self.adc_reg.adcifg().read().adcifg0().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }
        Ok(self.adc_get_result())
    }

    /// Stop the conversions started with [`start()`](Adc::start()), after the current conversion in the
    /// single modes and after the current sequence in the sequence modes (SLAU445I 21.2.7.6, p. 555).
    pub fn stop(&mut self) {
        let single = self.adc_reg.adcctl1().read().adcconseq().bits() == 0;
        if single {
            // Clearing ADCENC would cut a single conversion short (SLAU445I 21.2.7.6, p. 555: "poll the busy
            // bit until reset before clearing ADCENC")
            while self.adc_is_busy() {}
        }
        self.adc_reg.adcctl0().modify(|_, w| w.adcenc().clear_bit());
    }

    /// Set the window comparator thresholds (ADCLO, ADCHI), in the configured [`DataFormat`]. Each result
    /// then sets one of the flags [`AdcInterruptFlags::BelowWindow`], [`InsideWindow`](AdcInterruptFlags::InsideWindow)
    /// and [`AboveWindow`](AdcInterruptFlags::AboveWindow). (SLAU445I 21.2.7.7, p. 555: "The values in the
    /// ADCHI and ADCLO registers must be in the correct data format"; registers: SLAU445I 21.3.7, p. 568, to
    /// SLAU445I 21.3.10, p. 569)
    ///
    /// The ADC only sets these flags, so clear them with [`clear_interrupt_flags()`](Adc::clear_interrupt_flags())
    /// once handled (SLAU445I 21.2.7.7, p. 555: "The interrupt flags must be reset by software").
    pub fn set_window(&mut self, low: u16, high: u16) {
        self.adc_reg.adclo().write(|w| unsafe { w.bits(low) });
        self.adc_reg.adchi().write(|w| unsafe { w.bits(high) });
    }

    /// Request the `ADC` interrupt for `flags`, besides those already enabled (ADCIE: SLAU445I Table 21-13,
    /// p. 570).
    pub fn enable_interrupts(&mut self, flags: AdcInterruptFlags) {
        self.adc_reg.adcie().modify(|r, w| unsafe { w.bits(r.bits() | flags.bits()) });
    }

    /// Stop requesting the `ADC` interrupt for `flags` (ADCIE: SLAU445I Table 21-13, p. 570).
    pub fn disable_interrupts(&mut self, flags: AdcInterruptFlags) {
        self.adc_reg.adcie().modify(|r, w| unsafe { w.bits(r.bits() & !flags.bits()) });
    }

    /// The interrupt flags that are set, whether or not their interrupt is enabled (ADCIFG: SLAU445I
    /// Table 21-14, p. 571).
    pub fn interrupt_flags(&self) -> AdcInterruptFlags {
        AdcInterruptFlags::from_bits_truncate(self.adc_reg.adcifg().read().bits())
    }

    /// Clear `flags` (ADCIFG: SLAU445I Table 21-14, p. 571).
    pub fn clear_interrupt_flags(&mut self, flags: AdcInterruptFlags) {
        self.adc_reg.adcifg().modify(|r, w| unsafe { w.bits(r.bits() & !flags.bits()) });
    }

    /// The highest-priority pending interrupt among the enabled ones (ADCIV). Reading it clears its flag,
    /// except [`AdcVector::ResultReady`], which reading the result clears. (SLAU445I 21.2.7.10.1, p. 558)
    pub fn interrupt_source(&mut self) -> AdcVector {
        // ADCIV values (SLAU445I Table 21-15, p. 572)
        match self.adc_reg.adciv().read().bits() {
            0x02 => AdcVector::Overflow,
            0x04 => AdcVector::TimeOverflow,
            0x06 => AdcVector::AboveWindow,
            0x08 => AdcVector::BelowWindow,
            0x0A => AdcVector::InsideWindow,
            0x0C => AdcVector::ResultReady,
            _ => AdcVector::None,
        }
    }
}

// Clears ADCON and ADCENC (SLAU445I Table 21-3, p. 561 to p. 562)
fn disable_adc_reg(adc: &mut _pac::Adc) {
    unsafe {
        adc.adcctl0().clear_bits(|w| w
            .adcon().clear_bit()
            .adcenc().clear_bit());
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::adc::{Channel, OneShot};

    impl<REF, PIN> OneShot<Adc, u16, PIN> for Adc<REF>
    where PIN: Channel<Adc, ID = u8>
    {
        type Error = Infallible; // Only returns WouldBlock

        /// Begins a single ADC conversion if one isn't already underway, enabling the ADC in the process.
        ///
        /// If the result is ready it is returned as an ADC count, otherwise returns `WouldBlock`
        #[inline(always)]
        fn read(&mut self, pin: &mut PIN) -> nb::Result<u16, Self::Error> { self.read_count(pin) }
    }
}
