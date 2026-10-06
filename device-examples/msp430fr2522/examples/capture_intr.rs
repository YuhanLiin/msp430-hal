#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// This example also demonstrates how to write panic-free code using panic_never.

use core::cell::UnsafeCell;
use critical_section::with;
use embedded_hal::digital::StatefulOutputPin;
use msp430::interrupt::{enable, Mutex};
use msp430_rt::entry;
use msp430fr25x2::interrupt;
use msp430_hal::{
    capture::{
        CapCmp, CapTrigger, Capture, CaptureParts3, CaptureVector, TBxIV, TimerConfig, CCR2,
    },
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Batch, *},
    pin_mapping::*,
    pmm::Pmm,
    watchdog::Wdt,
};

#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

// We use UnsafeCell as a panic-free version of RefCell. If you aren't using `panic_never` then RefCell is more ergonomic.
static CAPTURE: Mutex<UnsafeCell<Option<Capture<msp430fr25x2::Ta0, CCR2>>>> =
    Mutex::new(UnsafeCell::new(None));
static VECTOR: Mutex<UnsafeCell<Option<TBxIV<msp430fr25x2::Ta0, DefaultMapping>>>> =
    Mutex::new(UnsafeCell::new(None));
static RED_LED: Mutex<UnsafeCell<Option<Pin<P1, Pin0, Output>>>> =
    Mutex::new(UnsafeCell::new(None));

// Connect push button input to P1.5, TA0.CCI2A, the capture input set up below (P1.6 is TA0CLK, no
// capture input: SLASEE4C Table 6-15, p. 58; SLASEE4C Figure 6-2, p. 54). When button is pressed, red
// LED should toggle. No debouncing, so sometimes inputs are missed. No board document covers the button
// or the LED (P1.0): there is none for the MSP430FR25x2. P1.0 is a GPIO output, P1SELx = 00 and P1DIR = 1
// (SLASEE4C Table 6-15, p. 58).
#[entry]
fn main() -> ! {
    let Some(periph) = msp430fr25x2::Peripherals::take() else {
        loop {}
    };
    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let red_led = p1.pin0;

    with(|cs| unsafe { *RED_LED.borrow(cs).get() = Some(red_led) });

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk() // ACLK from REFO, 32768 Hz (SLASEE4C Table 5-7, p. 27)
        .freeze(&mut fram);

    // P1.5 as TA0.CCI2A, capture input A of CCR2: P1SELx = 10 with P1DIR = 0 (SLASEE4C Table 6-15, p. 58;
    // SLASEE4C Figure 6-2, p. 54)
    let captures = CaptureParts3::config(periph.ta0, TimerConfig::aclk(&aclk))
        .config_cap2_input_A(p1.pin5.to_alternate2())
        .config_cap2_trigger(CapTrigger::FallingEdge)
        .commit();
    let mut capture = captures.cap2;
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
    // CCIE = 1: the CCR's CCIFG requests an interrupt (SLAU445I Table 13-6, p. 386)
    capture.enable_interrupts();
}

// Timer0_A3 CCR1, CCR2 and overflow interrupt, TA0IV (SLASEE4C Table 6-2, p. 46: vector FFF6h)
#[interrupt]
fn TIMER0_A1() {
    with(|cs| {
        let Some(vector) = unsafe { &mut *VECTOR.borrow(cs).get() }.as_mut() else {
            return;
        };
        let Some(capture) = unsafe { &mut *CAPTURE.borrow(cs).get() }.as_mut() else {
            return;
        };
        let Some(led) = unsafe { &mut *RED_LED.borrow(cs).get() }.as_mut() else {
            return;
        };

        // Reading TA0IV resets the highest-pending interrupt flag (SLAU445I 13.2.6.2, p. 380)
        if let CaptureVector::Capture2(cap) = vector.interrupt_vector() {
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
