//! GPIO batch configuration.
//!
//! The `Batch` abstraction allows "collecting" changes to the configurations of different pins on
//! a GPIO port and writing the changes to the hardware all at once.
//! Changes to individual pins are performed **statically** using typestates, so the number of
//! register writes are minimized.
//!
//! For example, `P2.batch().config_pin3(|p| p.to_input_pullup()).config_pin1(|p| p.to_output()).split(&pmm)`
//! configures P2.3 as a pullup input pin and P2.1 as an output pin and then writes the
//! configuration to the hardware in a single set of writes.
//!
//! The writes go to the port's PxIE, PxSELC, PxSEL0, PxSEL1, PxOUT, PxDIR and PxREN registers (SLAU445I
//! Tables 8-10 to 8-17, p. 334 to p. 336).

use crate::gpio::*;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::pmm::Pmm;
use crate::util::BitsExt;
use core::marker::PhantomData;

/// Proxy for a GPIO pin used for batch writes.
///
/// Configuring the proxy only changes the typestate of the proxy. Registers are only written once
/// all the proxies for the GPIO port are "committed". The values written for each typestate follow
/// SLAU445I Table 8-1, p. 313 (direction and pull resistor) and SLAU445I Table 8-3, p. 314 (function).
pub struct PinProxy<PORT: PortNum, PIN: PinNum, DIR> {
    _port: PhantomData<PORT>,
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
}

macro_rules! make_proxy {
    () => {
        PinProxy { _port: PhantomData, _pin: PhantomData, _dir: PhantomData }
    };
}

impl<PORT: PortNum, PIN: PinNum, PULL> PinProxy<PORT, PIN, Input<PULL>> {
    /// Configures pin as pulldown input (PxDIR = 0, PxREN = 1, PxOUT = 0: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn pulldown(self) -> PinProxy<PORT, PIN, Input<Pulldown>> { make_proxy!() }

    /// Configures pin as pullup input (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn pullup(self) -> PinProxy<PORT, PIN, Input<Pullup>> { make_proxy!() }

    /// Configures pin as floating input (PxDIR = 0, PxREN = 0: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn floating(self) -> PinProxy<PORT, PIN, Input<Floating>> { make_proxy!() }

    /// Configures pin as output (PxDIR = 1: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn to_output(self) -> PinProxy<PORT, PIN, Output> { make_proxy!() }
}

impl<PORT: PortNum, PIN: PinNum> PinProxy<PORT, PIN, Output> {
    /// Configures pin as floating input (PxDIR = 0, PxREN = 0: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn to_input_floating(self) -> PinProxy<PORT, PIN, Input<Floating>> { make_proxy!() }

    /// Configures pin as pullup input (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn to_input_pullup(self) -> PinProxy<PORT, PIN, Input<Pullup>> { make_proxy!() }

    /// Configures pin as pulldown input (PxDIR = 0, PxREN = 1, PxOUT = 0: SLAU445I Table 8-1, p. 313)
    #[inline(always)]
    pub fn to_input_pulldown(self) -> PinProxy<PORT, PIN, Input<Pulldown>> { make_proxy!() }
}

// The transitions between GPIO / alternate modes all have the same shape, we'll make a macro for them.
/// Implements a PinProxy transition.
macro_rules! pinproxy_transition {
    ($from:ty => $into:ty, $fn_name:ident(), $desc:literal) => {
        impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> PinProxy<PORT, PIN, $from> {
            #[doc = $desc]
            #[inline(always)]
            pub fn $fn_name(self) -> PinProxy<PORT, PIN, $into> { make_proxy!() }
        }
    };
    ($from:ty : $trait:path => $into:ty, $fn_name:ident(), $desc:literal) => {
        impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> PinProxy<PORT, PIN, $from>
        where Pin<PORT, PIN, DIR>: $trait
        {
            #[doc = $desc]
            #[inline(always)]
            pub fn $fn_name(self) -> PinProxy<PORT, PIN, $into> { make_proxy!() }
        }
    };
}

// GPIO to alternates: PxSEL1/PxSEL0 from 00 to 01, 10 or 11 (SLAU445I Table 8-3, p. 314)
pinproxy_transition!(DIR: ToAlternate1 => Alternate1<DIR>, to_alternate1(), "Convert pin to GPIO alternate function 1");
pinproxy_transition!(DIR: ToAlternate2 => Alternate2<DIR>, to_alternate2(), "Convert pin to GPIO alternate function 2");
pinproxy_transition!(DIR: ToAlternate3 => Alternate3<DIR>, to_alternate3(), "Convert pin to GPIO alternate function 3");

// Alternates to GPIO: PxSEL1/PxSEL0 back to 00 (SLAU445I Table 8-3, p. 314)
pinproxy_transition!(Alternate1<DIR> => DIR, to_gpio(), "Convert pin to GPIO function");
pinproxy_transition!(Alternate2<DIR> => DIR, to_gpio(), "Convert pin to GPIO function");
pinproxy_transition!(Alternate3<DIR> => DIR, to_gpio(), "Convert pin to GPIO function");

// Alternate 1 to other alternates (PxSEL1/PxSEL0 = 01 to 10 or 11, SLAU445I Table 8-3, p. 314)
pinproxy_transition!(Alternate1<DIR>: ToAlternate2 => Alternate2<DIR>, to_alternate2(), "Convert pin to GPIO alternate function 2");
pinproxy_transition!(Alternate1<DIR>: ToAlternate3 => Alternate3<DIR>, to_alternate3(), "Convert pin to GPIO alternate function 3");

// Alternate 2 to other alternates (PxSEL1/PxSEL0 = 10 to 01 or 11, SLAU445I Table 8-3, p. 314)
pinproxy_transition!(Alternate2<DIR>: ToAlternate1 => Alternate1<DIR>, to_alternate1(), "Convert pin to GPIO alternate function 1");
pinproxy_transition!(Alternate2<DIR>: ToAlternate3 => Alternate3<DIR>, to_alternate3(), "Convert pin to GPIO alternate function 3");

// Alternate 3 to other alternates (PxSEL1/PxSEL0 = 11 to 01 or 10, SLAU445I Table 8-3, p. 314)
pinproxy_transition!(Alternate3<DIR>: ToAlternate1 => Alternate1<DIR>, to_alternate1(), "Convert pin to GPIO alternate function 1");
pinproxy_transition!(Alternate3<DIR>: ToAlternate2 => Alternate2<DIR>, to_alternate2(), "Convert pin to GPIO alternate function 2");

// To and from ADCPCTL mode (SYSCFG2.ADCPCTLx, SLAU445I Table 1-31, p. 82)
#[cfg(feature = "adcpctl")]
pinproxy_transition!(DIR: ToAdcPctl => AdcMode<DIR>, to_adc_mode(), "Convert pin to ADC mode (ADCPCTL set)");
#[cfg(feature = "adcpctl")]
pinproxy_transition!(AdcMode<DIR> => DIR, from_adc_mode(), "Return pin to the mode it was in prior to ADCPCTL mode");

// Traits for deciding the value of a pin's registers: PxDIR, PxOUT and PxREN as in SLAU445I Table 8-1,
// p. 313, PxSEL0 and PxSEL1 as in SLAU445I Table 8-3, p. 314
trait PxdirOn {}
trait PxoutSet {}
trait PxoutClr {}
trait PxrenOn {}
trait Pxsel0On {}
trait Pxsel1On {}

// Whether PxDIR is set: 1 makes the pin an output (SLAU445I Table 8-11, p. 334)
trait WritePxdir {
    fn pxdir_on(&self) -> bool;
}
impl<T> WritePxdir for T {
    #[inline(always)]
    default fn pxdir_on(&self) -> bool { false }
}
impl<T: PxdirOn> WritePxdir for T {
    #[inline(always)]
    fn pxdir_on(&self) -> bool { true }
}

// Whether PxOUT is set during config: with PxREN = 1, PxOUT = 1 selects the pull-up (SLAU445I Table 8-10,
// p. 334)
trait WritePxoutSet {
    // The bit mask value - if true then the set bit mask is 1 (i.e. bit is set), if false then the bit mask is 0 (i.e. no effect)
    fn pxout_set_on(&self) -> bool;
}
impl<T> WritePxoutSet for T {
    #[inline(always)]
    default fn pxout_set_on(&self) -> bool { false }
}
impl<T: PxoutSet> WritePxoutSet for T {
    #[inline(always)]
    fn pxout_set_on(&self) -> bool { true }
}

// Whether PxOUT is cleared during config: with PxREN = 1, PxOUT = 0 selects the pull-down (SLAU445I
// Table 8-10, p. 334)
trait WritePxoutClr {
    // The bit mask value - if true then the clear bit mask is 1 (i.e. no effect), if false then the bit mask is 0 (i.e. bit is cleared)
    fn pxout_clr_on(&self) -> bool;
}
impl<T> WritePxoutClr for T {
    #[inline(always)]
    default fn pxout_clr_on(&self) -> bool { true }
}
impl<T: PxoutClr> WritePxoutClr for T {
    #[inline(always)]
    fn pxout_clr_on(&self) -> bool { false }
}

// Whether PxREN is set, enabling the pull resistor (SLAU445I Table 8-12, p. 335)
trait WritePxren {
    fn pxren_on(&self) -> bool;
}
impl<T> WritePxren for T {
    #[inline(always)]
    default fn pxren_on(&self) -> bool { false }
}
impl<T: PxrenOn> WritePxren for T {
    #[inline(always)]
    fn pxren_on(&self) -> bool { true }
}

// Whether PxSEL0 is set (SLAU445I Table 8-13, p. 335)
trait WritePxsel0 {
    fn pxsel0_on(&self) -> bool;
}
impl<T> WritePxsel0 for T {
    #[inline(always)]
    default fn pxsel0_on(&self) -> bool { false }
}
impl<T: Pxsel0On> WritePxsel0 for T {
    #[inline(always)]
    fn pxsel0_on(&self) -> bool { true }
}

// Whether PxSEL1 is set (SLAU445I Table 8-14, p. 335)
trait WritePxsel1 {
    fn pxsel1_on(&self) -> bool;
}
impl<T> WritePxsel1 for T {
    #[inline(always)]
    default fn pxsel1_on(&self) -> bool { false }
}
impl<T: Pxsel1On> WritePxsel1 for T {
    #[inline(always)]
    fn pxsel1_on(&self) -> bool { true }
}

// Register marker trait implementations. PxDIR, PxREN and PxOUT follow SLAU445I Table 8-1, p. 313,
// PxSEL0 and PxSEL1 follow SLAU445I Table 8-3, p. 314. A pin in an alternate function keeps the PxDIR of
// its type, because "PxDIR bits for I/O pins that are selected for other functions must be set as
// required by the other function" (SLAU445I 8.2.3, p. 313).
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Output> {}
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Alternate1<Output>> {}
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Alternate2<Output>> {}
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Alternate3<Output>> {}

impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Input<Pullup>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Input<Pulldown>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate1<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate1<Input<Pulldown>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate2<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate2<Input<Pulldown>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate3<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate3<Input<Pulldown>>> {}

impl<PORT: PortNum, PIN: PinNum> PxoutSet for PinProxy<PORT, PIN, Input<Pullup>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutSet for PinProxy<PORT, PIN, Alternate1<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutSet for PinProxy<PORT, PIN, Alternate2<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutSet for PinProxy<PORT, PIN, Alternate3<Input<Pullup>>> {}

impl<PORT: PortNum, PIN: PinNum> PxoutClr for PinProxy<PORT, PIN, Input<Pulldown>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutClr for PinProxy<PORT, PIN, Alternate1<Input<Pulldown>>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutClr for PinProxy<PORT, PIN, Alternate2<Input<Pulldown>>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutClr for PinProxy<PORT, PIN, Alternate3<Input<Pulldown>>> {}

impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel0On for PinProxy<PORT, PIN, Alternate1<DIR>> {}
impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel0On for PinProxy<PORT, PIN, Alternate3<DIR>> {}

impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel1On for PinProxy<PORT, PIN, Alternate2<DIR>> {}
impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel1On for PinProxy<PORT, PIN, Alternate3<DIR>> {}

// Derive bitmasks for different GPIO registers from pin numbers and register trait implementations. Bit x
// of each port register belongs to pin x (SLAU445I Table 8-13, p. 335).
trait MaskRegisters {
    fn pxout_set_mask(&self) -> u8;
    fn pxout_clr_mask(&self) -> u8;
    fn pxdir_mask(&self) -> u8;
    fn pxren_mask(&self) -> u8;
    fn pxsel0_mask(&self) -> u8;
    fn pxsel1_mask(&self) -> u8;
}

impl<PORT: PortNum, PIN: PinNum, DIR> MaskRegisters for PinProxy<PORT, PIN, DIR> {
    #[inline(always)]
    fn pxout_set_mask(&self) -> u8 { (self.pxout_set_on() as u8) << PIN::NUM }

    #[inline(always)]
    fn pxout_clr_mask(&self) -> u8 { (self.pxout_clr_on() as u8) << PIN::NUM }

    #[inline(always)]
    fn pxdir_mask(&self) -> u8 { (self.pxdir_on() as u8) << PIN::NUM }

    #[inline(always)]
    fn pxren_mask(&self) -> u8 { (self.pxren_on() as u8) << PIN::NUM }

    #[inline(always)]
    fn pxsel0_mask(&self) -> u8 { (self.pxsel0_on() as u8) << PIN::NUM }

    #[inline(always)]
    fn pxsel1_mask(&self) -> u8 { (self.pxsel1_on() as u8) << PIN::NUM }
}

// SYSCFG2.ADCPCTLx, on the devices that select ADC inputs there (SLAU445I Table 1-31, p. 82): set for a pin
// in `AdcMode`, cleared for another pin that has an ADC input, and left alone for the other pins
#[cfg(feature = "adcpctl")]
trait WriteAdcPctl {
    // The bits to set, and the bits to keep (0 clears the bit)
    fn adcpctl_set_mask(&self) -> u16;
    fn adcpctl_keep_mask(&self) -> u16;
}
#[cfg(feature = "adcpctl")]
impl<T> WriteAdcPctl for T {
    #[inline(always)]
    default fn adcpctl_set_mask(&self) -> u16 { 0 }
    #[inline(always)]
    default fn adcpctl_keep_mask(&self) -> u16 { !0 }
}
#[cfg(feature = "adcpctl")]
impl<PORT: PortNum, PIN: PinNum, DIR> WriteAdcPctl for PinProxy<PORT, PIN, DIR>
where Pin<PORT, PIN, DIR>: ToAdcPctl
{
    #[inline(always)]
    default fn adcpctl_set_mask(&self) -> u16 { 0 }
    #[inline(always)]
    default fn adcpctl_keep_mask(&self) -> u16 { <Pin<PORT, PIN, DIR> as ToAdcPctl>::CLR_MASK }
}
#[cfg(feature = "adcpctl")]
impl<PORT: PortNum, PIN: PinNum, DIR> WriteAdcPctl for PinProxy<PORT, PIN, AdcMode<DIR>>
where Pin<PORT, PIN, AdcMode<DIR>>: ToAdcPctl
{
    #[inline(always)]
    fn adcpctl_set_mask(&self) -> u16 { <Pin<PORT, PIN, AdcMode<DIR>> as ToAdcPctl>::SET_MASK }
    #[inline(always)]
    fn adcpctl_keep_mask(&self) -> u16 { !0 }
}

// Only ports with interrupts have PxIE (SLAU445I 8.2.6, p. 314)
trait InterruptOperations {
    fn maybe_write_pxie(&self, b: u8);
}

impl<P: GpioPeriph> InterruptOperations for P {
    #[inline(always)]
    default fn maybe_write_pxie(&self, _b: u8) {}
}

impl<P: IntrPeriph> InterruptOperations for P {
    // PxIE (SLAU445I Table 8-17, p. 336)
    #[inline(always)]
    fn maybe_write_pxie(&self, b: u8) { self.pxie_wr(b); }
}

/// What the terminating methods of [`Batch`] make of a pin's typestate: [`pulldown_all`](Batch::pulldown_all),
/// [`pullup_all`](Batch::pullup_all) and [`pulldown_unused`](Batch::pulldown_unused). The slots of pins the
/// device doesn't have stay [`Unavailable`].
#[doc(hidden)]
pub trait Termination {
    /// After `pulldown_all`
    type Pulldown;
    /// After `pullup_all`
    type Pullup;
    /// After `pulldown_unused`: floating inputs get their pulldown, everything else stays
    type UnusedPulldown;
}

impl Termination for Input<Floating> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Pd;
}
impl Termination for Input<Pulldown> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Pd;
}
impl Termination for Input<Pullup> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Pu;
}
impl Termination for Output {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Output;
}
impl<DIR> Termination for Alternate1<DIR> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Alternate1<DIR>;
}
impl<DIR> Termination for Alternate2<DIR> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Alternate2<DIR>;
}
impl<DIR> Termination for Alternate3<DIR> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = Alternate3<DIR>;
}
#[cfg(feature = "adcpctl")]
impl<DIR> Termination for AdcMode<DIR> {
    type Pulldown = Pd;
    type Pullup = Pu;
    type UnusedPulldown = AdcMode<DIR>;
}
impl Termination for Unavailable {
    type Pulldown = Unavailable;
    type Pullup = Unavailable;
    type UnusedPulldown = Unavailable;
}

/// Where a [`Batch`] starts from: a port whose registers hold their reset values ([`FromReset`]), or
/// pins configured before ([`FromParts`])
#[doc(hidden)]
pub trait BatchStart {
    /// Whether the port's registers hold their reset values
    const AFTER_RESET: bool;
}

/// A [`Batch`] made by [`Batch::new`], from a port whose registers hold their reset values
pub struct FromReset;
/// A [`Batch`] made by [`Parts::batch`], from pins configured before
pub struct FromParts;

impl BatchStart for FromReset {
    const AFTER_RESET: bool = true;
}
impl BatchStart for FromParts {
    const AFTER_RESET: bool = false;
}

impl<P: PortNum + PortPins>
    Batch<P, P::Init0, P::Init1, P::Init2, P::Init3, P::Init4, P::Init5, P::Init6, P::Init7, FromReset>
{
    /// Split into a batch of individual GPIO pin proxies. The pin slots the device has no pin for
    /// start out [`Unavailable`]. The pins that exist start out as floating inputs, as after a reset
    /// (SLAU445I 8.3.1, p. 316).
    ///
    /// The batch writes only the register bits that differ from their reset values. After every reset,
    /// PxDIR, PxREN, PxSEL0, PxSEL1 and PxIE are 00h (SLAU445I Table 8-4, p. 319; "After a POR or PUC
    /// reset, all port pins are configured as inputs with their module function disabled", SLAU445I
    /// 8.3.1, p. 316), and so are the ADCPCTLx bits of SYSCFG2 (SLAU445I Table 1-31, p. 82), also after a
    /// wake-up from LPMx.5 ("Upon exit from LPMx.5, all peripheral registers are set to their default
    /// conditions", SLAU445I 8.3.3, p. 318). If the program wrote the port's registers before, through
    /// the PAC, configure the pins through [`Parts::batch`], which writes every register:
    /// `Batch::new(port).split(&pmm).batch()`.
    pub fn new(_port: P) -> Self { Self::create() }
}

/// Collection of proxies for pins 0 to 7 of a specific port, used to commit configurations for
/// all pins in a single step. `START` is where it starts from: [`FromReset`] from [`Batch::new`],
/// [`FromParts`] from [`Parts::batch`].
pub struct Batch<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7, START = FromReset> {
    _start: PhantomData<START>,
    pin0: PinProxy<PORT, Pin0, DIR0>,
    pin1: PinProxy<PORT, Pin1, DIR1>,
    pin2: PinProxy<PORT, Pin2, DIR2>,
    pin3: PinProxy<PORT, Pin3, DIR3>,
    pin4: PinProxy<PORT, Pin4, DIR4>,
    pin5: PinProxy<PORT, Pin5, DIR5>,
    pin6: PinProxy<PORT, Pin6, DIR6>,
    pin7: PinProxy<PORT, Pin7, DIR7>,
}

type Pd = Input<Pulldown>;
type Pu = Input<Pullup>;

impl<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7, START: BatchStart>
    Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7, START>
{
    #[inline]
    fn write_regs(&self) {
        let pxdir = 0u8
            .set_mask(self.pin0.pxdir_mask())
            .set_mask(self.pin1.pxdir_mask())
            .set_mask(self.pin2.pxdir_mask())
            .set_mask(self.pin3.pxdir_mask())
            .set_mask(self.pin4.pxdir_mask())
            .set_mask(self.pin5.pxdir_mask())
            .set_mask(self.pin6.pxdir_mask())
            .set_mask(self.pin7.pxdir_mask());

        let pxout_set = 0u8
            .set_mask(self.pin0.pxout_set_mask())
            .set_mask(self.pin1.pxout_set_mask())
            .set_mask(self.pin2.pxout_set_mask())
            .set_mask(self.pin3.pxout_set_mask())
            .set_mask(self.pin4.pxout_set_mask())
            .set_mask(self.pin5.pxout_set_mask())
            .set_mask(self.pin6.pxout_set_mask())
            .set_mask(self.pin7.pxout_set_mask());

        let pxout_clr = 0u8
            .set_mask(self.pin0.pxout_clr_mask())
            .set_mask(self.pin1.pxout_clr_mask())
            .set_mask(self.pin2.pxout_clr_mask())
            .set_mask(self.pin3.pxout_clr_mask())
            .set_mask(self.pin4.pxout_clr_mask())
            .set_mask(self.pin5.pxout_clr_mask())
            .set_mask(self.pin6.pxout_clr_mask())
            .set_mask(self.pin7.pxout_clr_mask());

        let pxren = 0u8
            .set_mask(self.pin0.pxren_mask())
            .set_mask(self.pin1.pxren_mask())
            .set_mask(self.pin2.pxren_mask())
            .set_mask(self.pin3.pxren_mask())
            .set_mask(self.pin4.pxren_mask())
            .set_mask(self.pin5.pxren_mask())
            .set_mask(self.pin6.pxren_mask())
            .set_mask(self.pin7.pxren_mask());

        let pxsel0 = 0u8
            .set_mask(self.pin0.pxsel0_mask())
            .set_mask(self.pin1.pxsel0_mask())
            .set_mask(self.pin2.pxsel0_mask())
            .set_mask(self.pin3.pxsel0_mask())
            .set_mask(self.pin4.pxsel0_mask())
            .set_mask(self.pin5.pxsel0_mask())
            .set_mask(self.pin6.pxsel0_mask())
            .set_mask(self.pin7.pxsel0_mask());

        let pxsel1 = 0u8
            .set_mask(self.pin0.pxsel1_mask())
            .set_mask(self.pin1.pxsel1_mask())
            .set_mask(self.pin2.pxsel1_mask())
            .set_mask(self.pin3.pxsel1_mask())
            .set_mask(self.pin4.pxsel1_mask())
            .set_mask(self.pin5.pxsel1_mask())
            .set_mask(self.pin6.pxsel1_mask())
            .set_mask(self.pin7.pxsel1_mask());

        let p = unsafe { PORT::steal() };
        // From the reset values (see `Batch::new`), only the bits that differ from them are written. The
        // compiler knows which from the types.
        let after_reset = START::AFTER_RESET;
        // Turn off interrupts first so nothing fires during subsequent register writes, which can set
        // PxIFG flags (SLAU445I 8.2.6, p. 315). PxIE: SLAU445I Table 8-17, p. 336. They're off after a
        // reset.
        if !after_reset {
            p.maybe_write_pxie(0);
        }
        // Pins whose PxSEL0 and PxSEL1 bits both change switch through PxSELC, so they don't pass
        // through another function on the way (SLAU445I 8.2.5, p. 314). After that, every
        // remaining change is a single bit. PxSEL0, PxSEL1 and PxSELC: SLAU445I Tables 8-13 to 8-15,
        // p. 335 to p. 336.
        if after_reset {
            // From 00h, the bits that change are the ones set, and PxSELC leaves the pins of function 3
            // with both of theirs, so PxSEL0 and PxSEL1 only need a write for other pins
            let both = pxsel0 & pxsel1;
            if both != 0 {
                p.pxselc_wr(both);
            }
            if pxsel0 != both {
                p.pxsel0_wr(pxsel0);
            }
            if pxsel1 != both {
                p.pxsel1_wr(pxsel1);
            }
        } else {
            let both = (p.pxsel0_rd() ^ pxsel0) & (p.pxsel1_rd() ^ pxsel1);
            if both != 0 {
                p.pxselc_wr(both);
            }
            p.pxsel0_wr(pxsel0);
            p.pxsel1_wr(pxsel1);
        }

        // Only write to PxOUT if we need to match the pull resistor state to the typestate,
        // otherwise keep it at its previous value.
        // Instead of a write(), use a set_bits() and a clear_bits() to allow for leaving unchanged,
        // and skip each when no pin needs it (the masks follow from the typestates, so the compiler
        // decides). PxOUT: SLAU445I Table 8-10, p. 334.
        if pxout_set != 0 {
            p.pxout_set(pxout_set);
        }
        if pxout_clr != !0 {
            p.pxout_clear(pxout_clr);
        }

        // PxDIR (SLAU445I Table 8-11, p. 334) and PxREN (SLAU445I Table 8-12, p. 335), unless they keep
        // their reset value
        if !after_reset || pxdir != 0 {
            p.pxdir_wr(pxdir);
        }
        if !after_reset || pxren != 0 {
            p.pxren_wr(pxren);
        }

        // The ADC inputs among the pins (SYSCFG2.ADCPCTLx, SLAU445I Table 1-31, p. 82). Setting the bit
        // "disables both the output driver and input Schmitt trigger" of the pin (SLASE59F Table 6-17,
        // p. 55; SLASEE4C Table 6-15, p. 58).
        #[cfg(feature = "adcpctl")]
        {
            let adc_set = self.pin0.adcpctl_set_mask()
                | self.pin1.adcpctl_set_mask()
                | self.pin2.adcpctl_set_mask()
                | self.pin3.adcpctl_set_mask()
                | self.pin4.adcpctl_set_mask()
                | self.pin5.adcpctl_set_mask()
                | self.pin6.adcpctl_set_mask()
                | self.pin7.adcpctl_set_mask();
            let adc_keep = self.pin0.adcpctl_keep_mask()
                & self.pin1.adcpctl_keep_mask()
                & self.pin2.adcpctl_keep_mask()
                & self.pin3.adcpctl_keep_mask()
                & self.pin4.adcpctl_keep_mask()
                & self.pin5.adcpctl_keep_mask()
                & self.pin6.adcpctl_keep_mask()
                & self.pin7.adcpctl_keep_mask();
            if adc_set != 0 {
                p.adcpctl_set(adc_set);
            }
            // After a reset there are none to clear
            if !after_reset && adc_keep != !0 {
                p.adcpctl_clr(adc_keep);
            }
        }
    }

    #[inline(always)]
    pub(super) fn create() -> Self {
        Self {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Commits all pin configurations to GPIO registers and returns GPIO parts and turns off all
    /// interrupt enable bits (PxIE, SLAU445I Table 8-17, p. 336), which are off already in a batch from
    /// [`Batch::new`].
    ///
    /// Note that the pin's interrupt flags may become set as a result of
    /// this operation (SLAU445I 8.2.6, p. 315).
    ///
    /// GPIO input/output operations only work after the LOCKLPM5 bit has been cleared (SLAU445I
    /// 8.3.1, p. 316), which is ensured when passing `&Pmm` into the method, since [`Pmm::new`]
    /// clears LOCKLPM5.
    /// With [`Pmm::new_locked`] the configuration takes effect once [`Pmm::unlock_lpm5`] is
    /// called (SLAU445I 8.3.3, p. 318: "Any changes to the port configuration registers while
    /// LOCKLPM5 is set have no effect on the I/O pins").
    #[inline]
    pub fn split(self, _pmm: &Pmm) -> Parts<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        self.write_regs();
        Parts::new()
    }

    /// Edit configuration of pin 0 (bit 0 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin0<NEW, F: FnOnce(PinProxy<PORT, Pin0, DIR0>) -> PinProxy<PORT, Pin0, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, NEW, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: f(self.pin0),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 1 (bit 1 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin1<NEW, F: FnOnce(PinProxy<PORT, Pin1, DIR1>) -> PinProxy<PORT, Pin1, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, NEW, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: f(self.pin1),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 2 (bit 2 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin2<NEW, F: FnOnce(PinProxy<PORT, Pin2, DIR2>) -> PinProxy<PORT, Pin2, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, NEW, DIR3, DIR4, DIR5, DIR6, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: f(self.pin2),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 3 (bit 3 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin3<NEW, F: FnOnce(PinProxy<PORT, Pin3, DIR3>) -> PinProxy<PORT, Pin3, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, NEW, DIR4, DIR5, DIR6, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: f(self.pin3),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 4 (bit 4 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin4<NEW, F: FnOnce(PinProxy<PORT, Pin4, DIR4>) -> PinProxy<PORT, Pin4, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, NEW, DIR5, DIR6, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: f(self.pin4),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 5 (bit 5 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin5<NEW, F: FnOnce(PinProxy<PORT, Pin5, DIR5>) -> PinProxy<PORT, Pin5, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, NEW, DIR6, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: f(self.pin5),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 6 (bit 6 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin6<NEW, F: FnOnce(PinProxy<PORT, Pin6, DIR6>) -> PinProxy<PORT, Pin6, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, NEW, DIR7, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: f(self.pin6),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 7 (bit 7 of the port registers, SLAU445I Table 8-13, p. 335)
    #[inline(always)]
    pub fn config_pin7<NEW, F: FnOnce(PinProxy<PORT, Pin7, DIR7>) -> PinProxy<PORT, Pin7, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, NEW, START> {
        Batch {
            _start: PhantomData,
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: f(self.pin7),
        }
    }

    /// Set all pins the device has to inputs with pulldowns (PxDIR = 0, PxREN = 1, PxOUT = 0: SLAU445I
    /// Table 8-1, p. 313), whatever they were configured as. Leaving unused pins as floating massively
    /// increases power usage (relatively speaking) (SLAU445I 8.3.2, p. 317). To keep the pins you have
    /// configured, use [`pulldown_unused`](Batch::pulldown_unused) instead.
    #[inline(always)]
    pub fn pulldown_all(
        self,
    ) -> Batch<PORT, DIR0::Pulldown, DIR1::Pulldown, DIR2::Pulldown, DIR3::Pulldown, DIR4::Pulldown, DIR5::Pulldown, DIR6::Pulldown, DIR7::Pulldown, START>
    where
        DIR0: Termination, DIR1: Termination, DIR2: Termination, DIR3: Termination,
        DIR4: Termination, DIR5: Termination, DIR6: Termination, DIR7: Termination,
    {
        Batch::create()
    }

    /// Set all pins the device has to inputs with pullups (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I
    /// Table 8-1, p. 313), whatever they were configured as. Leaving unused pins as floating massively
    /// increases power usage (relatively speaking) (SLAU445I 8.3.2, p. 317).
    #[inline(always)]
    pub fn pullup_all(
        self,
    ) -> Batch<PORT, DIR0::Pullup, DIR1::Pullup, DIR2::Pullup, DIR3::Pullup, DIR4::Pullup, DIR5::Pullup, DIR6::Pullup, DIR7::Pullup, START>
    where
        DIR0: Termination, DIR1: Termination, DIR2: Termination, DIR3: Termination,
        DIR4: Termination, DIR5: Termination, DIR6: Termination, DIR7: Termination,
    {
        Batch::create()
    }

    /// Give every pin that is still a floating input its pulldown resistor, so it doesn't float, and leave
    /// every other pin as configured. Call it after configuring the pins you use, before
    /// [`split`](Batch::split): an unused pin that floats draws extra current. The user's guide recommends
    /// terminating unused pins this way, or as outputs (SLAU445I
    /// 8.3.2, p. 317: "To prevent a floating input and to reduce power consumption, unused I/O pins should be
    /// configured as I/O function, output direction, and left unconnected on the PC board ... Alternatively,
    /// the integrated pullup or pulldown resistor can be enabled by setting the PxREN bit of the unused pin
    /// to prevent a floating input").
    ///
    /// A pin that should float, an input driven from outside, say, gets its pulldown too: configure it with
    /// `floating()` after this call. Pins in a module function keep their configuration, and so do pins
    /// that something on the board pulls up, such as a button: give those their pullup, or the pulldown
    /// draws current through the outside resistor.
    #[inline(always)]
    pub fn pulldown_unused(
        self,
    ) -> Batch<
        PORT,
        DIR0::UnusedPulldown,
        DIR1::UnusedPulldown,
        DIR2::UnusedPulldown,
        DIR3::UnusedPulldown,
        DIR4::UnusedPulldown,
        DIR5::UnusedPulldown,
        DIR6::UnusedPulldown,
        DIR7::UnusedPulldown,
        START,
    >
    where
        DIR0: Termination, DIR1: Termination, DIR2: Termination, DIR3: Termination,
        DIR4: Termination, DIR5: Termination, DIR6: Termination, DIR7: Termination,
    {
        Batch::create()
    }
}
