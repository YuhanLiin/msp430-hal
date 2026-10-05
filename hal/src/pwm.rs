//! PWM ports
//!
//! Configures the board's TimerB peripherals into PWM ports. Each PWM port consists of multiple PWM
//! pins which all share the same period but have their own duty cycles (CCR0 sets the period: SLAU445I
//! 13.2.3.1, p. 371).
//!
//! Each PWM pin starts off in an "uninitialized" state and must be initialized by passing in the
//! appropriate alternate-function GPIO pin. Only initialized pins can be used for PWM.
//!
//! The outputs go high at the start of each period with [`PwmParts3::new`] (output mode reset/set,
//! SLAU445I Figure 13-12, p. 377), or are centered on the timer's return to 0 with
//! [`PwmParts3::new_center_aligned`] (output mode toggle/reset in up/down mode, SLAU445I Figure 13-14,
//! p. 379). [`Pwm::set_polarity`] makes an output active low.
//!
//! A new duty cycle takes effect as follows:
//! - With center-aligned PWM on a Timer_B, when the timer next counts to the top or to 0 (the compare
//!   latch, CLLD = 10b: SLAU445I 14.2.4.2.1, p. 400; SLAU445I Table 14-2, p. 400). A duty cycle above 0
//!   that replaces 0 and loads at 0, so one written while the timer counts down, starts out of step: the
//!   output is high from the duty cycle up to the top instead of around 0, until the timer reaches the top.
//!   Measured on an MSP430FR2476, only that change does this: not the same change written while the timer
//!   counts up, nor changes to 0, from the maximum or to it. Two outputs driving a half bridge can then be
//!   on together, so keep the pins at their GPIO level with [`Pwm::disable`] until the first duty cycles
//!   have loaded, as the MSP430FR2476 example `pwm_center_aligned` does. Outputs whose duty cycles must
//!   change in the same period, such as those two, can share a compare latch group, see
//!   [`TimerConfig::compare_latch_groups`].
//! - With edge-aligned PWM on a Timer_B, in the period after the timer next reaches the old duty cycle (the
//!   compare latch, CLLD = 11b: SLAU445I Table 14-2, p. 400), so a period is never cut short or left high to
//!   its end. That is one or two periods later. Erratum TB25 makes the modes that load at the start of a
//!   period load at once instead, on the MSP430FR2x5x and MSP430FR247x (SLAZ695J TB25, p. 11; SLAZ726B
//!   TB25, p. 8), and doesn't list this one. To change the duty cycle exactly once per period, set it in
//!   the interrupt at the start of each period, see [`Pwm::enable_period_interrupt`].
//! - On a Timer_A, at once. The timer is stopped for the write, as the user's guide says (SLAU445I
//!   13.2.4.2, p. 376).
//!
//! A duty cycle written below the timer's current count takes effect in the next period, as the count
//! has passed it: in reset/set mode "The output is reset when the timer counts to the TAxCCRn value"
//! (SLAU445I Table 13-2, p. 376), so that period's output stays high to the end.
//!
//! # Timer_B outputs and the comparators
//!
//! After reset the output of an eCOMP comparator switches all outputs of a Timer_B to high impedance while
//! it is high: eCOMP0 for TB0 and TB1, eCOMP1 for TB2 and TB3 (SLASEC4D Table 6-20, p. 76; SLASEO7C
//! Table 9-17, p. 61). SYSCFG2.TBxTRGSEL resets to 0, "Internal source selected" (SLAU445I Table 1-26,
//! p. 77; SLAU445I Table 1-31, p. 82). Measured on an MSP430FR2476, a TB0 PWM output stops whenever
//! eCOMP0's output is high, even with the comparator used for something else.
//! [`TimerConfig::high_impedance_trigger`](crate::timer::TimerConfig::high_impedance_trigger) selects the TBxTRG
//! pin instead, or nothing.

use crate::gpio::AlternatePin;
use crate::hw_traits::timer_base::{CCRn, Clld, Outmod, RunningMode, TimerBase};
use crate::pin_mapping::{DefaultMapping, PinMap};
use crate::timer::{CapCmpTimer3, CapCmpTimer7};
use core::marker::PhantomData;

pub use crate::timer::{
    CapCmp, CascadeOutput, CompareLatchGroups, TimerConfig, TimerDiv, TimerExDiv, TimerPeriph, CCR0, CCR1,
    CCR2, CCR3, CCR4, CCR5, CCR6,
};

// Sealed by CapCmp
/// Associates PWM pins with specific GPIO pins, for pin mapping `M`
///
/// The pins are the timer outputs TAx.n and TBx.n in the device's timer signal connection tables (SLASEC4D
/// Tables 6-16 to 6-19, p. 73 to p. 75; SLASE59F Tables 6-11 and 6-12, p. 50 to p. 51; SLASEO7C Tables 9-12
/// to 9-16, p. 55 to p. 60; SLASEE4C Figure 6-2, p. 54).
pub trait PwmPeriph<C, M: PinMap = DefaultMapping>: CapCmp<C> + CapCmp<CCR0> + TimerBase {
    /// GPIO type, in the alternate function that outputs the PWM signal
    type Gpio: AlternatePin;
}

fn setup_pwm<T: TimerPeriph<M>, M: PinMap>(timer: &T, config: TimerConfig<T, M>, period: u16) {
    // This leaves the timer stopped (MC = 0), for the writes below and in `setup_channel`
    config.write_regs(timer);
    // CCR0 sets the period, written while the timer is stopped (SLAU445I 13.2.3.1.1, p. 371). Its
    // output toggles once per period, for the period output (SLAU445I Table 13-2, p. 376).
    CCRn::<CCR0>::set_ccrn_stopped(timer, period);
    CCRn::<CCR0>::config_outmod_stopped(timer, Outmod::Toggle, Clld::Immediately);
}

/// Whether PWM outputs go high at the start of each period, or are centered on the timer's return to 0
#[derive(Copy, Clone, PartialEq, Eq)]
enum Alignment {
    Edge,
    Center,
}

/// Configure a PWM channel: its output mode, and on Timer_B when its compare latch loads the duty cycle,
/// so a new duty cycle starts with a period instead of cutting one short (SLAU445I 14.2.4.2.1, p. 400).
/// Erratum TB25 breaks this in up mode, see below (SLAZ695J TB25, p. 11; SLAZ726B TB25, p. 8). Both go in one
/// write of TBxCCTLn, while the timer is stopped, by `setup_pwm` (SLAU445I 13.2.7, p. 382; SLAU445I 14.2.7,
/// p. 407).
fn setup_channel<T: CapCmp<C>, C>(timer: &T, alignment: Alignment) {
    match alignment {
        Alignment::Edge => {
            // Set as the timer wraps to 0, reset when it reaches CCRn (SLAU445I Table 13-2, p. 376;
            // SLAU445I Figure 13-12, p. 377).
            // Load when the timer counts to the old duty cycle (CLLD = 11b: "when TBxR counts to the old
            // TBxCLn value", SLAU445I Table 14-2, p. 400): the period running then ends at the old duty
            // cycle, and the next ones have the new one. Loading when the timer counts to 0 (CLLD = 01b or
            // 10b) doesn't work in up mode on the MSP430FR2x5x and MSP430FR247x, the devices with a Timer_B:
            // "TBxCCRn will update immediately instead of the described condition", "contrary to the user
            // guide description of TBxCCTLn.CLLD = 0x01 or 0x10 modes" (SLAZ695J TB25, p. 11; SLAZ726B TB25,
            // p. 8). Loading at once (CLLD = 00b, the erratum's workaround) leaves a period high to its end
            // when a duty cycle below the timer's count replaces one above it. Measured on an MSP430FR2476,
            // with the duty cycle changed at random moments: 669 of 3000 periods stayed high to the end with
            // CLLD = 00b, none with CLLD = 11b.
            CCRn::<C>::config_outmod_stopped(timer, Outmod::ResetSet, Clld::AtOldValue);
        }
        Alignment::Center => {
            // In up/down mode: high from CCRn on the way down to CCRn on the way up, reset at the top
            // (SLAU445I Table 13-2, p. 376; SLAU445I Figure 13-14, p. 379).
            // Load when the timer counts to 0 or to the top (CLLD = 10b, SLAU445I Table 14-2, p. 400). A
            // duty cycle written while the timer counts down loads at 0, in the middle of a high stretch,
            // so that one stretch is not symmetric, and one that replaces 0 starts out of step (see the
            // module documentation).
            CCRn::<C>::config_outmod_stopped(timer, Outmod::ToggleReset, Clld::AtZeroOrTop);
        }
    }
}

/// Write a duty cycle to CCRn. With edge-aligned PWM on a Timer_B (CLLD = 11b), the compare latch loads when
/// the timer counts to the old duty cycle (SLAU445I Table 14-2, p. 400), which it never does after 100 %: in
/// up mode it counts no higher than CCR0 (SLAU445I 14.2.3.1, p. 394), and 100 % is CCR0 + 1. So a duty cycle
/// after 100 % loads at once (CLLD = 00b). That doesn't cut a period short, as the output stays high to the
/// end of the period either way, unless the 100 % was written less than a period earlier and hadn't loaded
/// yet. CLLD may change while the timer runs: SLAU445I 14.2.7, p. 407 doesn't list it ("Control bits that
/// are not listed below can be read or updated while the timer is running").
#[inline]
fn write_duty<T: CapCmp<C> + CapCmp<CCR0>, C>(timer: &T, duty: u16) {
    let leaving_full = CCRn::<C>::clld_rd(timer) == Clld::AtOldValue as u8
        && CCRn::<C>::get_ccrn(timer) > CCRn::<CCR0>::get_ccrn(timer);
    if leaving_full {
        CCRn::<C>::set_clld(timer, Clld::Immediately);
        CCRn::<C>::set_ccrn(timer, duty);
        CCRn::<C>::set_clld(timer, Clld::AtOldValue);
    } else {
        CCRn::<C>::set_ccrn(timer, duty);
    }
}

/// Collection of uninitialized PWM pins derived from timer peripheral with 3 capture-compare registers
///
/// The timers with 3 capture/compare registers, by device: TB0 to TB2 on the MSP430FR2x5x (SLASEC4D
/// Table 6-16, p. 73; SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-18, p. 74), TA0 and TA1 on the
/// MSP430FR2433 (SLASE59F Table 6-11, p. 50; SLASE59F Table 6-12, p. 51), TA0 to TA3 on the MSP430FR247x
/// (SLASEO7C Table 9-12, p. 55; SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14, p. 58), and TA0 and TA1
/// on the MSP430FR25x2 (SLASEE4C Figure 6-2, p. 54).
pub struct PwmParts3<T: CapCmpTimer3<M>, M: PinMap = DefaultMapping> {
    /// Square wave output of capture-compare register 0, see [`PeriodOutputUninit`]
    pub period_output: PeriodOutputUninit<T, M>,
    /// PWM pin 1 (derived from capture-compare register 1: SLAU445I Table 13-7, p. 388; SLAU445I
    /// Table 14-9, p. 413)
    pub pwm1: PwmUninit<T, CCR1, M>,
    /// PWM pin 2 (derived from capture-compare register 2: SLAU445I Table 13-7, p. 388; SLAU445I
    /// Table 14-9, p. 413)
    pub pwm2: PwmUninit<T, CCR2, M>,
    _pin_map: PhantomData<M>,
}

impl<T: CapCmpTimer3<M>, M: PinMap> PwmParts3<T, M> {
    /// Create uninitialized PWM pins with the same period. The timer counts from 0 up to and
    /// including `period`, so each PWM period is `period + 1` timer clock cycles (SLAU445I 13.2.3.1,
    /// p. 371; 14.2.3.1, p. 394), and each output is high at the start of the period.
    ///
    /// On a Timer_B a new duty cycle loads when the timer counts to the old one (CLLD = 11b), as erratum
    /// TB25 makes the loads at the start of a period happen at once in up mode (SLAZ695J TB25, p. 11;
    /// SLAZ726B TB25, p. 8), see the module documentation.
    pub fn new(timer: T, config: TimerConfig<T, M>, period: u16) -> Self {
        setup_pwm(&timer, config, period);
        setup_channel::<T, CCR1>(&timer, Alignment::Edge);
        setup_channel::<T, CCR2>(&timer, Alignment::Edge);
        // Start the timer to run PWM (up mode, MC = 01b: SLAU445I Table 13-1, p. 371; SLAU445I Table 14-1,
        // p. 394)
        timer.upmode();
        Self::parts()
    }

    /// Create uninitialized center-aligned PWM pins with the same period. The timer counts from 0 up
    /// to `period` and back down (up/down mode), so each PWM period is `2 * period` timer clock
    /// cycles (SLAU445I 13.2.3.4, p. 373; 14.2.3.4, p. 396), and each output is high for a stretch
    /// centered on the timer's return to 0. Duty cycles go up to `period`. Two outputs with nearly the
    /// same duty cycle, one of them active low, drive the two sides of a half bridge with a dead time
    /// between them (SLAU445I 13.2.3.5 and Figure 13-9, p. 374; SLAU445I 14.2.3.5 and Figure 14-9, p. 397).
    ///
    /// On a Timer_B a new duty cycle loads when the timer counts to 0 or to the top (CLLD = 10b). Erratum
    /// TB25, which makes such loads happen at once, names up mode only, and this uses up/down mode
    /// (SLAZ695J TB25, p. 11; SLAZ726B TB25, p. 8).
    pub fn new_center_aligned(timer: T, config: TimerConfig<T, M>, period: u16) -> Self {
        setup_pwm(&timer, config, period);
        setup_channel::<T, CCR1>(&timer, Alignment::Center);
        setup_channel::<T, CCR2>(&timer, Alignment::Center);
        // Up/down mode, MC = 11b (SLAU445I Table 13-1, p. 371; SLAU445I Table 14-1, p. 394)
        timer.updown_mode();
        Self::parts()
    }

    fn parts() -> Self {
        Self {
            period_output: PeriodOutputUninit(PhantomData, PhantomData),
            pwm1: PwmUninit::new(),
            pwm2: PwmUninit::new(),
            _pin_map: PhantomData,
        }
    }
}

/// Collection of uninitialized PWM pins derived from timer peripheral with 7 capture-compare registers
///
/// The timers with 7 capture/compare registers are TB3 on the MSP430FR2x5x (SLASEC4D Table 6-19, p. 75)
/// and TB0 on the MSP430FR247x (SLASEO7C Table 9-15, p. 59). Both are Timer_B.
pub struct PwmParts7<T: CapCmpTimer7<M>, M: PinMap = DefaultMapping> {
    /// Square wave output of capture-compare register 0, see [`PeriodOutputUninit`]
    pub period_output: PeriodOutputUninit<T, M>,
    /// PWM pin 1 (derived from capture-compare register 1, TBxCCR1: SLAU445I Table 14-9, p. 413)
    pub pwm1: PwmUninit<T, CCR1, M>,
    /// PWM pin 2 (derived from capture-compare register 2, TBxCCR2: SLAU445I Table 14-9, p. 413)
    pub pwm2: PwmUninit<T, CCR2, M>,
    /// PWM pin 3 (derived from capture-compare register 3, TBxCCR3: SLAU445I Table 14-9, p. 413)
    pub pwm3: PwmUninit<T, CCR3, M>,
    /// PWM pin 4 (derived from capture-compare register 4, TBxCCR4: SLAU445I Table 14-9, p. 413)
    pub pwm4: PwmUninit<T, CCR4, M>,
    /// PWM pin 5 (derived from capture-compare register 5, TBxCCR5: SLAU445I Table 14-9, p. 413)
    pub pwm5: PwmUninit<T, CCR5, M>,
    /// PWM pin 6 (derived from capture-compare register 6, TBxCCR6: SLAU445I Table 14-9, p. 413)
    pub pwm6: PwmUninit<T, CCR6, M>,
    _pin_map: PhantomData<M>,
}

impl<T: CapCmpTimer7<M>, M: PinMap> PwmParts7<T, M> {
    /// Create uninitialized PWM pins with the same period. The timer counts from 0 up to and
    /// including `period`, so each PWM period is `period + 1` timer clock cycles (SLAU445I 13.2.3.1,
    /// p. 371; 14.2.3.1, p. 394), and each output is high at the start of the period.
    ///
    /// A new duty cycle loads when the timer counts to the old one (CLLD = 11b), as erratum TB25 makes
    /// the loads at the start of a period happen at once in up mode (SLAZ695J TB25, p. 11; SLAZ726B
    /// TB25, p. 8), see the module documentation.
    pub fn new(timer: T, config: TimerConfig<T, M>, period: u16) -> Self {
        setup_pwm(&timer, config, period);
        Self::setup_channels(&timer, Alignment::Edge);
        // Up mode, MC = 01b (SLAU445I Table 14-1, p. 394)
        timer.upmode();
        Self::parts()
    }

    /// Create uninitialized center-aligned PWM pins with the same period, see
    /// [`PwmParts3::new_center_aligned`].
    pub fn new_center_aligned(timer: T, config: TimerConfig<T, M>, period: u16) -> Self {
        setup_pwm(&timer, config, period);
        Self::setup_channels(&timer, Alignment::Center);
        // Up/down mode, MC = 11b (SLAU445I Table 14-1, p. 394)
        timer.updown_mode();
        Self::parts()
    }

    fn setup_channels(timer: &T, alignment: Alignment) {
        setup_channel::<T, CCR1>(timer, alignment);
        setup_channel::<T, CCR2>(timer, alignment);
        setup_channel::<T, CCR3>(timer, alignment);
        setup_channel::<T, CCR4>(timer, alignment);
        setup_channel::<T, CCR5>(timer, alignment);
        setup_channel::<T, CCR6>(timer, alignment);
    }

    fn parts() -> Self {
        Self {
            period_output: PeriodOutputUninit(PhantomData, PhantomData),
            pwm1: PwmUninit::new(),
            pwm2: PwmUninit::new(),
            pwm3: PwmUninit::new(),
            pwm4: PwmUninit::new(),
            pwm5: PwmUninit::new(),
            pwm6: PwmUninit::new(),
            _pin_map: PhantomData,
        }
    }
}

/// The uninitialized output of capture-compare register 0, which toggles once per PWM period: a square wave
/// at half the PWM frequency (SLAU445I Table 13-2, p. 376), useful as a clock for other circuits. That holds
/// for center-aligned PWM too: in up/down mode the timer reaches CCR0 once per period (SLAU445I 13.2.3.4,
/// p. 373).
///
/// Only some timers have a pin for it: on the MSP430FR247x TA2, TA3 and TB0 (data sheet: TA2.0, TA3.0,
/// TB0.0; SLASEO7C Table 9-14, p. 58; SLASEO7C Table 9-15, p. 59; SLASEO7C Table 9-16, p. 60). The other
/// data sheets say the CCR0 outputs are not connected to pins (SLASEC4D 6.10.9, p. 73; SLASE59F 6.10.8,
/// p. 50 to p. 51; SLASEE4C 6.10.8, p. 54).
pub struct PeriodOutputUninit<T, M = DefaultMapping>(PhantomData<T>, PhantomData<M>);

impl<T: PwmPeriph<CCR0, M>, M: PinMap> PeriodOutputUninit<T, M> {
    /// Initializes the output by passing in the appropriately configured GPIO pin.
    #[inline]
    pub fn init(self, pin: <T as PwmPeriph<CCR0, M>>::Gpio) -> PeriodOutput<T, M> { PeriodOutput { pin } }
}

/// The output of capture-compare register 0 on its pin, see [`PeriodOutputUninit`]
pub struct PeriodOutput<T: PwmPeriph<CCR0, M>, M: PinMap = DefaultMapping> {
    pin: <T as PwmPeriph<CCR0, M>>::Gpio,
}

impl<T: PwmPeriph<CCR0, M>, M: PinMap> PeriodOutput<T, M> {
    /// Disconnect the pin from the timer (PxSEL: SLAU445I 8.2.5, p. 314). It then drives its GPIO output
    /// level (PxOUT, SLAU445I 8.2.2, p. 313).
    #[inline]
    pub fn disable(&mut self) { self.pin.set_function_gpio(); }

    /// Connect the pin to the timer again (PxSEL: SLAU445I 8.2.5, p. 314).
    #[inline]
    pub fn enable(&mut self) { self.pin.set_function_from_type(); }
}

/// The active level of a PWM output: the level it has for the duty cycle. Active high uses the output modes
/// reset/set or toggle/reset, active low set/reset or toggle/set (SLAU445I Table 13-2, p. 376; SLAU445I
/// Table 14-4, p. 401).
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Polarity {
    /// High for the duty cycle, as set up by `PwmParts`
    #[default]
    ActiveHigh,
    /// Low for the duty cycle
    ActiveLow,
}

/// Uninitialized PWM pin
pub struct PwmUninit<T, C, M = DefaultMapping>(PhantomData<T>, PhantomData<C>, PhantomData<M>);

impl<T: PwmPeriph<C, M>, C, M: PinMap> PwmUninit<T, C, M> {
    /// Initializes the PWM pin by passing in the appropriately configured GPIO pin.
    #[inline]
    pub fn init(self, pin: <T as PwmPeriph<C, M>>::Gpio) -> Pwm<T, C, M> {
        Pwm { _timer: PhantomData, _ccrn: PhantomData, _pin_map: PhantomData, pin }
    }
}

impl<T, C, M> PwmUninit<T, C, M> {
    #[inline]
    fn new() -> Self { Self(PhantomData, PhantomData, PhantomData) }
}

impl<T: CapCmp<CCR2>, M> PwmUninit<T, CCR2, M> {
    /// Use this PWM output to clock a cascaded timer instead of a pin, see
    /// [`TimerConfig::cascade`]. The cascaded timer then counts PWM periods. The CCR2 output drives the
    /// cascaded timer's INCLK (SLASEC4D Table 6-17, p. 74; SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14,
    /// p. 58; SLASEE4C Figure 6-2, p. 54).
    #[inline]
    pub fn into_cascade_output(self) -> CascadeOutput<T> { CascadeOutput::new() }
}

#[cfg(feature = "adc")]
impl<T: CapCmp<CCR1> + CapCmp<CCR0> + crate::adc::AdcTriggerTimer, M> PwmUninit<T, CCR1, M> {
    /// Use this PWM output to start ADC conversions with
    /// [`TriggerSource::Timer`](crate::adc::TriggerSource::Timer) instead of driving a pin (ADC trigger
    /// TB1.1B or TA1.1B: SLASEC4D Table 6-22, p. 77; SLASE59F Table 6-16, p. 53; SLASEO7C Table 9-20, p. 62;
    /// SLASEE4C Table 6-14, p. 56). The output is high for the first `high_cycles` timer cycles of each
    /// period, so it rises at the start of each period (reset/set, SLAU445I Table 13-2, p. 376); it needs to
    /// be at least 1. With [`SampleMode::WhileHigh`](crate::adc::SampleMode::WhileHigh) it sets the sample
    /// time (extended sample mode, SLAU445I 21.2.5.1, p. 543).
    #[inline]
    pub fn into_adc_trigger(self, high_cycles: u16) -> AdcTriggerOutput<T> {
        let mut output = AdcTriggerOutput(PhantomData);
        output.set_high_cycles(high_cycles);
        output
    }
}

impl<T: CapCmp<CCR2> + crate::ir::IrInputTimer, M> PwmUninit<T, CCR2, M> {
    /// Use this PWM output as an input of the infrared modulator, see [`crate::ir`], instead of driving a pin
    /// (CCR2 of TB0/TB1 or TA0/TA1: SLASEC4D Table 6-16, p. 73; SLASEC4D Table 6-17, p. 74; SLASE59F
    /// Table 6-11, p. 50; SLASE59F Table 6-12, p. 51; SLASEO7C Table 9-12, p. 55; SLASEO7C Table 9-13,
    /// p. 56; SLASEE4C Figure 6-2, p. 54). The output is high for the first `high_cycles` timer cycles of
    /// each period (reset/set, SLAU445I Table 13-2, p. 376).
    #[inline]
    pub fn into_ir_input(self, high_cycles: u16) -> crate::ir::IrInput<T> {
        let timer = unsafe { T::steal() };
        CCRn::<CCR2>::set_ccrn(&timer, high_cycles);
        crate::ir::IrInput(PhantomData)
    }
}

#[cfg(feature = "sac_l3")]
impl<C> PwmUninit<crate::pac::Tb2, C>
where
    crate::pac::Tb2: CapCmp<C>,
{
    /// Use this TB2 output to load the SAC DACs instead of driving a pin: `pwm1` with
    /// [`LoadTrigger::TB2_1`](crate::sac::LoadTrigger::TB2_1), `pwm2` with
    /// [`LoadTrigger::TB2_2`](crate::sac::LoadTrigger::TB2_2) (TB2.1 and TB2.2 go "To SAC DAC update
    /// trigger": SLASEC4D Table 6-18, p. 74; DACLSEL = 10b and 11b: SLASEC4D Table 6-32, p. 80). A DAC
    /// loads "on the rising edge" of its trigger (SLAU445I 20.2.3.4, p. 529), so this selects reset/set
    /// mode with CCRn = 1: the output is set when the timer counts to TBxCL0, once per period, and reset
    /// when it counts to 1 (SLAU445I Table 14-4, p. 401). That needs a `period` of at least 2.
    ///
    /// Writing the output mode also clears CLLD, so CCRn loads at once (CLLD = 00b: SLAU445I Table 14-2,
    /// p. 400). On `pwm1`, the controlling register of the compare latch groups, that ungroups them:
    /// "When the CLLD bits of the controlling TBxCCRn are set to zero, all compare latches update
    /// immediately when their corresponding TBxCCRn is written" (SLAU445I 14.2.4.2.2, p. 400; see
    /// [`TimerConfig::compare_latch_groups`]).
    #[inline]
    pub fn into_dac_trigger(self) -> crate::sac::DacTrigger<C> {
        let timer = unsafe { crate::pac::Tb2::steal() };
        // OUTMOD first: its write clears CLLD (SLAU445I Table 14-8, p. 411), so CCRn loads at once
        CCRn::<C>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<C>::set_ccrn(&timer, 1);
        crate::sac::DacTrigger(PhantomData)
    }
}

/// A PWM output that starts ADC conversions, see [`PwmUninit::into_adc_trigger()`]
pub struct AdcTriggerOutput<T>(PhantomData<T>);

impl<T: CapCmp<CCR1> + CapCmp<CCR0>> AdcTriggerOutput<T> {
    /// Change how many timer cycles the output stays high at the start of each period. It writes CCR1
    /// (TAxCCR1/TBxCCR1: SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413) as [`Pwm`] duty cycle
    /// changes do: with a Timer_A stopped for the write (SLAU445I 13.2.4.2, p. 376), and on a Timer_B at
    /// once after a value above the period, which the compare latch would otherwise never load (see
    /// `write_duty`).
    #[inline]
    pub fn set_high_cycles(&mut self, high_cycles: u16) {
        let timer = unsafe { T::steal() };
        write_duty::<T, CCR1>(&timer, high_cycles);
    }
}

/// The duty cycle of 100 %, in timer clock cycles. In up mode it's the period: the timer counts from 0 up
/// to and including CCR0 (SLAU445I 13.2.3.1, p. 371). With CCR0 at 65535 the period is 65536 cycles, which
/// a `u16` can't hold, so 100 % isn't reachable. For center-aligned PWM (up/down mode) it's CCR0: the period
/// is 2 * CCR0 cycles (SLAU445I 13.2.3.4, p. 373), and the output is high while the count is below CCRn, on
/// the way down and on the way up (SLAU445I Figure 13-14, p. 379).
#[inline]
fn max_duty<T: CapCmp<CCR0> + TimerBase>() -> u16 {
    let timer = unsafe { T::steal() };
    let ccr0 = CCRn::<CCR0>::get_ccrn(&timer);
    if timer.mode_rd() == RunningMode::UpDown as u8 {
        ccr0
    } else {
        ccr0.saturating_add(1)
    }
}

/// An initialized Pwm pin
pub struct Pwm<T: PwmPeriph<C, M>, C, M: PinMap = DefaultMapping> {
    _timer: PhantomData<T>,
    _ccrn: PhantomData<C>,
    _pin_map: PhantomData<M>,
    pin: <T as PwmPeriph<C, M>>::Gpio,
}

impl<T: PwmPeriph<C, M>, C, M: PinMap> Pwm<T, C, M> {
    /// The duty cycle in timer clock cycles: the output is high for this many cycles of each period, or
    /// twice as many with center-aligned PWM, where the timer passes each count twice per period (SLAU445I
    /// 13.2.3.4, p. 373; SLAU445I Figure 13-14, p. 379). It reads CCRn (TAxCCRn/TBxCCRn: SLAU445I
    /// Table 13-7, p. 388; SLAU445I Table 14-9, p. 413).
    #[inline]
    pub fn duty(&self) -> u16 {
        let timer = unsafe { T::steal() };
        CCRn::<C>::get_ccrn(&timer)
    }

    /// Disconnect the pin from the timer (PxSEL: SLAU445I 8.2.5, p. 314). It then drives its GPIO output
    /// level (PxOUT, SLAU445I 8.2.2, p. 313), for example low if it was set up with
    /// [`to_output_low()`](crate::gpio::Pin::to_output_low). The timer keeps running.
    #[inline]
    pub fn disable(&mut self) { self.pin.set_function_gpio(); }

    /// Connect the pin to the timer again (PxSEL: SLAU445I 8.2.5, p. 314).
    #[inline]
    pub fn enable(&mut self) { self.pin.set_function_from_type(); }

    /// Select the level the output has for the duty cycle. Only the top OUTMOD bit changes, so the mode
    /// doesn't pass through other output modes (SLAU445I 13.2.5.1.3, p. 379; 14.2.5.1.3, p. 404, note
    /// "Switching between output modes": "one of the OUTMOD bits should remain set during the transition"),
    /// and the timer is stopped for the change (SLAU445I 13.2.7, p. 382; SLAU445I 14.2.7, p. 407). The
    /// output takes its new level at its next set or reset event (SLAU445I Table 13-2, p. 376).
    #[inline]
    pub fn set_polarity(&mut self, polarity: Polarity) {
        let timer = unsafe { T::steal() };
        // Edge-aligned PWM uses reset/set (7) or set/reset (3), center-aligned toggle/reset (2) or
        // toggle/set (6) (SLAU445I Table 13-2, p. 376). The top output mode bit picks between each pair.
        let outmod = CCRn::<C>::outmod_rd(&timer);
        let center = outmod == Outmod::ToggleReset as u8 || outmod == Outmod::ToggleSet as u8;
        let high_bit = (polarity == Polarity::ActiveHigh) != center;
        CCRn::<C>::set_outmod_high_bit(&timer, high_bit);
    }

    /// Request the timer's overflow interrupt at the start of each period, shared by all PWM outputs of the
    /// timer (TAIE/TBIE, SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 410). Its flag is set when
    /// the timer counts from CCR0 to zero, with edge-aligned PWM ("The TBIFG interrupt flag is set when the
    /// timer counts from TBxCL0 to zero", SLAU445I 14.2.3.1, p. 394; SLAU445I 13.2.3.1, p. 371), and when
    /// it "completes counting down from 0001h to 0000h" with center-aligned PWM (SLAU445I 14.2.3.4,
    /// p. 396; SLAU445I 13.2.3.4, p. 373). It is served by the timer's second interrupt vector, with
    /// CCR1 and up (TAxIV/TBxIV, SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414); clear it
    /// there with [`Pwm::take_period_flag`].
    ///
    /// A duty cycle set in that interrupt changes once per period, at a known period: on a Timer_A, which
    /// writes it at once (SLAU445I 13.2.4.2, p. 376), the period that has just started has it, as the count
    /// is still low; with edge-aligned PWM on a Timer_B the next one has it, as the period that has just
    /// started ends at the old duty cycle (CLLD = 11b, see the module documentation). TI's workaround for
    /// erratum TB25 also sets the duty cycle in this interrupt (SLAZ695J TB25, p. 11; SLAZ726B TB25, p. 8);
    /// the HAL avoids the erratum with CLLD = 11b instead, so a duty cycle set at any time doesn't spoil a
    /// period either.
    #[inline]
    pub fn enable_period_interrupt(&mut self) {
        let timer = unsafe { T::steal() };
        timer.tbie_set();
    }

    /// Stop requesting the overflow interrupt (TAIE/TBIE, SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6,
    /// p. 410)
    #[inline]
    pub fn disable_period_interrupt(&mut self) {
        let timer = unsafe { T::steal() };
        timer.tbie_clr();
    }

    /// Whether a new period started since the flag was last cleared, and clear it (TAIFG/TBIFG, SLAU445I
    /// Table 13-4, p. 384; SLAU445I Table 14-6, p. 410). Call it in the interrupt handler, as the flag
    /// keeps requesting the interrupt while it is set.
    #[inline]
    pub fn take_period_flag(&mut self) -> bool {
        let timer = unsafe { T::steal() };
        let set = timer.tbifg_rd();
        if set {
            timer.tbifg_clr();
        }
        set
    }
}

mod ehal1 {
    use super::*;
    use core::convert::Infallible;
    use embedded_hal::pwm::{ErrorType, SetDutyCycle};

    impl<T: PwmPeriph<C, M>, C, M: PinMap> ErrorType for Pwm<T, C, M> {
        type Error = Infallible;
    }

    impl<T: PwmPeriph<C, M>, C, M: PinMap> SetDutyCycle for Pwm<T, C, M> {
        /// The PWM period in timer clock cycles, `period + 1` (SLAU445I 13.2.3.1, p. 371). For
        /// center-aligned PWM it's `period`, half the period of `2 * period` cycles (SLAU445I 13.2.3.4,
        /// p. 373). A duty cycle of 0 keeps the output low and the maximum keeps it high.
        #[inline]
        fn max_duty_cycle(&self) -> u16 { max_duty::<T>() }

        /// Set the duty cycle to `duty / max_duty`.
        ///
        /// The caller is responsible for ensuring that the duty cycle value is less than or equal to the maximum duty cycle value,
        /// as reported by `max_duty_cycle`.
        ///
        /// As the error type is `Infallible` this can be safely unwrapped.
        ///
        /// It writes CCRn (TAxCCRn/TBxCCRn: SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413). A
        /// Timer_A is stopped for the write, as the user's guide says it "should be stopped" before new data
        /// is written to TAxCCRn in compare mode (SLAU445I 13.2.4.2, p. 376), so it misses the few timer
        /// clocks that takes, and that period is longer by as much. A Timer_B keeps running: it buffers the
        /// value in its compare latch until the moment the module documentation gives (SLAU445I 14.2.4.2.1,
        /// p. 400).
        #[inline]
        fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
            let timer = unsafe { T::steal() };
            write_duty::<T, C>(&timer, duty);
            Ok(())
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::PwmPin;

    impl<T: PwmPeriph<C, M>, C, M: PinMap> PwmPin for Pwm<T, C, M> {
        /// Number of cycles
        type Duty = u16;

        /// Writes CCRn as `set_duty_cycle` does, with a Timer_A stopped for the write (TAxCCRn/TBxCCRn:
        /// SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413; SLAU445I 13.2.4.2, p. 376).
        #[inline]
        fn set_duty(&mut self, duty: Self::Duty) {
            let timer = unsafe { T::steal() };
            write_duty::<T, C>(&timer, duty);
        }

        #[inline]
        fn get_duty(&self) -> Self::Duty { self.duty() }

        /// The PWM period in timer clock cycles, `period + 1` (SLAU445I 13.2.3.1, p. 371). For
        /// center-aligned PWM it's `period`, half the period of `2 * period` cycles (SLAU445I 13.2.3.4,
        /// p. 373). A duty of 0 keeps the output low and the maximum keeps it high.
        #[inline]
        fn get_max_duty(&self) -> Self::Duty { max_duty::<T>() }

        #[inline]
        fn disable(&mut self) { Pwm::disable(self) }

        #[inline]
        fn enable(&mut self) { Pwm::enable(self) }
    }
}
