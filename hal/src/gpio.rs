//! GPIO abstractions for configuring and controlling individual GPIO pins.
//!
//! To specify any pin, use the bounds `Pin<PORT: PortNum, PIN: PinNum>`.
//! To specify any pin on a port that supports interrupts, use the bounds `Pin<PORT: IntrPortNum, PIN: PinNum>`.
//! To specify any pin on port Px, use the bounds `Pin<Px, PIN: PinNum>`.
//!
//! Note that interrupts are only supported by some hardware ports (e.g. Ports 1 to 4 on the MSP430FR2355), so interrupt-related
//! methods are only available on those pins (SLAU445I 8.1, p. 312; SLASEC4D 6.10.3, p. 69: "Interrupt
//! conditions are possible in P1, P2, P3, and P4").
//!
//! Pins can be converted to alternate functionalities 1 to 3, but the availability of these
//! conversions on each pin is limited by the hardware capabilities in the [`datasheet`], so not
//! every pin in every configuration can be converted to every alternate functionality (SLAU445I 8.2.5,
//! p. 314; the Port Px Pin Functions tables of each data sheet, such as SLASEC4D Tables 6-63 to 6-68,
//! p. 96 to p. 106).
//!
//! For devices that expose ADC functionality through the ADCPCTLx bits, a special ADC mode is provided
//! instead (SYSCFG2.ADCPCTLx, SLAU445I Table 1-31, p. 82).
//!
//! [`datasheet`]: http://www.ti.com/lit/ds/symlink/msp430fr2355.pdf

pub use crate::batch_gpio::*;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::hw_traits::Steal;
use crate::util::BitsExt;
use core::convert::Infallible;
use core::marker::PhantomData;

// Make PAC GPIO peripherals available as a re-export
pub use crate::device_specific::gpio::*;

mod sealed {
    use super::*;

    pub trait SealedPinNum {}
    pub trait SealedGpioFunction {}
    pub trait SealedAlternateMode {}

    impl SealedPinNum for Pin0 {}
    impl SealedPinNum for Pin1 {}
    impl SealedPinNum for Pin2 {}
    impl SealedPinNum for Pin3 {}
    impl SealedPinNum for Pin4 {}
    impl SealedPinNum for Pin5 {}
    impl SealedPinNum for Pin6 {}
    impl SealedPinNum for Pin7 {}

    impl SealedGpioFunction for Output {}
    impl<PULL> SealedGpioFunction for Input<PULL> {}

    impl<DIR> SealedAlternateMode for Alternate1<DIR> {}
    impl<DIR> SealedAlternateMode for Alternate2<DIR> {}
    impl<DIR> SealedAlternateMode for Alternate3<DIR> {}
}

/// Trait that encompasses all `Pinx` types for specifying a pin number. Pin x of a port is bit x of each
/// of the port's registers (SLAU445I Table 8-13, p. 335).
pub trait PinNum: sealed::SealedPinNum {
    // Pin number
    #[doc(hidden)]
    const NUM: u8;

    // Bitmask with all zeros except for the bit corresponding to the pin. Bit x of a port register
    // belongs to pin Px.x (SLAU445I Table 8-13, p. 335: "P1SEL1.5 ... is selected for P1.5").
    #[doc(hidden)]
    const SET_MASK: u8 = 1 << Self::NUM;
    // Bitmask with all ones except for the bit corresponding to the pin.
    #[doc(hidden)]
    const CLR_MASK: u8 = !Self::SET_MASK;
}

// Don't need to seal, since GpioPeriph is private
/// Marker trait that encompasses all GPIO port types
pub trait PortNum: GpioPeriph {}
impl<PORT: GpioPeriph> PortNum for PORT {}

// Don't need to seal, since PortNum is already sealed
/// Marker trait for all Ports that support interrupts (P1 and P2 always, other ports depending on the
/// device: SLAU445I 8.2.6, p. 314)
pub trait IntrPortNum: IntrPeriph {}
impl<PORT: IntrPeriph> IntrPortNum for PORT {}

/// Typestate of a pin slot that has no pin on this device, such as P3.3 to P3.7 on the
/// MSP430FR2433 (SLASE59F 6.11.4, p. 59: "Port P3 (P3.0 to P3.2)"). Its [`Pin`] has no methods.
pub struct Unavailable;

/// The typestates each pin slot of a port starts in: `Input<Floating>` for the pins that exist,
/// [`Unavailable`] for the ones the device doesn't have. After a reset every pin is an input with its
/// module function disabled (SLAU445I 8.3.1, p. 316), PxDIR, PxREN, PxSEL0 and PxSEL1 resetting to 0
/// (SLAU445I Table 8-11, p. 334; SLAU445I Tables 8-12 to 8-14, p. 335).
#[doc(hidden)]
pub trait PortPins {
    type Init0;
    type Init1;
    type Init2;
    type Init3;
    type Init4;
    type Init5;
    type Init6;
    type Init7;
}

// Implement PortPins for a port with `$count` pins. The pins a device lacks are always the top ones:
// P5.5 to P5.7 and P6.7 on the MSP430FR2x5x (SLASEC4D Table 6-67, p. 104 and SLASEC4D Table 6-68, p. 106),
// P3.3 to P3.7 on the MSP430FR2433 (SLASE59F Table 6-20, p. 59), P6.3 to P6.7 on the MSP430FR247x
// (SLASEO7C Table 9-28, p. 70) and P2.7 on the MSP430FR25x2 (SLASEE4C Table 6-16, p. 60).
macro_rules! impl_port_pins {
    (@impl $Px:ty, $i0:ty, $i1:ty, $i2:ty, $i3:ty, $i4:ty, $i5:ty, $i6:ty, $i7:ty) => {
        impl $crate::gpio::PortPins for $Px {
            type Init0 = $i0;
            type Init1 = $i1;
            type Init2 = $i2;
            type Init3 = $i3;
            type Init4 = $i4;
            type Init5 = $i5;
            type Init6 = $i6;
            type Init7 = $i7;
        }
    };
    ($Px:ty, 8) => { impl_port_pins!(@impl $Px, In, In, In, In, In, In, In, In); };
    ($Px:ty, 7) => { impl_port_pins!(@impl $Px, In, In, In, In, In, In, In, Unavailable); };
    ($Px:ty, 5) => { impl_port_pins!(@impl $Px, In, In, In, In, In, Unavailable, Unavailable, Unavailable); };
    ($Px:ty, 3) => {
        impl_port_pins!(@impl $Px, In, In, In, Unavailable, Unavailable, Unavailable, Unavailable, Unavailable);
    };
}
pub(crate) use impl_port_pins;

/// Starting typestate of the pins that exist
#[doc(hidden)]
pub type In = Input<Floating>;

/// Pin number 0
pub struct Pin0;
impl PinNum for Pin0 {
    const NUM: u8 = 0;
}

/// Pin number 1
pub struct Pin1;
impl PinNum for Pin1 {
    const NUM: u8 = 1;
}

/// Pin number 2
pub struct Pin2;
impl PinNum for Pin2 {
    const NUM: u8 = 2;
}

/// Pin number 3
pub struct Pin3;
impl PinNum for Pin3 {
    const NUM: u8 = 3;
}

/// Pin number 4
pub struct Pin4;
impl PinNum for Pin4 {
    const NUM: u8 = 4;
}

/// Pin number 5
pub struct Pin5;
impl PinNum for Pin5 {
    const NUM: u8 = 5;
}

/// Pin number 6
pub struct Pin6;
impl PinNum for Pin6 {
    const NUM: u8 = 6;
}

/// Pin number 7
pub struct Pin7;
impl PinNum for Pin7 {
    const NUM: u8 = 7;
}

/// Marker trait for GPIO typestates representing pins in GPIO (non-alternate) state (PxSEL1/PxSEL0 = 00,
/// SLAU445I Table 8-3, p. 314)
pub trait GpioFunction: sealed::SealedGpioFunction {}

/// Direction typestate for GPIO output (PxDIR = 1, SLAU445I Table 8-1, p. 313)
pub struct Output;
impl GpioFunction for Output {}

/// Direction typestate for GPIO input (PxDIR = 0, SLAU445I Table 8-1, p. 313).
/// The type parameter specifies pull direction of input.
pub struct Input<PULL>(PhantomData<PULL>);
impl<PULL> GpioFunction for Input<PULL> {}

/// Pull typestate for pullup inputs (PxREN = 1, PxOUT = 1: SLAU445I Table 8-1, p. 313)
pub struct Pullup;

/// Pull typestate for pulldown inputs (PxREN = 1, PxOUT = 0: SLAU445I Table 8-1, p. 313)
pub struct Pulldown;

/// Pull typestate for floating inputs (PxREN = 0: SLAU445I Table 8-1, p. 313)
pub struct Floating;

/// A single GPIO pin.
pub struct Pin<PORT: PortNum, PIN: PinNum, DIR> {
    _port: PhantomData<PORT>,
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
}

macro_rules! make_pin {
    () => {
        Pin { _port: PhantomData, _pin: PhantomData, _dir: PhantomData }
    };

    ($dir:ty) => {
        Pin::<_, _, $dir> { _port: PhantomData, _pin: PhantomData, _dir: PhantomData }
    };
}

impl<PORT: PortNum, PIN: PinNum, PULL> Pin<PORT, PIN, Input<PULL>> {
    /// Configures pin as pulldown input.
    /// This method requires a `Pxout` token because configuring pull direction requires setting
    /// the PxOUT register (SLAU445I 8.2.4, p. 313 and SLAU445I Table 8-1, p. 313), which can race with
    /// setting an output pin on the same port.
    #[inline]
    pub fn pulldown(self) -> Pin<PORT, PIN, Input<Pulldown>> {
        let p = unsafe { PORT::steal() };
        p.pxout_clear(PIN::CLR_MASK);
        p.pxren_set(PIN::SET_MASK);
        make_pin!()
    }

    /// Configures pin as pullup input.
    /// This method requires a `Pxout` token because configuring pull direction requires setting
    /// the PxOUT register (SLAU445I 8.2.4, p. 313 and SLAU445I Table 8-1, p. 313), which can race with
    /// setting an output pin on the same port.
    #[inline]
    pub fn pullup(self) -> Pin<PORT, PIN, Input<Pullup>> {
        let p = unsafe { PORT::steal() };
        p.pxout_set(PIN::SET_MASK);
        p.pxren_set(PIN::SET_MASK);
        make_pin!()
    }

    /// Configures pin as floating input (PxREN = 0, SLAU445I Table 8-1, p. 313)
    #[inline]
    pub fn floating(self) -> Pin<PORT, PIN, Input<Floating>> {
        let p = unsafe { PORT::steal() };
        p.pxren_clear(PIN::CLR_MASK);
        make_pin!()
    }
}

impl<PORT: IntrPortNum, PIN: PinNum, PULL> Pin<PORT, PIN, Input<PULL>> {
    /// Set interrupt trigger to rising edge and clear interrupt flag. PxIES = 0 selects the
    /// low-to-high transition, and writing PxIES can set the flag (SLAU445I 8.2.6.2, p. 316).
    #[inline]
    pub fn select_rising_edge_trigger(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxies_clear(PIN::CLR_MASK);
        p.pxifg_clear(PIN::CLR_MASK);
        self
    }

    /// Set interrupt trigger to falling edge and clear interrupt flag. PxIES = 1 selects the
    /// high-to-low transition, and writing PxIES can set the flag (SLAU445I 8.2.6.2, p. 316).
    #[inline]
    pub fn select_falling_edge_trigger(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxies_set(PIN::SET_MASK);
        p.pxifg_clear(PIN::CLR_MASK);
        self
    }

    /// Enable interrupts on input pin.
    /// Note that changing other GPIO configurations while interrupts are enabled can cause
    /// spurious interrupts (SLAU445I 8.2.6, p. 315: "Writing to PxOUT, PxDIR, or PxREN can result
    /// in setting the corresponding PxIFG flags").
    ///
    /// The trigger edge (PxIES) is undefined after a reset (SLAU445I Table 8-16, p. 336), so select
    /// it first with [`select_rising_edge_trigger`](Self::select_rising_edge_trigger) or
    /// [`select_falling_edge_trigger`](Self::select_falling_edge_trigger).
    ///
    /// After a reset, enable interrupts once LOCKLPM5 is cleared, and clear the flag with
    /// [`clear_ifg`](Self::clear_ifg) first: "After clearing LOCKLPM5, all interrupt flags should be
    /// cleared (...). Then port interrupts can be enabled by setting the corresponding PxIE bits"
    /// (SLAU445I 8.3.1, p. 316).
    #[inline]
    pub fn enable_interrupts(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxie_set(PIN::SET_MASK);
        self
    }

    /// Disable interrupts on input pin (PxIE, SLAU445I Table 8-17, p. 336).
    #[inline]
    pub fn disable_interrupt(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxie_clear(PIN::CLR_MASK);
        self
    }

    /// Set interrupt flag high, triggering an ISR if interrupts are enabled (SLAU445I 8.2.6, p. 315; PxIFG,
    /// SLAU445I Table 8-18, p. 337).
    #[inline]
    pub fn set_ifg(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxifg_set(PIN::SET_MASK);
        self
    }

    /// Clear interrupt flag (PxIFG, SLAU445I Table 8-18, p. 337).
    #[inline]
    pub fn clear_ifg(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxifg_clear(PIN::CLR_MASK);
        self
    }

    /// Wait for interrupt flag to go high nonblockingly. Clear the flag if high (PxIFG, SLAU445I Table 8-18,
    /// p. 337).
    #[inline]
    pub fn wait_for_ifg(&mut self) -> nb::Result<(), Infallible> {
        let p = unsafe { PORT::steal() };
        if p.pxifg_rd().check(PIN::NUM) != 0 {
            p.pxifg_clear(PIN::CLR_MASK);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// Interrupt vector register used to determine which pin caused a port ISR (PxIV, SLAU445I Tables 8-5 to
/// 8-8, p. 332 to p. 333)
pub struct PxIV<PORT: PortNum>(PhantomData<PORT>);

impl<PORT: IntrPortNum> PxIV<PORT> {
    /// When called inside an ISR, returns the pin number of the highest priority interrupt flag
    /// that's currently enabled. Automatically clears the same flag. For a given port, the lowest
    /// numbered pin has the highest interrupt priority (SLAU445I 8.2.6, p. 315).
    #[inline]
    pub fn get_interrupt_vector(&mut self) -> GpioVector {
        let p = unsafe { PORT::steal() };
        p.pxiv_rd()
    }
}

/// Indicates which pin on the GPIO port caused the ISR: PxIV reads 00h for none and 02h to 10h for
/// pins 0 to 7 (SLAU445I Tables 8-5 to 8-8, p. 332 to p. 333).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GpioVector {
    /// No ISR
    NoIsr,
    /// ISR caused by pin 0
    Pin0Isr,
    /// ISR caused by pin 1
    Pin1Isr,
    /// ISR caused by pin 2
    Pin2Isr,
    /// ISR caused by pin 3
    Pin3Isr,
    /// ISR caused by pin 4
    Pin4Isr,
    /// ISR caused by pin 5
    Pin5Isr,
    /// ISR caused by pin 6
    Pin6Isr,
    /// ISR caused by pin 7
    Pin7Isr,
}

impl<PORT: PortNum, PIN: PinNum, PULL> Pin<PORT, PIN, Input<PULL>> {
    /// Configures pin as output (PxDIR = 1, SLAU445I Table 8-1, p. 313).
    ///
    /// The pin drives whatever its PxOUT bit holds, which is undefined after a reset (SLAU445I
    /// Table 8-10, p. 334), or the pull-up/pull-down selection of an input with a pull resistor
    /// (SLAU445I 8.2.2, p. 313). Use [`to_output_low`](Self::to_output_low) or
    /// [`to_output_high`](Self::to_output_high) to start from a known level.
    #[inline]
    pub fn to_output(self) -> Pin<PORT, PIN, Output> {
        let p = unsafe { PORT::steal() };
        p.pxdir_set(PIN::SET_MASK);
        make_pin!()
    }

    /// Configures pin as output, driving it low from the start: PxOUT is cleared (SLAU445I Table 8-10,
    /// p. 334) before PxDIR is set (SLAU445I Table 8-11, p. 334)
    #[inline]
    pub fn to_output_low(self) -> Pin<PORT, PIN, Output> {
        let p = unsafe { PORT::steal() };
        p.pxout_clear(PIN::CLR_MASK);
        p.pxdir_set(PIN::SET_MASK);
        make_pin!()
    }

    /// Configures pin as output, driving it high from the start: PxOUT is set (SLAU445I Table 8-10, p. 334)
    /// before PxDIR is set (SLAU445I Table 8-11, p. 334)
    #[inline]
    pub fn to_output_high(self) -> Pin<PORT, PIN, Output> {
        let p = unsafe { PORT::steal() };
        p.pxout_set(PIN::SET_MASK);
        p.pxdir_set(PIN::SET_MASK);
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum> Pin<PORT, PIN, Output> {
    /// Configures pin as floating input (PxDIR = 0, SLAU445I Table 8-1, p. 313)
    #[inline]
    pub fn to_input_floating(self) -> Pin<PORT, PIN, Input<Floating>> {
        let p = unsafe { PORT::steal() };
        p.pxdir_clear(PIN::CLR_MASK);
        make_pin!(Input<Floating>).floating()
    }

    /// Configures pin as pullup input (PxDIR = 0, SLAU445I Table 8-1, p. 313)
    #[inline]
    pub fn to_input_pullup(self) -> Pin<PORT, PIN, Input<Pullup>> {
        let p = unsafe { PORT::steal() };
        p.pxdir_clear(PIN::CLR_MASK);
        make_pin!(Input<Floating>).pullup()
    }

    /// Configures pin as pulldown input (PxDIR = 0, SLAU445I Table 8-1, p. 313)
    #[inline]
    pub fn to_input_pulldown(self) -> Pin<PORT, PIN, Input<Pulldown>> {
        let p = unsafe { PORT::steal() };
        p.pxdir_clear(PIN::CLR_MASK);
        make_pin!(Input<Floating>).pulldown()
    }
}

/// GPIO parts for a specific port, including all 8 pins (a port has up to eight I/O lines, SLAU445I 8.1,
/// p. 312; the slots of missing pins are [`Unavailable`]).
pub struct Parts<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
    /// Pin0
    pub pin0: Pin<PORT, Pin0, DIR0>,
    /// Pin1
    pub pin1: Pin<PORT, Pin1, DIR1>,
    /// Pin2
    pub pin2: Pin<PORT, Pin2, DIR2>,
    /// Pin3
    pub pin3: Pin<PORT, Pin3, DIR3>,
    /// Pin4
    pub pin4: Pin<PORT, Pin4, DIR4>,
    /// Pin5
    pub pin5: Pin<PORT, Pin5, DIR5>,
    /// Pin6
    pub pin6: Pin<PORT, Pin6, DIR6>,
    /// Pin7
    pub pin7: Pin<PORT, Pin7, DIR7>,
    /// Interrupt vector register
    pub pxiv: PxIV<PORT>,
}

impl<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
    Parts<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
{
    /// Converts all parts into a GPIO batch so the entire port can be configured at once
    #[inline]
    pub fn batch(self) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        Batch::create()
    }

    #[inline]
    pub(super) fn new() -> Self {
        Self {
            pin0: make_pin!(),
            pin1: make_pin!(),
            pin2: make_pin!(),
            pin3: make_pin!(),
            pin4: make_pin!(),
            pin5: make_pin!(),
            pin6: make_pin!(),
            pin7: make_pin!(),
            pxiv: PxIV(PhantomData),
        }
    }
}

// Trait will not be used as a bound outside the HAL, since it's only used as an associated type
// bound inside the HAL, so just keep it hidden. It writes PxSEL0, PxSEL1 and PxSELC (SLAU445I Tables 8-13
// to 8-15, p. 335 to p. 336) and SYSCFG2.ADCPCTLx (SLAU445I Table 1-31, p. 82).
#[doc(hidden)]
pub trait ChangeSelectBits {
    fn set_sel0(&mut self);
    fn set_sel1(&mut self);
    fn clear_sel0(&mut self);
    fn clear_sel1(&mut self);
    fn flip_selc(&mut self);
    #[cfg(feature = "adcpctl")]
    fn set_adcpctl(&mut self, mask: u16);
    #[cfg(feature = "adcpctl")]
    fn clr_adcpctl(&mut self, mask: u16);
}

// Methods for managing sel1, sel0, and selc registers
impl<PORT: PortNum, PIN: PinNum, DIR> ChangeSelectBits for Pin<PORT, PIN, DIR> {
    // PxSEL0 (SLAU445I Table 8-13, p. 335)
    #[inline]
    fn set_sel0(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel0_set(PIN::SET_MASK);
    }

    // PxSEL1 (SLAU445I Table 8-14, p. 335)
    #[inline]
    fn set_sel1(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel1_set(PIN::SET_MASK);
    }

    // PxSEL0 (SLAU445I Table 8-13, p. 335)
    #[inline]
    fn clear_sel0(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel0_clear(PIN::CLR_MASK);
    }

    // PxSEL1 (SLAU445I Table 8-14, p. 335)
    #[inline]
    fn clear_sel1(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel1_clear(PIN::CLR_MASK);
    }

    // PxSELC (SLAU445I Table 8-15, p. 336)
    #[inline]
    fn flip_selc(&mut self) {
        let p = unsafe { PORT::steal() };
        // Change both sel0 and sel1 bits at once: each bit set in PxSELC complements that bit of
        // PxSEL0 and PxSEL1 (SLAU445I 8.2.5, p. 314 and SLAU445I Table 8-15, p. 336)
        p.pxselc_wr(0u8.set(PIN::NUM));
    }

    // SYSCFG2.ADCPCTLx (SLAU445I Table 1-31, p. 82)
    #[cfg(feature = "adcpctl")]
    #[inline]
    fn set_adcpctl(&mut self, mask: u16) {
        let p = unsafe { PORT::steal() };
        p.adcpctl_set(mask);
    }

    // SYSCFG2.ADCPCTLx (SLAU445I Table 1-31, p. 82)
    #[cfg(feature = "adcpctl")]
    #[inline]
    fn clr_adcpctl(&mut self, mask: u16) {
        let p = unsafe { PORT::steal() };
        p.adcpctl_clr(mask);
    }
}

/// Typestate for GPIO alternate function 1: PxSEL1/PxSEL0 = 01, the primary module function (SLAU445I
/// Table 8-3, p. 314)
pub struct Alternate1<DIR>(PhantomData<DIR>);

/// Typestate for GPIO alternate function 2: PxSEL1/PxSEL0 = 10, the secondary module function (SLAU445I
/// Table 8-3, p. 314)
pub struct Alternate2<DIR>(PhantomData<DIR>);

/// Typestate for GPIO alternate function 3: PxSEL1/PxSEL0 = 11, the tertiary module function (SLAU445I
/// Table 8-3, p. 314)
///
/// On the MSP430FR247x this function disconnects the pull resistor, whatever PxREN says: the port diagram
/// enables the resistor only while "PxSEL.x = 11" is false (SLASEO7C Figure 9-4, p. 64). Measured on an
/// MSP430FR2476: with its pullup on, an eCOMP input in this function stayed below 1.2 V. So a pull setting
/// from before has no effect here.
pub struct Alternate3<DIR>(PhantomData<DIR>);

// Only used as a bound inside the HAL, so keep it hidden
/// Alternate function typestates, with the PxSEL bits that select them (SLAU445I Table 8-3, p. 314)
#[doc(hidden)]
pub trait AlternateMode: sealed::SealedAlternateMode {
    /// PxSEL0 bit value that selects this function
    const SEL0: bool;
    /// PxSEL1 bit value that selects this function
    const SEL1: bool;
}

impl<DIR> AlternateMode for Alternate1<DIR> {
    const SEL0: bool = true;
    const SEL1: bool = false;
}

impl<DIR> AlternateMode for Alternate2<DIR> {
    const SEL0: bool = false;
    const SEL1: bool = true;
}

impl<DIR> AlternateMode for Alternate3<DIR> {
    const SEL0: bool = true;
    const SEL1: bool = true;
}

// Only used as a bound inside the HAL, so keep it hidden
/// A pin whose type puts it in one of its three alternate functions: `Alternate1`, `Alternate2`
/// or `Alternate3` (PxSEL1/PxSEL0 = 01, 10 or 11, TI's primary, secondary and tertiary module
/// functions, SLAU445I Table 8-3, p. 314). For example, on the FR247x `Pin<P1, Pin1, Alternate2<Output>>`
/// is P1.1 in alternate function 2, the TA0.1 timer output (SLASEO7C Table 9-23, p. 65).
///
/// The type alone gives the pin's port and the PxSEL bits of its alternate function, so code
/// that only knows the type can find the pin. The methods switch the pin in hardware between
/// that one alternate function and plain GPIO, without changing the pin's type. PWM uses them
/// to disconnect its output while disabled, and LPMx.5 entry uses them to check whether XIN is
/// still connected to XT1, which LPM3.5 with the crystal running needs (SLAU445I 1.4.3.1, step 2,
/// p. 41).
#[doc(hidden)]
pub trait AlternatePin: ChangeSelectBits {
    /// The pin's port
    type Port: PortNum;
    /// The pin's bit within its port
    const MASK: u8;
    /// PxSEL0 bit value of the alternate function in the pin's type
    const SEL0: bool;
    /// PxSEL1 bit value of the alternate function in the pin's type
    const SEL1: bool;

    /// Whether the pin's PxSEL bits currently select the alternate function in its type (PxSEL0 and
    /// PxSEL1, SLAU445I Table 8-13, p. 335 and SLAU445I Table 8-14, p. 335)
    #[inline]
    fn function_matches_type() -> bool {
        let port = unsafe { Self::Port::steal() };
        let sel0 = port.pxsel0_rd() & Self::MASK != 0;
        let sel1 = port.pxsel1_rd() & Self::MASK != 0;
        sel0 == Self::SEL0 && sel1 == Self::SEL1
    }

    /// Set the pin's PxSEL bits to the alternate function in its type: PxSEL0 for `Alternate1`,
    /// PxSEL1 for `Alternate2`, both for `Alternate3` (SLAU445I Table 8-3, p. 314). Does nothing if
    /// they already are.
    #[inline]
    fn set_function_from_type(&mut self) {
        match (Self::SEL0, Self::SEL1) {
            (true, false) => self.set_sel0(),
            (false, true) => self.set_sel1(),
            // Alternate function 3 needs both PxSEL bits set. Setting one bit at a time would
            // briefly select function 1 or 2 in between, so flip both in one write through
            // PxSELC, as `to_alternate3()` does (SLAU445I 8.2.5, p. 314). A flip toggles (SLAU445I
            // Table 8-15, p. 336), so only flip while the pin is a GPIO.
            _ => if !Self::function_matches_type() { self.flip_selc() },
        }
    }

    /// Clear the pin's PxSEL bits, so it acts as a plain GPIO with its PxOUT and PxDIR
    /// settings (SLAU445I Table 8-3, p. 314). The pin's type stays the same. Does nothing if the
    /// bits are already clear.
    #[inline]
    fn set_function_gpio(&mut self) {
        match (Self::SEL0, Self::SEL1) {
            (true, false) => self.clear_sel0(),
            (false, true) => self.clear_sel1(),
            // Clear both PxSEL bits in one write, see `set_function_from_type`
            _ => if Self::function_matches_type() { self.flip_selc() },
        }
    }
}

impl<PORT: PortNum, PIN: PinNum, MODE: AlternateMode> AlternatePin for Pin<PORT, PIN, MODE> {
    type Port = PORT;
    const MASK: u8 = PIN::SET_MASK;
    const SEL0: bool = MODE::SEL0;
    const SEL1: bool = MODE::SEL1;
}

#[cfg(feature = "adcpctl")]
/// Typestate for GPIO ADC mode (for devices that use ADCPCTLx, SYSCFG2: SLAU445I Table 1-31, p. 82)
pub struct AdcMode<DIR>(PhantomData<DIR>);

// Sealing these traits takes a lot of work, and I'll never add any items in the future, so they
// are unsealed
/// Marker trait for all Pins that have alternate function 1 available (the "01" rows of the Port Px Pin
/// Functions tables in each data sheet, for example SLASEO7C Table 9-23, p. 65)
pub trait ToAlternate1 {}
/// Marker trait for all Pins that have alternate function 2 available (the "10" rows of the Port Px Pin
/// Functions tables in each data sheet, for example SLASEO7C Table 9-23, p. 65)
pub trait ToAlternate2 {}
/// Marker trait for all Pins that have alternate function 3 available (the "11" rows of the Port Px Pin
/// Functions tables in each data sheet, for example SLASEO7C Table 9-23, p. 65)
pub trait ToAlternate3 {}

#[cfg(feature = "adcpctl")]
/// Marker trait for all Pins that have ADC functionality via ADCPCTLx (bit x of SYSCFG2, SLAU445I
/// Table 1-31, p. 82)
pub trait ToAdcPctl: crate::adc::AdcPctlCapable {
    #[doc(hidden)]
    const SET_MASK: u16 = 1 << Self::ADCPCTLX;
    #[doc(hidden)]
    const CLR_MASK: u16 = !(1 << Self::ADCPCTLX);
}

// From GPIO, PxSEL1/PxSEL0 = 00 (SLAU445I Table 8-3, p. 314). Function 3 needs both bits changed, in
// one write through PxSELC (SLAU445I 8.2.5, p. 314).
impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> Pin<PORT, PIN, DIR>
where Self: ToAlternate1
{
    /// Convert pin to GPIO alternate function 1 (sets PxSEL0: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_alternate1(mut self) -> Pin<PORT, PIN, Alternate1<DIR>> {
        self.set_sel0();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> Pin<PORT, PIN, DIR>
where Self: ToAlternate2
{
    /// Convert pin to GPIO alternate function 2 (sets PxSEL1: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_alternate2(mut self) -> Pin<PORT, PIN, Alternate2<DIR>> {
        self.set_sel1();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> Pin<PORT, PIN, DIR>
where Self: ToAlternate3
{
    /// Convert pin to GPIO alternate function 3 (sets PxSEL0 and PxSEL1 in one PxSELC write: SLAU445I
    /// 8.2.5, p. 314)
    #[inline]
    pub fn to_alternate3(mut self) -> Pin<PORT, PIN, Alternate3<DIR>> {
        self.flip_selc();
        make_pin!()
    }
}

// sel0 = 1, sel1 = 0: the primary module function (SLAU445I Table 8-3, p. 314)
impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate1<DIR>> {
    /// Convert pin to GPIO function (clears PxSEL0: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_gpio(mut self) -> Pin<PORT, PIN, DIR> {
        self.clear_sel0();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate1<DIR>>
where Self: ToAlternate2
{
    /// Convert pin to alternate function 2 (01 to 10, both PxSEL bits flipped in one PxSELC write:
    /// SLAU445I 8.2.5, p. 314)
    #[inline]
    pub fn to_alternate2(mut self) -> Pin<PORT, PIN, Alternate2<DIR>> {
        self.flip_selc();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate1<DIR>>
where Self: ToAlternate3
{
    /// Convert pin to alternate function 3 (sets PxSEL1: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_alternate3(mut self) -> Pin<PORT, PIN, Alternate3<DIR>> {
        self.set_sel1();
        make_pin!()
    }
}

// sel0 = 0, sel1 = 1: the secondary module function (SLAU445I Table 8-3, p. 314)
impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate2<DIR>> {
    /// Convert pin to GPIO function (clears PxSEL1: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_gpio(mut self) -> Pin<PORT, PIN, DIR> {
        self.clear_sel1();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate2<DIR>>
where Self: ToAlternate1
{
    /// Convert pin to alternate function 1 (10 to 01, both PxSEL bits flipped in one PxSELC write:
    /// SLAU445I 8.2.5, p. 314)
    #[inline]
    pub fn to_alternate1(mut self) -> Pin<PORT, PIN, Alternate1<DIR>> {
        self.flip_selc();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate2<DIR>>
where Self: ToAlternate3
{
    /// Convert pin to alternate function 3 (sets PxSEL0: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_alternate3(mut self) -> Pin<PORT, PIN, Alternate3<DIR>> {
        self.set_sel0();
        make_pin!()
    }
}

// sel0 = 1, sel1 = 1: the tertiary module function (SLAU445I Table 8-3, p. 314)
impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate3<DIR>> {
    /// Convert pin to GPIO function (11 to 00, both PxSEL bits cleared in one PxSELC write: SLAU445I 8.2.5,
    /// p. 314)
    #[inline]
    pub fn to_gpio(mut self) -> Pin<PORT, PIN, DIR> {
        self.flip_selc();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate3<DIR>>
where Self: ToAlternate1
{
    /// Convert pin to alternate function 1 (clears PxSEL1: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_alternate1(mut self) -> Pin<PORT, PIN, Alternate1<DIR>> {
        self.clear_sel1();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate3<DIR>>
where Self: ToAlternate2
{
    /// Convert pin to alternate function 2 (clears PxSEL0: SLAU445I Table 8-3, p. 314)
    #[inline]
    pub fn to_alternate2(mut self) -> Pin<PORT, PIN, Alternate2<DIR>> {
        self.clear_sel0();
        make_pin!()
    }
}

#[cfg(feature = "adcpctl")]
impl<PORT: PortNum, PIN: PinNum, MODE> Pin<PORT, PIN, MODE>
where Self: ToAdcPctl
{
    /// Convert pin to ADC mode (ADCPCTL set, SYSCFG2: SLAU445I Table 1-31, p. 82). Setting the bit
    /// "disables both the output driver and input Schmitt trigger" of the pin (SLASE59F Table 6-17, p. 55;
    /// SLASEE4C Table 6-15, p. 58).
    #[inline]
    pub fn to_adc_mode(mut self) -> Pin<PORT, PIN, AdcMode<MODE>> {
        self.set_adcpctl(Self::SET_MASK);
        make_pin!()
    }
}

#[cfg(feature = "adcpctl")]
impl<PORT: PortNum, PIN: PinNum, MODE> Pin<PORT, PIN, AdcMode<MODE>>
where Self: ToAdcPctl
{
    /// Return pin to the mode it was in prior to ADCPCTL mode (clears ADCPCTLx in SYSCFG2: SLAU445I
    /// Table 1-31, p. 82)
    #[inline]
    pub fn from_adc_mode(mut self) -> Pin<PORT, PIN, MODE> {
        self.clr_adcpctl(Self::CLR_MASK);
        make_pin!()
    }
}

// Inputs read PxIN, outputs drive PxOUT (SLAU445I 8.2.1, p. 313 and SLAU445I 8.2.2, p. 313)
mod ehal1 {
    use super::*;
    use core::convert::Infallible;
    use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

    impl<PORT: PortNum, PIN: PinNum, DIR> ErrorType for Pin<PORT, PIN, DIR> {
        type Error = Infallible;
    }

    impl<PORT: PortNum, PIN: PinNum, PULL> InputPin for Pin<PORT, PIN, Input<PULL>> {
        // PxIN (SLAU445I Table 8-9, p. 334)
        #[inline]
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxin_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_low(&mut self) -> Result<bool, Self::Error> { self.is_high().map(|r| !r) }
    }

    impl<PORT: PortNum, PIN: PinNum> OutputPin for Pin<PORT, PIN, Output> {
        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn set_low(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_clear(PIN::CLR_MASK);
            Ok(())
        }

        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn set_high(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_set(PIN::SET_MASK);
            Ok(())
        }
    }

    impl<PORT: PortNum, PIN: PinNum> StatefulOutputPin for Pin<PORT, PIN, Output> {
        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxout_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_set_low(&mut self) -> Result<bool, Self::Error> { self.is_set_high().map(|r| !r) }

        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_toggle(PIN::SET_MASK);
            Ok(())
        }
    }
}

// Inputs read PxIN, outputs drive PxOUT (SLAU445I 8.2.1, p. 313 and SLAU445I 8.2.2, p. 313)
#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::digital::v2::{
        InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin,
    };

    impl<PORT: PortNum, PIN: PinNum, PULL> InputPin for Pin<PORT, PIN, Input<PULL>> {
        type Error = void::Void;

        // PxIN (SLAU445I Table 8-9, p. 334)
        #[inline]
        fn is_high(&self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxin_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_low(&self) -> Result<bool, Self::Error> { self.is_high().map(|r| !r) }
    }

    impl<PORT: PortNum, PIN: PinNum> OutputPin for Pin<PORT, PIN, Output> {
        type Error = void::Void;

        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn set_low(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_clear(PIN::CLR_MASK);
            Ok(())
        }

        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn set_high(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_set(PIN::SET_MASK);
            Ok(())
        }
    }

    impl<PORT: PortNum, PIN: PinNum> StatefulOutputPin for Pin<PORT, PIN, Output> {
        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxout_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_set_low(&self) -> Result<bool, Self::Error> { self.is_set_high().map(|r| !r) }
    }

    impl<PORT: PortNum, PIN: PinNum> ToggleableOutputPin for Pin<PORT, PIN, Output> {
        type Error = void::Void;

        // PxOUT (SLAU445I Table 8-10, p. 334)
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_toggle(PIN::SET_MASK);
            Ok(())
        }
    }
}
