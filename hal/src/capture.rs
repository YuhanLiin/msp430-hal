//! Capture ports
//!
//! Configures the board's TimerB peripherals into capture pins. Each capture pin has a 16-bit
//! capture register where its timer value is written whenever its capture event is triggered
//! (SLAU445I 13.2.4.1, p. 374; 14.2.4.1, p. 398).
//!
//! Due to hardware constraints, the configurations for all capture pins derived from a timer must
//! be decided before any of them can be used: CM, CCIS, SCS and CAP are not to be changed while the
//! timer runs (SLAU445I 13.2.7, p. 382; 14.2.7, p. 407). This differs from `Pwm`, where pins are
//! initialized on an individual basis.
//!
//! A capture can also be started from software, see [`Capture::trigger_capture`], which records the
//! timer count at that moment (SLAU445I 13.2.4.1.1, p. 376; 14.2.4.1.1, p. 399).

use crate::hw_traits::timer_base::{CCRn, Ccis, Cm};
use crate::pin_mapping::*;
use crate::timer::{CapCmpTimer2, CapCmpTimer3, CapCmpTimer7, TimerVector};
use core::marker::PhantomData;

pub use crate::timer::{
    CapCmp, TimerConfig, TimerDiv, TimerExDiv, TimerPeriph, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5,
    CCR6,
};

/// Capture edge trigger (CM: SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
pub enum CapTrigger {
    /// Capture on rising edge
    RisingEdge,
    /// Capture on falling edge
    FallingEdge,
    /// Capture on both edges
    BothEdges,
}

impl From<CapTrigger> for Cm {
    #[inline]
    fn from(val: CapTrigger) -> Self {
        match val {
            CapTrigger::RisingEdge => Cm::RisingEdge,
            CapTrigger::FallingEdge => Cm::FallingEdge,
            CapTrigger::BothEdges => Cm::BothEdges,
        }
    }
}

struct PinConfig {
    select: Ccis,
    trigger: CapTrigger,
}

impl Default for PinConfig {
    fn default() -> Self {
        Self { select: Ccis::Gnd, trigger: CapTrigger::RisingEdge }
    }
}

/// The capture input A of capture pins whose input A isn't connected: those of TA2 and TA3 on the
/// MSP430FR2433 (SLASE59F Table 6-13, p. 51; SLASE59F Table 6-14, p. 52), and capture pin 0 of TA0 and TA1
/// on the MSP430FR2433 (SLASE59F Table 6-11, p. 50; SLASE59F Table 6-12, p. 51), of TA1 on the
/// MSP430FR247x (SLASEO7C Table 9-13, p. 56) and the MSP430FR25x2 (SLASEE4C Figure 6-2, p. 54), and of TB2
/// and TB3 on the MSP430FR2x5x (SLASEC4D Table 6-18, p. 74; SLASEC4D Table 6-19, p. 75).
///
/// It has no values, so input A can't be selected for these capture pins.
pub enum NoCapturePin {}

/// Extension trait for creating capture pins from timer peripherals
///
/// Input A of capture pin n is the timer's CCInA input (CCIS = 00b: SLAU445I Table 13-6, p. 386; SLAU445I
/// Table 14-8, p. 411). Which pin drives it is device specific: see the timer signal connection tables
/// (SLASEC4D Tables 6-16 to 6-19, p. 73 to p. 75; SLASE59F Tables 6-11 to 6-14, p. 50 to p. 52; SLASEO7C
/// Tables 9-12 to 9-15, p. 55 to p. 59; SLASEE4C Figure 6-2, p. 54).
pub trait CapturePeriph<M: PinMap = DefaultMapping>: TimerPeriph<M> {
    /// GPIO pin that supplies input A for capture pin 0
    type Gpio0;
    /// GPIO pin that supplies input A for capture pin 1
    type Gpio1;
    /// GPIO pin that supplies input A for capture pin 2
    type Gpio2;
    /// GPIO pin that supplies input A for capture pin 3
    type Gpio3;
    /// GPIO pin that supplies input A for capture pin 4
    type Gpio4;
    /// GPIO pin that supplies input A for capture pin 5
    type Gpio5;
    /// GPIO pin that supplies input A for capture pin 6
    type Gpio6;
}

macro_rules! config_fn {
    (methods $config_sel_b:ident, $config_trigger:ident, $config_sw:ident, $pin:ident) => {
        #[allow(non_snake_case)]
        #[inline(always)]
        /// Configure the capture input select of the capture pin as capture input B (CCIS = 01b: SLAU445I
        /// Table 13-6, p. 386; SLAU445I Table 14-8, p. 411). Which signal that is depends on the device and
        /// the timer, see the timer signal connection tables listed at [`CapturePeriph`].
        ///
        /// On the MSP430FR247x it can't be eCOMP0's output, erratum COMP12: "eCOMP0 output can not be
        /// selected internally to the Timer0_B7 CCI1B input (TB0CCTL1.CCIS = 01b)". The workaround:
        /// "Connect eCOMP0 output and Timer B capture input externally through GPIOs" (SLAZ726B COMP12,
        /// p. 5), such as eCOMP0's output pin P3.4 to TB0.CCI1A on P4.7, input A of the same capture pin
        /// (SLASEO7C Table 9-22, p. 63; SLASEO7C Table 9-15, p. 59).
        pub fn $config_sel_b(mut self) -> Self {
            self.$pin.select = Ccis::InputB;
            self
        }

        #[inline(always)]
        /// Configure the capture trigger event of the capture pin (CM: SLAU445I 13.2.4.1, p. 374)
        pub fn $config_trigger(mut self, trigger: CapTrigger) -> Self {
            self.$pin.trigger = trigger;
            self
        }

        #[inline(always)]
        /// Configure the capture pin for captures started from software, see
        /// [`Capture::trigger_capture`]: its input starts at GND and it captures on both edges (SLAU445I
        /// 13.2.4.1.1, p. 376; 14.2.4.1.1, p. 399).
        pub fn $config_sw(mut self) -> Self {
            self.$pin.select = Ccis::Gnd;
            self.$pin.trigger = CapTrigger::BothEdges;
            self
        }
    };

    ($config_sel_a:ident, $config_sel_b:ident, $config_trigger:ident, $config_sw:ident, $pin:ident, $gpio:ident) => {
        #[allow(non_snake_case)]
        #[inline(always)]
        /// Configure the capture input select of the capture pin as capture input A, which
        /// requires a correctly configured GPIO pin (CCIS = 00b: SLAU445I Table 13-6, p. 386; SLAU445I
        /// Table 14-8, p. 411).
        pub fn $config_sel_a(mut self, _gpio: T::$gpio) -> Self {
            self.$pin.select = Ccis::InputA;
            self
        }
        config_fn!(methods $config_sel_b, $config_trigger, $config_sw, $pin);
    };

    ($config_sel_a:ident, $config_sel_b:ident, $config_trigger:ident, $config_sw:ident, $pin:ident) => {
        #[allow(non_snake_case)]
        #[inline(always)]
        /// Configure the capture input select of the capture pin as capture input A (CCIS = 00b: SLAU445I
        /// Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
        pub fn $config_sel_a(mut self) -> Self {
            self.$pin.select = Ccis::InputA;
            self
        }
        config_fn!(methods $config_sel_b, $config_trigger, $config_sw, $pin);
    };
}

/// Builder object for configuring capture ports derived from timer peripherals with 2
/// capture-compare registers, see [`CaptureConfig3`]
///
/// The timers with 2 capture/compare registers are TA2 and TA3 on the MSP430FR2433 (SLASE59F Table 6-13,
/// p. 51; SLASE59F Table 6-14, p. 52).
pub struct CaptureConfig2<T, M = DefaultMapping>
where
    T: CapturePeriph<M> + CapCmpTimer2<M>,
    M: PinMap,
{
    timer: T,
    config: TimerConfig<T, M>,
    cap0: PinConfig,
    cap1: PinConfig,
}

impl<T, M> CaptureParts2<T, M>
where
    T: CapturePeriph<M> + CapCmpTimer2<M>,
    M: PinMap,
{
    /// Create capture configuration
    pub fn config(timer: T, config: TimerConfig<T, M>) -> CaptureConfig2<T, M> {
        CaptureConfig2 { timer, config, cap0: PinConfig::default(), cap1: PinConfig::default() }
    }
}

impl<T, M> CaptureConfig2<T, M>
where
    T: CapturePeriph<M> + CapCmpTimer2<M>,
    M: PinMap,
{
    config_fn!(config_cap0_input_A, config_cap0_input_B, config_cap0_trigger, config_cap0_software, cap0, Gpio0);
    config_fn!(config_cap1_input_A, config_cap1_input_B, config_cap1_trigger, config_cap1_software, cap1, Gpio1);

    /// Writes all previously configured timer and capture settings into peripheral registers (TAxCTL and
    /// TAxCCTLn: SLAU445I Table 13-4, p. 384; SLAU445I Table 13-6, p. 386), then starts the timer in
    /// continuous mode (SLAU445I 13.2.3.2, p. 372).
    pub fn commit(self) -> CaptureParts2<T, M> {
        let timer = self.timer;
        self.config.write_regs(&timer);
        // Capture settings are written while the timer is stopped (SLAU445I 13.2.7, p. 382; 14.2.7, p. 407)
        CCRn::<CCR0>::config_cap_mode(&timer, self.cap0.trigger.into(), self.cap0.select);
        CCRn::<CCR1>::config_cap_mode(&timer, self.cap1.trigger.into(), self.cap1.select);
        timer.continuous();

        CaptureParts2 { cap0: Capture::new(), cap1: Capture::new(), tbxiv: TBxIV(PhantomData, PhantomData) }
    }
}

/// Builder object for configuring capture ports derived from timer peripherals with 3
/// capture-compare registers
///
/// Each pin has a input source, which determines the signal that controls the capture, and a
/// capture trigger event, which determines the input transitions that actually trigger the
/// capture (CCIS and CM: SLAU445I 13.2.4.1, p. 374; 14.2.4.1, p. 398). By default, all pins use GND
/// as their input source and trigger a capture on a rising edge.
///
/// The timers with 3 capture/compare registers, by device: TB0 to TB2 on the MSP430FR2x5x (SLASEC4D
/// Table 6-16, p. 73; SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-18, p. 74), TA0 and TA1 on the
/// MSP430FR2433 (SLASE59F Table 6-11, p. 50; SLASE59F Table 6-12, p. 51), TA0 to TA3 on the MSP430FR247x
/// (SLASEO7C Table 9-12, p. 55; SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14, p. 58), and TA0 and TA1
/// on the MSP430FR25x2 (SLASEE4C Figure 6-2, p. 54).
pub struct CaptureConfig3<T, M = DefaultMapping>
where
    T: CapturePeriph<M> + CapCmpTimer3<M>,
    M: PinMap,
{
    timer: T,
    config: TimerConfig<T, M>,
    cap0: PinConfig,
    cap1: PinConfig,
    cap2: PinConfig,
}

impl<T, M> CaptureParts3<T, M>
where
    T: CapturePeriph<M> + CapCmpTimer3<M>,
    M: PinMap,
{
    /// Create capture configuration
    pub fn config(timer: T, config: TimerConfig<T, M>) -> CaptureConfig3<T, M> {
        CaptureConfig3 {
            timer,
            config,
            cap0: PinConfig::default(),
            cap1: PinConfig::default(),
            cap2: PinConfig::default(),
        }
    }
}

impl<T, M> CaptureConfig3<T, M>
where
    T: CapturePeriph<M> + CapCmpTimer3<M>,
    M: PinMap,
{
    config_fn!(config_cap0_input_A, config_cap0_input_B, config_cap0_trigger, config_cap0_software, cap0, Gpio0);
    config_fn!(config_cap1_input_A, config_cap1_input_B, config_cap1_trigger, config_cap1_software, cap1, Gpio1);
    config_fn!(config_cap2_input_A, config_cap2_input_B, config_cap2_trigger, config_cap2_software, cap2, Gpio2);

    /// Writes all previously configured timer and capture settings into peripheral registers (TAxCTL or
    /// TBxCTL, and TAxCCTLn or TBxCCTLn: SLAU445I Table 13-4, p. 384; SLAU445I Table 13-6, p. 386; SLAU445I
    /// Table 14-6, p. 409; SLAU445I Table 14-8, p. 411), then starts the timer in continuous mode (SLAU445I
    /// 13.2.3.2, p. 372; 14.2.3.2, p. 395).
    pub fn commit(self) -> CaptureParts3<T, M> {
        let timer = self.timer;
        self.config.write_regs(&timer);
        // Capture settings are written while the timer is stopped (SLAU445I 13.2.7, p. 382; 14.2.7, p. 407)
        CCRn::<CCR0>::config_cap_mode(&timer, self.cap0.trigger.into(), self.cap0.select);
        CCRn::<CCR1>::config_cap_mode(&timer, self.cap1.trigger.into(), self.cap1.select);
        CCRn::<CCR2>::config_cap_mode(&timer, self.cap2.trigger.into(), self.cap2.select);
        timer.continuous();

        CaptureParts3 {
            cap0: Capture::new(),
            cap1: Capture::new(),
            cap2: Capture::new(),
            tbxiv: TBxIV(PhantomData, PhantomData),
        }
    }
}

/// Builder object for configuring capture ports derived from timer peripherals with 7
/// capture-compare registers
///
/// Each pin has a input source, which determines the signal that controls the capture, and a
/// capture trigger event, which determines the input transitions that actually trigger the
/// capture (CCIS and CM: SLAU445I 13.2.4.1, p. 374; 14.2.4.1, p. 398). By default, all pins use GND
/// as their input source and trigger a capture on a rising edge.
///
/// The timers with 7 capture/compare registers are TB3 on the MSP430FR2x5x (SLASEC4D Table 6-19, p. 75)
/// and TB0 on the MSP430FR247x (SLASEO7C Table 9-15, p. 59). Both are Timer_B.
pub struct CaptureConfig7<T, M = DefaultMapping>
where
    T: CapturePeriph<M> + CapCmpTimer7<M>,
    M: PinMap,
{
    timer: T,
    config: TimerConfig<T, M>,
    cap0: PinConfig,
    cap1: PinConfig,
    cap2: PinConfig,
    cap3: PinConfig,
    cap4: PinConfig,
    cap5: PinConfig,
    cap6: PinConfig,
}

impl<T, M> CaptureParts7<T, M>
where
    T: CapturePeriph<M> + CapCmpTimer7<M>,
    M: PinMap,
{
    /// Create capture configuration
    pub fn config(timer: T, config: TimerConfig<T, M>) -> CaptureConfig7<T, M> {
        CaptureConfig7 {
            timer,
            config,
            cap0: PinConfig::default(),
            cap1: PinConfig::default(),
            cap2: PinConfig::default(),
            cap3: PinConfig::default(),
            cap4: PinConfig::default(),
            cap5: PinConfig::default(),
            cap6: PinConfig::default(),
        }
    }
}

impl<T, M> CaptureConfig7<T, M>
where
    T: CapturePeriph<M> + CapCmpTimer7<M>,
    M: PinMap,
{
    config_fn!(config_cap0_input_A, config_cap0_input_B, config_cap0_trigger, config_cap0_software, cap0, Gpio0);
    config_fn!(config_cap1_input_A, config_cap1_input_B, config_cap1_trigger, config_cap1_software, cap1, Gpio1);
    config_fn!(config_cap2_input_A, config_cap2_input_B, config_cap2_trigger, config_cap2_software, cap2, Gpio2);
    config_fn!(config_cap3_input_A, config_cap3_input_B, config_cap3_trigger, config_cap3_software, cap3, Gpio3);
    config_fn!(config_cap4_input_A, config_cap4_input_B, config_cap4_trigger, config_cap4_software, cap4, Gpio4);
    config_fn!(config_cap5_input_A, config_cap5_input_B, config_cap5_trigger, config_cap5_software, cap5, Gpio5);
    config_fn!(config_cap6_input_A, config_cap6_input_B, config_cap6_trigger, config_cap6_software, cap6, Gpio6);

    /// Writes all previously configured timer and capture settings into peripheral registers (TBxCTL and
    /// TBxCCTLn: SLAU445I Table 14-6, p. 409; SLAU445I Table 14-8, p. 411), then starts the timer in
    /// continuous mode (SLAU445I 14.2.3.2, p. 395).
    pub fn commit(self) -> CaptureParts7<T, M> {
        let timer = self.timer;
        self.config.write_regs(&timer);
        // Capture settings are written while the timer is stopped (SLAU445I 13.2.7, p. 382; 14.2.7, p. 407)
        CCRn::<CCR0>::config_cap_mode(&timer, self.cap0.trigger.into(), self.cap0.select);
        CCRn::<CCR1>::config_cap_mode(&timer, self.cap1.trigger.into(), self.cap1.select);
        CCRn::<CCR2>::config_cap_mode(&timer, self.cap2.trigger.into(), self.cap2.select);
        CCRn::<CCR3>::config_cap_mode(&timer, self.cap3.trigger.into(), self.cap3.select);
        CCRn::<CCR4>::config_cap_mode(&timer, self.cap4.trigger.into(), self.cap4.select);
        CCRn::<CCR5>::config_cap_mode(&timer, self.cap5.trigger.into(), self.cap5.select);
        CCRn::<CCR6>::config_cap_mode(&timer, self.cap6.trigger.into(), self.cap6.select);
        timer.continuous();

        CaptureParts7 {
            cap0: Capture::new(),
            cap1: Capture::new(),
            cap2: Capture::new(),
            cap3: Capture::new(),
            cap4: Capture::new(),
            cap5: Capture::new(),
            cap6: Capture::new(),
            tbxiv: TBxIV(PhantomData, PhantomData),
        }
    }
}

/// Collection of capture pins derived from timer peripheral with 2 capture-compare registers
///
/// The timers with 2 capture/compare registers are TA2 and TA3 on the MSP430FR2433 (SLASE59F Table 6-13,
/// p. 51; SLASE59F Table 6-14, p. 52).
pub struct CaptureParts2<T, M = DefaultMapping>
where
    T: CapCmpTimer2<M>,
    M: PinMap,
{
    /// Capture pin 0 (derived from capture-compare register 0, TAxCCR0: SLAU445I Table 13-7, p. 388)
    pub cap0: Capture<T, CCR0>,
    /// Capture pin 1 (derived from capture-compare register 1, TAxCCR1: SLAU445I Table 13-7, p. 388)
    pub cap1: Capture<T, CCR1>,
    /// Interrupt vector register (TAxIV: SLAU445I Table 13-8, p. 388)
    pub tbxiv: TBxIV<T, M>,
}

/// Collection of capture pins derived from timer peripheral with 3 capture-compare registers
///
/// The timers with 3 capture/compare registers, by device: TB0 to TB2 on the MSP430FR2x5x (SLASEC4D
/// Table 6-16, p. 73; SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-18, p. 74), TA0 and TA1 on the
/// MSP430FR2433 (SLASE59F Table 6-11, p. 50; SLASE59F Table 6-12, p. 51), TA0 to TA3 on the MSP430FR247x
/// (SLASEO7C Table 9-12, p. 55; SLASEO7C Table 9-13, p. 56; SLASEO7C Table 9-14, p. 58), and TA0 and TA1
/// on the MSP430FR25x2 (SLASEE4C Figure 6-2, p. 54).
pub struct CaptureParts3<T, M = DefaultMapping>
where
    T: CapCmpTimer3<M>,
    M: PinMap,
{
    /// Capture pin 0 (derived from capture-compare register 0: SLAU445I Table 13-7, p. 388; SLAU445I
    /// Table 14-9, p. 413)
    pub cap0: Capture<T, CCR0>,
    /// Capture pin 1 (derived from capture-compare register 1: SLAU445I Table 13-7, p. 388; SLAU445I
    /// Table 14-9, p. 413)
    pub cap1: Capture<T, CCR1>,
    /// Capture pin 2 (derived from capture-compare register 2: SLAU445I Table 13-7, p. 388; SLAU445I
    /// Table 14-9, p. 413)
    pub cap2: Capture<T, CCR2>,
    /// Interrupt vector register (TAxIV/TBxIV: SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414)
    pub tbxiv: TBxIV<T, M>,
}

/// Collection of capture pins derived from timer peripheral with 7 capture-compare registers
///
/// The timers with 7 capture/compare registers are TB3 on the MSP430FR2x5x (SLASEC4D Table 6-19, p. 75)
/// and TB0 on the MSP430FR247x (SLASEO7C Table 9-15, p. 59). Both are Timer_B.
pub struct CaptureParts7<T, M = DefaultMapping>
where
    T: CapCmpTimer7<M>,
    M: PinMap,
{
    /// Capture pin 0 (derived from capture-compare register 0, TBxCCR0: SLAU445I Table 14-9, p. 413)
    pub cap0: Capture<T, CCR0>,
    /// Capture pin 1 (derived from capture-compare register 1, TBxCCR1: SLAU445I Table 14-9, p. 413)
    pub cap1: Capture<T, CCR1>,
    /// Capture pin 2 (derived from capture-compare register 2, TBxCCR2: SLAU445I Table 14-9, p. 413)
    pub cap2: Capture<T, CCR2>,
    /// Capture pin 3 (derived from capture-compare register 3, TBxCCR3: SLAU445I Table 14-9, p. 413)
    pub cap3: Capture<T, CCR3>,
    /// Capture pin 4 (derived from capture-compare register 4, TBxCCR4: SLAU445I Table 14-9, p. 413)
    pub cap4: Capture<T, CCR4>,
    /// Capture pin 5 (derived from capture-compare register 5, TBxCCR5: SLAU445I Table 14-9, p. 413)
    pub cap5: Capture<T, CCR5>,
    /// Capture pin 6 (derived from capture-compare register 6, TBxCCR6: SLAU445I Table 14-9, p. 413)
    pub cap6: Capture<T, CCR6>,
    /// Interrupt vector register (TBxIV: SLAU445I Table 14-10, p. 414)
    pub tbxiv: TBxIV<T, M>,
}

/// Single capture pin with its own capture register: a capture/compare block in capture mode, which copies
/// the timer value into its TAxCCRn/TBxCCRn (SLAU445I 13.2.4.1, p. 374; SLAU445I Table 13-7, p. 388;
/// SLAU445I Table 14-9, p. 413)
pub struct Capture<T: CapCmp<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmp<C>, C> Capture<T, C> {
    fn new() -> Self { Self(PhantomData, PhantomData) }
}

// Candidate for embedded_hal inclusion
/// Single input capture pin (a capture/compare block in capture mode: SLAU445I 13.2.4.1, p. 374)
pub trait CapturePin {
    /// Type  of value returned by capture
    type Capture;
    /// Enumeration of `Capture` errors
    ///
    /// Possible errors:
    ///
    /// - *overcapture*, the previous capture value was overwritten because it
    ///   was not read in a timely manner (COV: SLAU445I 13.2.4.1, p. 375)
    type Error;

    /// "Waits" for a transition in the capture `channel` and returns the value
    /// of counter at that instant
    fn capture(&mut self) -> nb::Result<Self::Capture, Self::Error>;
}

impl<T: CapCmp<C>, C> CapturePin for Capture<T, C> {
    type Capture = u16;
    type Error = OverCapture;

    #[inline]
    fn capture(&mut self) -> nb::Result<Self::Capture, Self::Error> {
        let timer = unsafe { T::steal() };
        // The capture register is read once CCIFG is set (SLAU445I 13.2.4.1, p. 374, note "Reading
        // TAxCCRn in Capture mode")
        if timer.ccifg_rd() {
            let ccrn = timer.get_ccrn();
            timer.ccifg_clr();
            read_overcapture::<T, C>(ccrn).map_err(nb::Error::Other)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: CapCmp<C>, C> Capture<T, C> {
    #[inline]
    /// Enable capture interrupts (CCIE: SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_set();
    }

    #[inline]
    /// Disable capture interrupts (CCIE: SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_clr();
    }

    #[inline]
    /// Start a capture from software, on a capture pin set up with its `config_capN_software()` method:
    /// switches the capture input between GND and VCC (SLAU445I 13.2.4.1.1, p. 376; 14.2.4.1.1, p. 399).
    /// The capture records the timer count at the next timer clock edge (SCS = 1: SLAU445I 13.2.4.1,
    /// p. 375); read it with [`capture()`](CapturePin::capture).
    pub fn trigger_capture(&mut self) {
        let timer = unsafe { T::steal() };
        timer.toggle_ccis_low_bit();
    }
}

/// Check COV after reading a capture and clearing its CCIFG. COV is set by a capture that comes before
/// the previous capture was read (SLAU445I 13.2.4.1, p. 375; SLAU445I Figure 13-11, p. 375, "Capture
/// Cycle"; SLAU445I 14.2.4.1, p. 398). So a capture that arrives between the read and the clear doesn't set
/// COV, and the clear hides its CCIFG; as it is never read, the next capture sets COV, so it is reported as
/// an overcapture then instead of being lost silently.
#[inline(always)]
fn read_overcapture<T: CapCmp<C>, C>(ccrn: u16) -> Result<u16, OverCapture> {
    let timer = unsafe { T::steal() };
    let (cov, _) = timer.cov_ccifg_rd();
    if cov {
        timer.cov_clr();
        Err(OverCapture(ccrn))
    } else {
        Ok(ccrn)
    }
}

impl<T: CapCmp<CCR0>> Capture<T, CCR0> {
    /// Read the capture from CCR0's own interrupt handler.
    ///
    /// CCR0 has a dedicated interrupt vector, and its capture flag is cleared automatically when
    /// that interrupt is serviced (SLAU445I 13.2.6.1, p. 380; 14.2.6.1, p. 405), so `capture()` finds
    /// nothing there. Only call this from the CCR0 interrupt handler: anywhere else it returns the last
    /// capture again.
    #[inline]
    pub fn interrupt_capture(&mut self) -> Result<u16, OverCapture> {
        let timer = unsafe { T::steal() };
        read_overcapture::<T, CCR0>(timer.get_ccrn())
    }
}

/// Error returned when the previous capture was overwritten before being read (COV: SLAU445I 13.2.4.1,
/// p. 375)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OverCapture(pub u16);

/// Capture TBIV interrupt vector: the TAxIV/TBxIV values, from no interrupt through CCR1 to CCR6 to the
/// timer overflow (SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414). CCR0 has its own vector
/// (SLAU445I 13.2.6.1, p. 380; 14.2.6.1, p. 405).
pub enum CaptureVector<T> {
    /// No pending interrupt
    NoInterrupt,
    /// Interrupt caused by capture register 1.
    Capture1(InterruptCapture<T, CCR1>),
    /// Interrupt caused by capture register 2.
    Capture2(InterruptCapture<T, CCR2>),
    /// Interrupt caused by capture register 3.
    Capture3(InterruptCapture<T, CCR3>),
    /// Interrupt caused by capture register 4.
    Capture4(InterruptCapture<T, CCR4>),
    /// Interrupt caused by capture register 5.
    Capture5(InterruptCapture<T, CCR5>),
    /// Interrupt caused by capture register 6.
    Capture6(InterruptCapture<T, CCR6>),
    /// Interrupt caused by main timer overflow
    MainTimer,
}

/// Token returned when reading the interrupt vector that allows a one-time read of the capture
/// register corresponding to the interrupt (TAxCCRn/TBxCCRn: SLAU445I Table 13-7, p. 388; SLAU445I
/// Table 14-9, p. 413).
pub struct InterruptCapture<T, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmp<C>, C> InterruptCapture<T, C> {
    /// Performs a one-time capture read without considering the interrupt flag. Always call this
    /// instead of `capture()` after reading the capture interrupt vector, since reading the vector
    /// already clears the interrupt flag that `capture()` checks for (SLAU445I 13.2.6.2, p. 380; 14.2.6.2,
    /// p. 405).
    #[inline]
    pub fn interrupt_capture(self, _cap: &mut Capture<T, C>) -> Result<u16, OverCapture> {
        let timer = unsafe { T::steal() };
        read_overcapture::<T, C>(timer.get_ccrn())
    }
}

/// Interrupt vector register for determining which capture-register caused an ISR (TAxIV/TBxIV: SLAU445I
/// Table 13-8, p. 388; SLAU445I Table 14-10, p. 414)
pub struct TBxIV<T: TimerPeriph<M>, M: PinMap = DefaultMapping>(PhantomData<T>, PhantomData<M>);

impl<T: TimerPeriph<M>, M: PinMap> TBxIV<T, M> {
    #[inline]
    /// Read the capture interrupt vector and resets corresponding interrupt flag (SLAU445I 13.2.6.2,
    /// p. 380; 14.2.6.2, p. 405). If the vector corresponds to an available capture, a one-time capture
    /// read token will be returned as well.
    pub fn interrupt_vector(&mut self) -> CaptureVector<T> {
        let timer = unsafe { T::steal() };
        match timer.tbxiv_rd() {
            TimerVector::NoInterrupt => CaptureVector::NoInterrupt,
            TimerVector::SubTimer1 => {
                CaptureVector::Capture1(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer2 => {
                CaptureVector::Capture2(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer3 => {
                CaptureVector::Capture3(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer4 => {
                CaptureVector::Capture4(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer5 => {
                CaptureVector::Capture5(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer6 => {
                CaptureVector::Capture6(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::MainTimer => CaptureVector::MainTimer,
        }
    }
}
