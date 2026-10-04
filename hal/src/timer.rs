//! Countdown timers
//!
//! Configures the board's TimerB peripherals into periodic countdown timers. Each peripheral
//! consists of a main timer and multiple "sub-timers". Sub-timers have their own thresholds and
//! interrupts but share their countdowns with their main timer: they are its capture/compare blocks
//! (SLAU445I 13.2.4, p. 374; 14.2.4, p. 398).
//!
//! This module also contains traits used by other HAL modules that depend on TimerB, such as
//! `Capture` and `Pwm`.
//!
//! # Timer_B outputs and the comparators
//!
//! After reset the output of an eCOMP comparator switches all outputs of a Timer_B to high impedance while
//! it is high: eCOMP0 for TB0 and TB1, eCOMP1 for TB2 and TB3 (SLASEC4D Table 6-20, p. 76; SLASEO7C
//! Table 9-17, p. 61). SYSCFG2.TBxTRGSEL resets to 0, "Internal source selected" (SLAU445I Table 1-26,
//! p. 77; SLAU445I Table 1-31, p. 82). Measured on an MSP430FR2476, a TB0 PWM output stops whenever
//! eCOMP0's output is high, even with the comparator used for something else.
//! [`TimerConfig::high_impedance_trigger`] selects the TBxTRG pin instead, or nothing.

use crate::clock::{Aclk, Smclk};
use crate::hw_traits::timer_base::{CCRn, Outmod, RunningMode, Tbssel, TimerBase};
use crate::pin_mapping::*;
use core::convert::Infallible;
use core::marker::PhantomData;

pub use crate::hw_traits::timer_base::{
    TimerDiv, TimerExDiv, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};

// Trait effectively sealed by CCRn
/// Trait indicating that the peripheral can be used as a sub-timer, PWM, or capture
pub trait CapCmp<C>: CCRn<C> {}
impl<T: CCRn<C>, C> CapCmp<C> for T {}

// Trait effectively sealed by TimerB
/// Trait indicating that the peripheral can be used as a timer
pub trait TimerPeriph<M: PinMap = DefaultMapping>: TimerBase + CapCmp<CCR0> {
    /// Pin type used for external TBxCLK of this timer, or [`NoTbxclkPin`] if it has none (TASSEL/TBSSEL =
    /// 00b: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
    type Tbxclk;

    /// Additional configuration
    #[inline(always)]
    fn configure_pin_mapping() {}
}

/// The external TBxCLK pin of timers that don't have one, such as TA2 and TA3 on the
/// MSP430FR2433 (SLASE59F Table 6-13, p. 51; SLASE59F Table 6-14, p. 52)
///
/// It has no values, so [`TimerConfig::tbclk`] can't be called for these timers.
pub enum NoTbxclkPin {}

// Traits effectively sealed by CCRn
/// Trait indicating that the peripheral has 2 capture compare registers: TA2 and TA3 on the MSP430FR2433
/// (SLASE59F 6.10.8, p. 51: "two capture/compare registers each")
pub trait CapCmpTimer2<M: PinMap = DefaultMapping>: TimerPeriph<M> + CapCmp<CCR1> {}
/// Trait indicating that the peripheral has 3 capture compare registers ("three capture/compare registers
/// each"): TB0 to TB2 on the MSP430FR2x5x (SLASEC4D 6.10.9, p. 73), TA0 and TA1 on the MSP430FR2433
/// (SLASE59F 6.10.8, p. 50), TA0 to TA3 on the MSP430FR247x (SLASEO7C 9.10.8, p. 55), and TA0 and TA1 on
/// the MSP430FR25x2 (SLASEE4C 6.10.8, p. 54)
pub trait CapCmpTimer3<M: PinMap = DefaultMapping>:
    TimerPeriph<M> + CapCmp<CCR1> + CapCmp<CCR2>
{}
/// Trait indicating that the peripheral has 7 capture compare registers: TB3 on the MSP430FR2x5x
/// (SLASEC4D 6.10.9, p. 73: "seven capture/compare registers") and TB0 on the MSP430FR247x (SLASEO7C
/// Table 9-15, p. 59, CCR0 to CCR6)
pub trait CapCmpTimer7<M: PinMap = DefaultMapping>:
    TimerPeriph<M>
    + CapCmp<CCR1>
    + CapCmp<CCR2>
    + CapCmp<CCR3>
    + CapCmp<CCR4>
    + CapCmp<CCR5>
    + CapCmp<CCR6>
{}

// Trait effectively sealed by TimerBase
/// Trait indicating a Timer_B. Its counter length can be changed, see [`TimerConfig::counter_length`], and its
/// compare registers are buffered, which PWM uses so that a duty cycle change never spoils a period
/// (SLAU445I 14.1.1, p. 391; 14.2.4.2.1, p. 400). Erratum TB25 makes two of the load modes load at once in up
/// mode on the MSP430FR2x5x and MSP430FR247x, so edge-aligned PWM uses one it doesn't list (SLAZ695J TB25,
/// p. 11; SLAZ726B TB25, p. 8; see [`crate::pwm`]).
pub trait TimerB: TimerBase {}

/// The number of bits a Timer_B counts with (CNTL), which sets its highest count in continuous mode
/// (SLAU445I 14.2.1.1, p. 393; SLAU445I Table 14-6, p. 409)
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum CounterLength {
    /// 16 bits, up to 0xFFFF, as after reset (CNTL = 00b, reset value 0h: SLAU445I Table 14-6, p. 409)
    #[default]
    _16Bit = 0,
    /// 12 bits, up to 0x0FFF (CNTL = 01b: SLAU445I Table 14-6, p. 409)
    _12Bit = 1,
    /// 10 bits, up to 0x03FF (CNTL = 10b: SLAU445I Table 14-6, p. 409)
    _10Bit = 2,
    /// 8 bits, up to 0x00FF (CNTL = 11b: SLAU445I Table 14-6, p. 409)
    _8Bit = 3,
}

/// Which compare latches of a Timer_B load together (TBCLGRP: SLAU445I 14.2.4.2.2, p. 400; SLAU445I
/// Table 14-3, p. 400; SLAU445I Table 14-6, p. 409), see [`TimerConfig::compare_latch_groups`].
///
/// The user's guide lists the groups of a Timer_B with seven capture/compare registers. The ones with three,
/// TB0 to TB2 on the MSP430FR2x5x (SLASEC4D Table 6-16, p. 73; SLASEC4D Table 6-17, p. 74; SLASEC4D
/// Table 6-18, p. 74), have no TBxCL3 to TBxCL6, and it doesn't say what `Triples` and `All` group there.
/// `Pairs` groups their TBxCL1 and TBxCL2.
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum CompareLatchGroups {
    /// Each compare latch loads on its own, as after reset (TBCLGRP = 00b, reset value 0h: SLAU445I
    /// Table 14-6, p. 409)
    #[default]
    Independent = 0,
    /// TBxCL1 + TBxCL2, TBxCL3 + TBxCL4 and TBxCL5 + TBxCL6, controlled by TBxCCR1, TBxCCR3 and TBxCCR5;
    /// TBxCL0 on its own (TBCLGRP = 01b)
    Pairs = 1,
    /// TBxCL1 + TBxCL2 + TBxCL3 and TBxCL4 + TBxCL5 + TBxCL6, controlled by TBxCCR1 and TBxCCR4; TBxCL0 on its
    /// own (TBCLGRP = 10b)
    Triples = 2,
    /// TBxCL0 to TBxCL6 all together, controlled by TBxCCR1 (TBCLGRP = 11b)
    All = 3,
}

/// What switches all outputs of a Timer_B to high impedance (TBxOUTH, SYSCFG2.TBxTRGSEL: SLAU445I 14.2.5,
/// p. 401; SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31, p. 82; data sheets: SLASEC4D Table 6-20,
/// p. 76; SLASEO7C Table 9-17, p. 61), for example to stop a motor driver on a fault
pub enum HighImpedanceTrigger<'a, T> {
    /// The output of an eCOMP comparator: eCOMP0 for TB0 and TB1, eCOMP1 for TB2 and TB3 (SLASEC4D
    /// Table 6-20, p. 76; SLASEO7C Table 9-17, p. 61). This is the setting after reset (TBxTRGSEL = 0:
    /// SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31, p. 82), so the outputs stop whenever that
    /// comparator's output is high, even if it's used for something else.
    Comparator,
    /// The timer's TBxTRG pin, in its trigger function: the outputs stop while it's high (SLAU445I 14.2.5,
    /// p. 401: "When the TBOUTH pin function is selected for the pin ... and when the pin is pulled high").
    /// TB3 on the MSP430FR2x5x has no such pin (SLASEC4D Table 6-20, p. 76: "TB3TRGSEL = 1", "N/A").
    Pin(&'a dyn HighImpedancePin<T>),
    /// Nothing switches the outputs to high impedance.
    None,
}

/// Marker trait for the TBxTRG pin of a Timer_B in its trigger function, see [`HighImpedanceTrigger::Pin`]
///
/// The pins: on the MSP430FR2x5x P1.2 is TB0TRG (SLASEC4D Table 6-63, p. 96), P2.3 is TB1TRG (SLASEC4D
/// Table 6-64, p. 98) and P5.3 is TB2TRG (SLASEC4D Table 6-67, p. 104); on the MSP430FR247x P3.5 is TB0TRG
/// (SLASEO7C Table 9-25, p. 67). Each is an input in its trigger function.
pub trait HighImpedancePin<T> {}

/// Trait indicating a Timer_B whose outputs can be switched to high impedance, see
/// [`TimerConfig::high_impedance_trigger`]: TB0 to TB3 on the MSP430FR2x5x, TB0 on the MSP430FR247x
/// (SLASEC4D Table 6-20, p. 76; SLASEO7C Table 9-17, p. 61)
pub trait HighImpedanceTimer: TimerB {
    #[doc(hidden)]
    /// Set (`true`, the TBxTRG pin) or clear (`false`, the comparator) this timer's TBxTRGSEL bit in
    /// SYSCFG2 (SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31, p. 82)
    fn set_trgsel(external: bool);
}

/// Implement [`HighImpedanceTimer`] for the Timer_B `$timer`, whose trigger select bit is the SYSCFG2 field
/// `$trgsel` (SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31, p. 82)
#[cfg(feature = "timer_b")]
macro_rules! high_impedance_timer_impl {
    ($timer:ty, $trgsel:ident) => {
        impl $crate::timer::HighImpedanceTimer for $timer {
            #[inline(always)]
            fn set_trgsel(external: bool) {
                let sys = unsafe { &*$crate::_pac::Sys::ptr() };
                if external {
                    unsafe { sys.syscfg2().set_bits(|w| w.$trgsel().set_bit()) };
                } else {
                    unsafe { sys.syscfg2().clear_bits(|w| w.$trgsel().clear_bit()) };
                }
            }
        }
    };
}
#[cfg(feature = "timer_b")]
pub(crate) use high_impedance_timer_impl;

// Traits effectively sealed by TimerBase
/// Trait indicating that the timer can be clocked from VLOCLK, see [`TimerConfig::vloclk`]
///
/// A timer's fourth clock input, INCLK (TASSEL/TBSSEL = 11b: SLAU445I Table 13-4, p. 384; SLAU445I
/// Table 14-6, p. 409), is wired differently on each device and each timer. These timers have the VLO on
/// it (data sheets, timer signal connections and clock distribution):
///
/// | Device                     | Timers   | Reference                                              |
/// |----------------------------|----------|--------------------------------------------------------|
/// | MSP430FR2475, MSP430FR2476 | TA0, TA2 | SLASEO7C Table 9-12, p. 55; SLASEO7C Table 9-14, p. 58 |
/// | MSP430FR2512, MSP430FR2522 | TA0      | SLASEE4C Figure 6-2, p. 54; SLASEE4C Table 6-8, p. 49  |
/// | Other devices              | None     | SLASEC4D Table 6-9, p. 68; SLASE59F Table 6-7, p. 46   |
pub trait VloclkTimer: TimerBase {}

/// Trait indicating that the timer can be clocked by another timer, see [`TimerConfig::cascade`]
///
/// A timer's fourth clock input, INCLK, is wired differently on each device and each timer. These
/// timers have the CCR2 output of their `Source` timer on it (data sheets, timer signal
/// connections):
///
/// | Device                     | Timer ← `Source`     | Reference                                              |
/// |----------------------------|----------------------|--------------------------------------------------------|
/// | MSP430FR2475, MSP430FR2476 | TA1 ← TA0, TA3 ← TA2 | SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14, p. 58 |
/// | MSP430FR2512, MSP430FR2522 | TA1 ← TA0            | SLASEE4C Figure 6-2, p. 54                             |
/// | MSP430FR2x5x               | TB1 ← TB0            | SLASEC4D Table 6-17, p. 74                             |
/// | MSP430FR2433               | None                 | SLASE59F Tables 6-11 to 6-14, p. 50 to p. 52           |
pub trait CascadedTimer: TimerBase {
    /// Timer whose CCR2 output clocks this timer
    type Source: CapCmp<CCR2>;
}

/// The CCR2 output of timer `T`, set up to clock a [`CascadedTimer`] with
/// [`TimerConfig::cascade`]
///
/// While `T` runs, the output is high for the first count of each period, so the cascaded timer,
/// which counts on the rising edges of its clock (SLAU445I 13.2.1, p. 370), counts as `T` wraps around
/// to 0. This needs a period of at least 2 counts.
pub struct CascadeOutput<T>(PhantomData<T>);

impl<T: CapCmp<CCR2>> CascadeOutput<T> {
    #[inline]
    pub(crate) fn new() -> Self {
        let timer = unsafe { T::steal() };
        // Reset/set mode sets the output as the timer wraps around to 0, and resets it when the
        // timer reaches CCR2 (SLAU445I Table 13-2, p. 376; 13.2.5.1.1, p. 377). With CCR2 at 0 both
        // happen at once and the output stays low.
        CCRn::<CCR2>::set_ccrn(&timer, 1);
        CCRn::<CCR2>::config_outmod(&timer, Outmod::ResetSet);
        CascadeOutput(PhantomData)
    }
}

/// Configuration object for the TimerB peripheral
///
/// Used to configure `Timer`, `Capture`, and `Pwm`, which all use the TimerB peripheral.
pub struct TimerConfig<T, M = DefaultMapping>
where
    T: TimerPeriph<M>,
    M: PinMap,
{
    _timer: PhantomData<T>,
    sel: Tbssel,
    div: TimerDiv,
    ex_div: TimerExDiv,
    cntl: u8,
    tbclgrp: u8,
    /// The timer's TBxTRGSEL setter, and whether to select the pin (SLAU445I Table 1-26, p. 77; SLAU445I
    /// Table 1-31, p. 82)
    trgsel: Option<(fn(bool), bool)>,
    _pin_map: PhantomData<M>,
}

impl<T, M> TimerConfig<T, M>
where
    T: TimerPeriph<M>,
    M: PinMap,
{
    #[inline]
    fn with_clock(sel: Tbssel) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
            cntl: CounterLength::_16Bit as u8,
            tbclgrp: CompareLatchGroups::Independent as u8,
            trgsel: None,
            _pin_map: PhantomData,
        }
    }

    /// Configure timer clock source to ACLK (TASSEL/TBSSEL = 01b: SLAU445I Table 13-4, p. 384; SLAU445I
    /// Table 14-6, p. 409)
    #[inline]
    pub fn aclk(_aclk: &Aclk) -> Self { Self::with_clock(Tbssel::Aclk) }

    /// Configure timer clock source to SMCLK (TASSEL/TBSSEL = 10b: SLAU445I Table 13-4, p. 384; SLAU445I
    /// Table 14-6, p. 409)
    #[inline]
    pub fn smclk(_smclk: &Smclk) -> Self { Self::with_clock(Tbssel::Smclk) }

    /// Configure timer clock source to TBCLK, the timer's clock pin (TASSEL/TBSSEL = 00b: SLAU445I
    /// Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
    #[inline]
    pub fn tbclk(_pin: T::Tbxclk) -> Self { Self::with_clock(Tbssel::Tbxclk) }

    /// Configure the normal clock divider and expansion clock divider settings (ID and TAIDEX/TBIDEX:
    /// SLAU445I 13.2.1.1, p. 370; 14.2.1.2, p. 393)
    #[inline]
    pub fn clk_div(self, div: TimerDiv, ex_div: TimerExDiv) -> Self {
        TimerConfig { div, ex_div, ..self }
    }

    #[inline]
    pub(crate) fn write_regs(self, timer: &T) {
        T::configure_pin_mapping();
        // TBCLR, then TBIDEX, then TBxCTL with MC = 0, as for starting a timer (SLAU445I 13.2.2,
        // p. 370; 14.2.2, p. 393). The functions that start the timer set TBCLR again, which a TBIDEX
        // change needs (SLAU445I 13.3.6, p. 389; 14.3.6, p. 414: "After programming TBIDEX bits and
        // configuring the timer, set TBCLR bit").
        timer.reset();
        timer.set_tbidex(self.ex_div);
        timer.config_clock(self.sel, self.div);
        timer.set_cntl(self.cntl);
        timer.set_tbclgrp(self.tbclgrp);
        // TBxTRGSEL: 0 = internal source (eCOMP), 1 = external source (TBxTRG pin) (SLAU445I Table 1-26,
        // p. 77; SLAU445I Table 1-31, p. 82)
        if let Some((set_trgsel, external)) = self.trgsel {
            set_trgsel(external);
        }
    }
}

impl<T, M> TimerConfig<T, M>
where
    T: TimerPeriph<M> + TimerB,
    M: PinMap,
{
    /// Set how many bits this Timer_B counts with (CNTL). In continuous mode it then counts up to 0xFF,
    /// 0x3FF, 0xFFF or 0xFFFF before it starts over (SLAU445I 14.2.1.1, p. 393; 14.2.3.2, p. 395).
    #[inline]
    pub fn counter_length(self, length: CounterLength) -> Self { TimerConfig { cntl: length as u8, ..self } }

    /// Group the compare latches of this Timer_B (TBCLGRP), so that new values in several capture/compare
    /// registers take effect together, for example the duty cycles of PWM outputs that must change in the
    /// same period. A group loads when "all TBxCCRn registers of the group" have been written, "even when new
    /// TBxCCRn data = old TBxCCRn data", and its load event occurs: that of the controlling register in
    /// [`CompareLatchGroups`] (SLAU445I 14.2.4.2.2, p. 400).
    ///
    /// The controlling register's load event (CLLD) "must not be set to zero", or "all compare latches update
    /// immediately when their corresponding TBxCCRn is written" (SLAU445I 14.2.4.2.2, p. 400). PWM sets it
    /// when a channel is initialized (see [`crate::pwm`]), so with PWM the controlling channel must be in use.
    ///
    /// Measured on an MSP430FR2476, a group only waits for all its registers when the controlling register
    /// loads when the timer counts to 0 or to the top (CLLD = 01b or 10b, SLAU445I Table 14-2, p. 400): with
    /// CLLD = 11b, "when TBxR counts to the old TBxCLn value", each register still loaded on its own, in every
    /// counting mode. So this works with center-aligned PWM, which uses 10b, and not with edge-aligned PWM,
    /// which uses 11b because of erratum TB25 (see [`crate::pwm`]).
    ///
    /// Changed with the timer stopped, as SLAU445I 14.2.7, p. 407 lists TBCLGRP.
    #[inline]
    pub fn compare_latch_groups(self, groups: CompareLatchGroups) -> Self {
        TimerConfig { tbclgrp: groups as u8, ..self }
    }
}

impl<T, M> TimerConfig<T, M>
where
    T: TimerPeriph<M> + HighImpedanceTimer,
    M: PinMap,
{
    /// Select what switches all outputs of this Timer_B to high impedance, see [`HighImpedanceTrigger`].
    #[inline]
    pub fn high_impedance_trigger(self, trigger: HighImpedanceTrigger<T>) -> Self {
        // TBxTRGSEL selects the comparator (0) or the pin (1) (SLAU445I Table 1-26, p. 77; SLAU445I
        // Table 1-31, p. 82; SLASEC4D Table 6-20, p. 76; SLASEO7C Table 9-17, p. 61). Selecting the pin
        // without putting it in its trigger function disables the trigger: only the selected TBOUTH pin
        // function triggers (SLAU445I 14.2.5, p. 401).
        let external = !matches!(trigger, HighImpedanceTrigger::Comparator);
        TimerConfig { trgsel: Some((T::set_trgsel, external)), ..self }
    }
}

impl<T, M> TimerConfig<T, M>
where
    T: TimerPeriph<M> + VloclkTimer,
    M: PinMap,
{
    /// Configure timer clock source to VLOCLK, which runs at about 10 kHz but is only accurate to
    /// ±50 % (VLOCLK "10 kHz ±50%": SLASEO7C Table 9-8, p. 50; SLASEE4C Table 6-8, p. 49). Only some
    /// timers have this option, see [`VloclkTimer`]. It selects INCLK (TASSEL = 11b: SLAU445I Table 13-4,
    /// p. 384).
    #[inline]
    pub fn vloclk() -> Self { Self::with_clock(Tbssel::Inclk) }
}

impl<T, M> TimerConfig<T, M>
where
    T: TimerPeriph<M> + CascadedTimer,
    M: PinMap,
{
    /// Configure the timer to be clocked by its source timer (cascading): it counts once per
    /// period of the source timer, while that timer runs. Only some timers have this option, see
    /// [`CascadedTimer`]. It selects INCLK (TASSEL/TBSSEL = 11b: SLAU445I Table 13-4, p. 384; SLAU445I
    /// Table 14-6, p. 409).
    ///
    /// `source` is the source timer's CCR2 output, from [`SubTimer::into_cascade_output`] or
    /// [`PwmUninit::into_cascade_output`](crate::pwm::PwmUninit::into_cascade_output). For
    /// example, a source timer with a period of 1 s lets this timer count seconds.
    #[inline]
    pub fn cascade(_source: &CascadeOutput<T::Source>) -> Self { Self::with_clock(Tbssel::Inclk) }
}

/// Main timer and sub-timer for timer peripherals with 2 capture-compare registers
///
/// The timers with 2 capture/compare registers are TA2 and TA3 on the MSP430FR2433 (SLASE59F Table 6-13,
/// p. 51; SLASE59F Table 6-14, p. 52).
pub struct TimerParts2<T, M = DefaultMapping>
where
    T: CapCmpTimer2<M>,
    M: PinMap,
{
    /// Main timer
    pub timer: Timer<T, M>,
    /// Timer interrupt vector (TAxIV: SLAU445I Table 13-8, p. 388)
    pub tbxiv: TBxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register, TAxCCR1: SLAU445I Table 13-7, p. 388)
    pub subtimer1: SubTimer<T, CCR1>,
}

impl<T, M> TimerParts2<T, M>
where
    T: CapCmpTimer2<M>,
    M: PinMap,
{
    /// Create new set of timers out of a TBx peripheral
    #[inline(always)]
    pub fn new(_timer: T, config: TimerConfig<T, M>) -> Self {
        config.write_regs(unsafe { &T::steal() });
        Self {
            timer: Timer::new(),
            tbxiv: TBxIV(PhantomData),
            subtimer1: SubTimer::new(),
        }
    }
}

/// Main timer and sub-timers for timer peripherals with 3 capture-compare registers
///
/// The timers with 3 capture/compare registers, by device: TB0 to TB2 on the MSP430FR2x5x (SLASEC4D
/// Table 6-16, p. 73; SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-18, p. 74), TA0 and TA1 on the
/// MSP430FR2433 (SLASE59F Table 6-11, p. 50; SLASE59F Table 6-12, p. 51), TA0 to TA3 on the MSP430FR247x
/// (SLASEO7C Table 9-12, p. 55; SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14, p. 58), and TA0 and TA1
/// on the MSP430FR25x2 (SLASEE4C Figure 6-2, p. 54).
pub struct TimerParts3<T, M = DefaultMapping>
where
    T: CapCmpTimer3<M>,
    M: PinMap,
{
    /// Main timer
    pub timer: Timer<T, M>,
    /// Timer interrupt vector (TAxIV/TBxIV: SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414)
    pub tbxiv: TBxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register: SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413)
    pub subtimer1: SubTimer<T, CCR1>,
    /// Sub-timer 2 (derived from CCR2 register: SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413)
    pub subtimer2: SubTimer<T, CCR2>,
}

impl<T, M> TimerParts3<T, M>
where
    T: CapCmpTimer3<M>,
    M: PinMap,
{
    /// Create new set of timers out of a TBx peripheral
    #[inline(always)]
    pub fn new(_timer: T, config: TimerConfig<T, M>) -> Self {
        config.write_regs(unsafe { &T::steal() });
        Self {
            timer: Timer::new(),
            tbxiv: TBxIV(PhantomData),
            subtimer1: SubTimer::new(),
            subtimer2: SubTimer::new(),
        }
    }
}

/// Main timer and sub-timers for timer peripherals with 7 capture-compare registers
///
/// The timers with 7 capture/compare registers are TB3 on the MSP430FR2x5x (SLASEC4D Table 6-19, p. 75)
/// and TB0 on the MSP430FR247x (SLASEO7C Table 9-15, p. 59). Both are Timer_B.
pub struct TimerParts7<T, M = DefaultMapping>
where
    T: CapCmpTimer7<M>,
    M: PinMap,
{
    /// Main timer
    pub timer: Timer<T, M>,
    /// Timer interrupt vector (TBxIV: SLAU445I Table 14-10, p. 414)
    pub tbxiv: TBxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register, TBxCCR1: SLAU445I Table 14-9, p. 413)
    pub subtimer1: SubTimer<T, CCR1>,
    /// Sub-timer 2 (derived from CCR2 register, TBxCCR2: SLAU445I Table 14-9, p. 413)
    pub subtimer2: SubTimer<T, CCR2>,
    /// Sub-timer 3 (derived from CCR3 register, TBxCCR3: SLAU445I Table 14-9, p. 413)
    pub subtimer3: SubTimer<T, CCR3>,
    /// Sub-timer 4 (derived from CCR4 register, TBxCCR4: SLAU445I Table 14-9, p. 413)
    pub subtimer4: SubTimer<T, CCR4>,
    /// Sub-timer 5 (derived from CCR5 register, TBxCCR5: SLAU445I Table 14-9, p. 413)
    pub subtimer5: SubTimer<T, CCR5>,
    /// Sub-timer 6 (derived from CCR6 register, TBxCCR6: SLAU445I Table 14-9, p. 413)
    pub subtimer6: SubTimer<T, CCR6>,
}

impl<T, M> TimerParts7<T, M>
where
    T: CapCmpTimer7<M>,
    M: PinMap,
{
    /// Create new set of timers out of a TBx peripheral
    #[inline(always)]
    pub fn new(_timer: T, config: TimerConfig<T, M>) -> Self {
        config.write_regs(unsafe { &T::steal() });
        Self {
            timer: Timer::new(),
            tbxiv: TBxIV(PhantomData),
            subtimer1: SubTimer::new(),
            subtimer2: SubTimer::new(),
            subtimer3: SubTimer::new(),
            subtimer4: SubTimer::new(),
            subtimer5: SubTimer::new(),
            subtimer6: SubTimer::new(),
        }
    }
}

/// Main periodic countdown timer
pub struct Timer<T: TimerPeriph<M>, M: PinMap = DefaultMapping> {
    /// The mode it counts in once started, for `resume()`
    mode: RunningMode,
    _timer: PhantomData<T>,
    _pin_map: PhantomData<M>,
}

impl<T, M> Timer<T, M>
where
    T: TimerPeriph<M>,
    M: PinMap,
{
    fn new() -> Self { Self { mode: RunningMode::Up, _timer: PhantomData, _pin_map: PhantomData } }
}

/// Sub-timer associated with a main timer
///
/// Each sub-timer has its own interrupt mechanism and threshold, but shares its countdown value
/// with its main timer (a capture/compare block: SLAU445I 13.2.4, p. 374; 14.2.4, p. 398).
pub struct SubTimer<T: CapCmp<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmp<C>, C> SubTimer<T, C> {
    fn new() -> Self { Self(PhantomData, PhantomData) }
}

/// Indicates which sub/main timer caused the interrupt to fire: the TAxIV/TBxIV values 00h to 0Eh, from
/// no interrupt through CCR1 to CCR6 to the timer overflow (SLAU445I Table 13-8, p. 388; SLAU445I
/// Table 14-10, p. 414). CCR0 has its own interrupt vector and isn't in this list (SLAU445I 13.2.6.1,
/// p. 380; 14.2.6.1, p. 405).
pub enum TimerVector {
    /// No pending interrupt
    NoInterrupt,
    /// Interrupt caused by sub-timer 1
    SubTimer1,
    /// Interrupt caused by sub-timer 2
    SubTimer2,
    /// Interrupt caused by sub-timer 3
    SubTimer3,
    /// Interrupt caused by sub-timer 4
    SubTimer4,
    /// Interrupt caused by sub-timer 5
    SubTimer5,
    /// Interrupt caused by sub-timer 6
    SubTimer6,
    /// Interrupt caused by main timer overflow
    MainTimer,
}

#[inline]
pub(crate) fn read_tbxiv<T: TimerBase>(timer: &T) -> TimerVector {
    // TBxIV only takes these values (SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414)
    match timer.tbxiv_rd() {
        0 => TimerVector::NoInterrupt,
        2 => TimerVector::SubTimer1,
        4 => TimerVector::SubTimer2,
        6 => TimerVector::SubTimer3,
        8 => TimerVector::SubTimer4,
        10 => TimerVector::SubTimer5,
        12 => TimerVector::SubTimer6,
        14 => TimerVector::MainTimer,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}

/// Interrupt vector register for determining which timer caused an ISR (TAxIV/TBxIV: SLAU445I Table 13-8,
/// p. 388; SLAU445I Table 14-10, p. 414)
pub struct TBxIV<T>(PhantomData<T>);

impl<T: TimerBase> TBxIV<T> {
    #[inline]
    /// Read the timer interrupt vector. Automatically resets corresponding interrupt flag (SLAU445I
    /// 13.2.6.2, p. 380; 14.2.6.2, p. 405).
    pub fn interrupt_vector(&mut self) -> TimerVector {
        let timer = unsafe { T::steal() };
        read_tbxiv(&timer)
    }
}

impl<T, M> Timer<T, M>
where
    T: TimerPeriph<M>,
    M: PinMap,
{
    /// Enable timer countdown expiration interrupts (TBIE: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6,
    /// p. 410)
    #[inline(always)]
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.tbie_set();
    }

    /// Disable timer countdown expiration interrupts (TBIE: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6,
    /// p. 410)
    #[inline(always)]
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.tbie_clr();
    }

    #[inline]
    /// Clears the timer, sets the count, and starts the timer in upcounting mode (up mode: SLAU445I
    /// 13.2.3.1, p. 371; 14.2.3.1, p. 394).
    pub fn start(&mut self, count: u16) {
        let timer = unsafe { T::steal() };
        // CCR0 is updated while the timer is stopped (SLAU445I 13.2.3.1.1, p. 371)
        timer.stop();
        timer.set_ccrn(count);
        timer.upmode();
        self.mode = RunningMode::Up;
    }

    #[inline]
    /// Clears the timer and starts it counting up to `count` and back down to 0 (up/down mode), so a
    /// period lasts `2 * count` timer cycles. [`wait()`](Timer::wait) returns once per period, when the
    /// count gets back to 0 (SLAU445I 13.2.3.4, p. 373; 14.2.3.4, p. 396), and sub-timers fire twice per
    /// period, on the way up and on the way down (SLAU445I Figure 13-14, p. 379; SLAU445I Figure 14-14,
    /// p. 404).
    pub fn start_up_down(&mut self, count: u16) {
        let timer = unsafe { T::steal() };
        // CCR0 is updated while the timer is stopped (SLAU445I 13.2.3.4.1, p. 373)
        timer.stop();
        timer.set_ccrn(count);
        timer.updown_mode();
        self.mode = RunningMode::UpDown;
    }

    #[inline]
    /// Checks if the timer has reached the target value. Returns `Ok(())` if so, otherwise `WouldBlock`.
    ///
    /// It checks TBIFG, which is set as the count goes from the target value to 0 (SLAU445I 13.2.3.1,
    /// p. 371; 14.2.3.1, p. 394), or in up/down mode as it gets back down to 0.
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        let timer = unsafe { T::steal() };
        if timer.tbifg_rd() {
            timer.tbifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    /// Pause the timer at the current value (MC = 0, "The timer is halted": SLAU445I Table 13-1, p. 371;
    /// SLAU445I Table 14-1, p. 394)
    pub fn pause(&mut self) {
        let timer = unsafe { T::steal() };
        timer.stop();
    }

    #[inline]
    /// Resume counting from the current value, in the direction and mode it was counting in (the
    /// count direction is latched: SLAU445I 13.2.3.4, p. 373; 14.2.3.4, p. 396)
    pub fn resume(&mut self) {
        let timer = unsafe { T::steal() };
        timer.resume(self.mode);
    }

    #[inline]
    /// Get the current timer value.
    ///
    /// A timer clocked asynchronously to MCLK (from ACLK, for example) can return a wrong value
    /// when read while it counts, so this takes the median of three reads, as the user's guide
    /// suggests (SLAU445I 13.2.1, p. 370; 14.2.1, p. 393, notes "Accessing TAxR" and "Accessing TBxR":
    /// "TBxR can be read multiple times while the timer is running, and a majority vote taken in
    /// software").
    pub fn count(&mut self) -> u16 {
        let timer = unsafe { T::steal() };
        let (a, b, c) = (timer.get_tbxr(), timer.get_tbxr(), timer.get_tbxr());
        a.min(b).max(a.max(b).min(c))
    }
}

impl<T: CapCmp<C>, C> SubTimer<T, C> {
    #[inline]
    /// Set the threshold for one of the sub-timers. Once the main timer counts to this threshold
    /// the sub-timer will fire (SLAU445I 13.2.4.2, p. 376; 14.2.4.2, p. 399). Note that the main timer
    /// resets once it counts to its own threshold, not the sub-timer thresholds. It follows that the
    /// sub-timer threshold must not be more than the main threshold for it to fire: the main timer
    /// counts up to and including its threshold (SLAU445I 13.2.3.1, p. 371; 14.2.3.1, p. 394).
    ///
    /// It writes TAxCCRn/TBxCCRn (SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413) and clears
    /// CCIFG (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412). A Timer_A is stopped for the write,
    /// as the user's guide says "the timer should be stopped by writing the MC bits to zero (MC = 0) before
    /// writing new data to TAxCCRn" (SLAU445I 13.2.4.2, p. 376), so the main timer misses the few timer
    /// clocks that takes.
    pub fn set_count(&mut self, count: u16) {
        let timer = unsafe { T::steal() };
        timer.set_ccrn(count);
        timer.ccifg_clr();
    }

    #[inline]
    /// Wait for the sub-timer to fire (its CCIFG: SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        let timer = unsafe { T::steal() };
        if timer.ccifg_rd() {
            timer.ccifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline(always)]
    /// Enable the sub-timer interrupts (CCIE: SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_set();
    }

    #[inline(always)]
    /// Disable the sub-timer interrupts (CCIE: SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_clr();
    }
}

impl<T: CapCmp<CCR2>> SubTimer<T, CCR2> {
    #[inline]
    /// Use CCR2 to clock a cascaded timer instead, see [`TimerConfig::cascade`]. The CCR2 output drives the
    /// cascaded timer's INCLK (SLASEC4D Table 6-17, p. 74; SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14,
    /// p. 58; SLASEE4C Figure 6-2, p. 54).
    pub fn into_cascade_output(self) -> CascadeOutput<T> { CascadeOutput::new() }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::timer::{Cancel, CountDown, Periodic};

    impl<T, M> CountDown for Timer<T, M>
    where
        T: TimerPeriph<M> + CapCmp<CCR0>,
        M: PinMap,
    {
        type Time = u16;

        #[inline]
        fn start<U: Into<Self::Time>>(&mut self, count: U) { self.start(count.into()) }

        #[inline]
        fn wait(&mut self) -> nb::Result<(), void::Void> {
            self.wait().map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl<T, M> Cancel for Timer<T, M>
    where
        T: TimerPeriph<M> + CapCmp<CCR0>,
        M: PinMap,
    {
        type Error = void::Void;

        #[inline(always)]
        fn cancel(&mut self) -> Result<(), Self::Error> {
            self.pause();
            Ok(())
        }
    }

    impl<T, M> Periodic for Timer<T, M>
    where
        T: TimerPeriph<M>,
        M: PinMap,
    {}
}
