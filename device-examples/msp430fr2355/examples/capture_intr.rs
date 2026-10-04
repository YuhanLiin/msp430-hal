//! A capture input handled in its interrupt: each falling edge on P1.6 toggles LED1.
//!
//! P1.6 is TB0.CCI1A, input A of TB0's CCR1, which captures each falling edge. Its interrupt reads
//! TB0IV, which gives the capture, and toggles LED1. The LaunchPad's buttons, S1 on P4.1 and S2 on
//! P2.3, aren't capture inputs, so the edges come from the function generator or a jumper wire. The
//! example also shows how to write panic-free code, with `panic_never` in release builds.
//! (TB0.CCI1A on P1.6: SLASEC4D Table 6-16, p. 73. TB0IV: SLAU445I 14.2.6.2, p. 405. Capture inputs:
//! SLASEC4D Tables 6-16 to 6-19, p. 73 to p. 75. S1, S2, and LED1 on P1.0, red: SLAU680 Figure 18,
//! p. 26.)
//!
//! How to test (function generator, or a jumper wire):
//! 1. Generator: square wave, 1 Hz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load High-Z. Check the
//!    levels on the scope before connecting: a negative or >3.6 V signal can damage the pin. Connect it
//!    to P1.6 (J1 pin 3), its ground to GND (J3 pin 22).
//! 2. Flash this example. Expected: LED1 is on for 1 s, then off for 1 s.
//! 3. Without the generator: put a jumper wire on P1.6 (J1 pin 3), and touch its free end to 3.3 V (J1
//!    pin 1), then to GND (J2 pin 20): LED1 toggles. P1.6 has no pull resistor, so it floats between
//!    touches, and the contact bounces: LED1 can toggle more than once, or not at all.
//! (Header pins: SLAU680 Figure 10, p. 15.)
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::UnsafeCell;
use critical_section::with;
use embedded_hal::digital::StatefulOutputPin;
use msp430::interrupt::{enable, Mutex};
use msp430_rt::entry;
use msp430fr2355::interrupt;
use msp430_hal::{
    capture::{
        CapCmp, CapTrigger, Capture, CaptureParts3, CaptureVector, TBxIV, TimerConfig, CCR1,
    },
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    gpio::*,
    pmm::Pmm,
    watchdog::Wdt,
};

#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

// We use UnsafeCell as a panic-free version of RefCell. If you aren't using `panic_never` then RefCell is more ergonomic.
static CAPTURE: Mutex<UnsafeCell<Option<Capture<msp430fr2355::Tb0, CCR1>>>> =
    Mutex::new(UnsafeCell::new(None));
static VECTOR: Mutex<UnsafeCell<Option<TBxIV<msp430fr2355::Tb0>>>> =
    Mutex::new(UnsafeCell::new(None));
static RED_LED: Mutex<UnsafeCell<Option<Pin<P1, Pin0, Output>>>> =
    Mutex::new(UnsafeCell::new(None));

#[entry]
fn main() -> ! {
    let Some(periph) = msp430fr2355::Peripherals::take() else { loop {} };
    let mut fram = Fram::new(periph.frctl);
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let red_led = p1.pin0;

    with(|cs| unsafe { *RED_LED.borrow(cs).get() = Some(red_led) });

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let captures = CaptureParts3::config(periph.tb0, TimerConfig::aclk(&aclk))
        .config_cap1_input_A(p1.pin6.to_alternate2()) // TB0.CCI1A, P1SELx = 10 (SLASEC4D Table 6-63, p. 96)
        .config_cap1_trigger(CapTrigger::FallingEdge)
        .commit();
    let mut capture = captures.cap1;
    let vectors = captures.tbxiv;

    setup_capture(&mut capture);
    with(|cs| {
        unsafe { *CAPTURE.borrow(cs).get() = Some(capture) }
        unsafe { *VECTOR.borrow(cs).get() = Some(vectors) }
    });
    unsafe { enable() };

    loop {}
}

fn setup_capture<T: CapCmp<C>, C>(capture: &mut Capture<T, C>) {
    capture.enable_interrupts();
}

// The vector at FFF6h shared by TB0CCR1, TB0CCR2 and TB0IFG, decoded with TB0IV (SLASEC4D Table 6-2,
// p. 63)
#[interrupt]
fn TIMER0_B1() {
    with(|cs| {
        let Some(vector) = unsafe { &mut *VECTOR.borrow(cs).get() }.as_mut() else { return; };
        let Some(capture) = unsafe { &mut *CAPTURE.borrow(cs).get() }.as_mut() else { return; };
        let Some(led) = unsafe { &mut *RED_LED.borrow(cs).get() }.as_mut() else { return; };

        if let CaptureVector::Capture1(cap) = vector.interrupt_vector() {
            if cap.interrupt_capture(capture).is_ok() {
                led.toggle().unwrap();
            }
        };
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
