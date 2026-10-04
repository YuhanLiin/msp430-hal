#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// This example also demonstrates how to write panic-free code using panic_never.

use core::cell::UnsafeCell;
use critical_section::with;
use embedded_hal::digital::StatefulOutputPin;
use msp430::interrupt::{enable, Mutex};
use msp430_rt::entry;
use msp430fr247x::interrupt;
use msp430_hal::{
    capture::{
        CCR1, CapCmp, CapTrigger, Capture, CaptureParts3, CaptureVector, TBxIV, TimerConfig
    }, clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::{Batch, *}, pin_mapping::*, pmm::Pmm, watchdog::Wdt
};

#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

// We use UnsafeCell as a panic-free version of RefCell. If you aren't using `panic_never` then RefCell is more ergonomic.
static CAPTURE: Mutex<UnsafeCell<Option<Capture<msp430fr247x::Ta2, CCR1>>>> =
    Mutex::new(UnsafeCell::new(None));
static VECTOR: Mutex<UnsafeCell<Option<TBxIV<msp430fr247x::Ta2, DefaultMapping>>>> =
    Mutex::new(UnsafeCell::new(None));
static LED1: Mutex<UnsafeCell<Option<Pin<P1, Pin0, Output>>>> =
    Mutex::new(UnsafeCell::new(None));

// Connect push button input to P3.3, J4 pin 35 (SLAU802 Figure 10, p. 13). When button is pressed,
// LED1 (P1.0), which is green, should toggle (SLAU802 Figure 19, p. 25). No debouncing,
// so sometimes inputs are missed.
#[entry]
fn main() -> ! {
    let Some(periph) = msp430fr247x::Peripherals::take() else { loop {} };
    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let p3 = Batch::new(periph.p3).split(&pmm);
    let led1 = p1.pin0;

    with(|cs| unsafe { *LED1.borrow(cs).get() = Some(led1) });

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range, ACLK from the VLO (SELMS, SELA: SLAU445I
    // Table 3-8, p. 117). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I Table 3-1, p. 98 lists
    // that for the enhanced clock system only, and the HAL follows the data sheet.
    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // TA2 CCR1 input A (CCI1A) is P3.3 with P3SEL = 01 and P3DIR = 0 (SLASEO7C Table 9-14, p. 58;
    // SLASEO7C Table 9-25, p. 67). ACLK is TASSEL = 01b (SLASEO7C Table 9-8, p. 50). Capture on the
    // falling edge: CM = 10b, CCIS = 00b (SLAU445I Table 13-6, p. 386).
    let captures = CaptureParts3::config(periph.ta2, TimerConfig::aclk(&aclk))
        .config_cap1_input_A(p3.pin3.to_alternate1())
        .config_cap1_trigger(CapTrigger::FallingEdge)
        .commit();
    let mut capture = captures.cap1;
    let vectors = captures.tbxiv;

    setup_capture(&mut capture);
    with(|cs| {
        unsafe { *CAPTURE.borrow(cs).get() = Some(capture) }
        unsafe { *VECTOR.borrow(cs).get() = Some(vectors) }
    });
    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { enable() };

    loop {}
}

/// Enable the capture interrupt (CCIE: SLAU445I Table 13-6, p. 386)
fn setup_capture<T: CapCmp<C>, C>(capture: &mut Capture<T, C>) {
    capture.enable_interrupts();
}

// The TA2 CCR1, CCR2 and overflow interrupt vector (FFEEh: SLASEO7C Table 9-2, p. 46). Reading TA2IV
// gives the highest pending source and clears its flag (SLAU445I 13.2.6.2, p. 380; SLAU445I
// Table 13-8, p. 388).
#[interrupt]
fn TIMER2_A1() {
    with(|cs| {
        let Some(vector) = unsafe { &mut *VECTOR.borrow(cs).get() }.as_mut() else { return; };
        let Some(capture) = unsafe { &mut *CAPTURE.borrow(cs).get() }.as_mut() else { return; };
        let Some(led) = unsafe { &mut *LED1.borrow(cs).get() }.as_mut() else { return; };

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
