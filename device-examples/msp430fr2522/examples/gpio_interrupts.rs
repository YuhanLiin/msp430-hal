#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use critical_section::with;
use msp430fr25x2::interrupt;

use core::cell::RefCell;
use embedded_hal::digital::*;
use msp430::interrupt::{enable as enable_int, Mutex};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Batch, GpioVector, Output, Pin, Pin0, PxIV, P1, P2},
    pmm::Pmm,
    watchdog::{Wdt, WdtClkPeriods},
};
use nb::block;
use panic_msp430 as _;

static RED_LED: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));

// Red LED should blink 1 second on, 1 second off
// A press of the P2.3 button toggles the red LED too: the main loop sets P2.6's interrupt flag, whose
// handler toggles it
// No board document covers the LEDs (the red one on P1.0 here) or the button: there is none for the
// MSP430FR25x2. P2.3 and P2.6 only exist on the 20-pin RHL package (SLASEE4C Table 4-2, p. 14).
// All three pins are GPIO, PxSELx = 00 (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60): P1.0 an
// output, P2.3 an input with its pullup and P2.6 one with its pulldown (SLAU445I Table 8-1, p. 313).
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363), then use it as an
    // interval timer (WDTTMSEL = 1: SLAU445I Table 12-2, p. 366)
    let mut wdt = Wdt::constrain(periph.wdt_a).to_interval();

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1) // 32 kHz MCLK (REFO, 32768 Hz: SLASEE4C Table 5-7, p. 27)
        .smclk_on(SmclkDiv::_2) // 16 kHz SMCLK
        .aclk_refoclk()
        .freeze(&mut Fram::new(periph.frctl));

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);

    let red_led = p1.pin0.to_output();
    // Onboard button with interrupt disabled
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually. (P1 and P2 pins can interrupt:
    // SLASEE4C 6.10.3, p. 51)
    let mut pin = p2.pin6.pulldown();
    let p2iv = p2.pxiv;

    with(|cs| RED_LED.borrow_ref_mut(cs).replace(red_led));
    with(|cs| P2IV.borrow_ref_mut(cs).replace(p2iv));

    // ACLK / 2^15, WDTIS = 100b: "1 s at 32.768 kHz" (SLAU445I Table 12-2, p. 366)
    wdt.set_aclk(&aclk)
        .enable_interrupts()
        .set_interval_and_start(WdtClkPeriods::_32k);
    // P2IES = 0 sets P2IFG on a rising edge, 1 on a falling edge (SLAU445I 8.2.6.2, p. 316); P2IE lets the
    // flag request the interrupt (SLAU445I 8.2.6.3, p. 316)
    pin.select_rising_edge_trigger().enable_interrupts();
    button.select_falling_edge_trigger();

    unsafe { enable_int() };

    loop {
        // Poll the button's P2IFG; software can also set a PxIFG flag to request the interrupt
        // (SLAU445I 8.2.6, p. 315: "a software-initiated interrupt")
        block!(button.wait_for_ifg()).ok();
        pin.set_ifg();
    }
}

// Port 2 interrupt, P2IV (SLASEE4C Table 6-2, p. 46: P2IFG.0 to P2IFG.6, vector FFE4h)
#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut red_led) = *RED_LED.borrow_ref_mut(cs) else {
            return;
        };
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {
            return;
        };

        // Reading P2IV clears the highest-priority pending flag (SLAU445I 8.2.6, p. 315)
        if let GpioVector::Pin6Isr = p2iv.get_interrupt_vector() {
            red_led.toggle().ok();
        }
    });
}

// Watchdog timer interval mode interrupt (SLASEE4C Table 6-2, p. 46: WDTIFG, vector FFEEh)
#[interrupt]
fn WDT() {
    with(|cs| {
        RED_LED.borrow_ref_mut(cs).as_mut().map(|red_led| {
            red_led.toggle().ok();
        })
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
