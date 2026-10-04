//! GPIO interrupts and the watchdog's interval interrupt: LED1 toggles every 3.3 s or so, and each press
//! of S2 toggles both the blue part of LED2 and LED1.
//!
//! The watchdog, as an interval timer, interrupts every 2^15 ACLK cycles, about 3.3 s with ACLK from the
//! VLO, and its interrupt toggles LED1. The main loop polls the interrupt flag of S2 (P2IFG.3, with its
//! interrupt disabled): at each press it toggles the blue part of LED2 and sets the flag of P2.7 in
//! software, which requests the port 2 interrupt, and that toggles LED1.
//! (WDTIS = 100b: SLAU445I Table 12-2, p. 366. The VLO runs at about 10 kHz, within ±50 %: SLASEO7C
//! Table 9-8, p. 50. Software can set PxIFG: SLAU445I 8.2.6, p. 315. LED1 on P1.0 is green, the blue
//! part of LED2 is P4.7, and S2 is P2.3: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 toggles every 3.3 s or so (anything from 2.2 s to 6.6 s is right).
//! 3. Press S2: the blue part of LED2 toggles, and so does LED1.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use critical_section::with;
use msp430fr247x::interrupt;

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

static LED1: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    // Stop the watchdog, then use it as an interval timer (WDTHOLD, WDTTMSEL: SLAU445I Table 12-2,
    // p. 366; interval timer mode: SLAU445I 12.2.3, p. 363)
    let mut wdt = Wdt::constrain(periph.wdt_a).to_interval();

    // REFO runs at 32.768 kHz (SLASEO7C 8.12.3.4, p. 30)
    // (MCLK from REFOCLK, SELMS = 001b, and ACLK from the VLO: SLAU445I Table 3-8, p. 117; SMCLK =
    // MCLK / 2, DIVS = 01b: SLAU445I Table 3-9, p. 118. ACLK from the VLO: SLASEO7C 9.10.2, p. 49;
    // SLAU445I Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the
    // data sheet.)
    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1) // 32 kHz MCLK
        .smclk_on(SmclkDiv::_2) // 16 kHz SMCLK
        .aclk_vloclk()
        .freeze(&mut Fram::new(periph.frctl));

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    // P2.3 with its pullup and P2.7 with its pulldown (PxREN = 1, PxOUT = 1 or 0: SLAU445I Table 8-1,
    // p. 313)
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);

    let led1 = p1.pin0.to_output();
    // Onboard button with interrupt disabled (S2: SLAU802 Figure 19, p. 25)
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually.
    // (Software can set PxIFG to request the interrupt: SLAU445I 8.2.6, p. 315)
    let mut pin = p2.pin7.pulldown();
    // P4.7 drives the blue part of LED2 (SLAU802 Figure 19, p. 25)
    let mut led2_blue = p4.pin7.to_output();
    let p2iv = p2.pxiv;

    with(|cs| LED1.borrow_ref_mut(cs).replace(led1));
    with(|cs| P2IV.borrow_ref_mut(cs).replace(p2iv));

    // ACLK is WDTSSEL = 01b (SLAU445I Table 12-2, p. 366); WDTIE enables the interval interrupt
    // (SLAU445I Table 1-9, p. 62). PxIES = 0 sets PxIFG on a low-to-high transition, PxIES = 1 on a
    // high-to-low one (SLAU445I Table 8-16, p. 336).
    wdt.set_aclk(&aclk)
        .enable_interrupts()
        .set_interval_and_start(WdtClkPeriods::_32k);
    pin.select_rising_edge_trigger().enable_interrupts();
    button.select_falling_edge_trigger();

    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { enable_int() };

    // P2IFG.3 is set on the selected edge even with its interrupt disabled, so it can be polled
    // (SLAU445I 8.2.6, p. 315)
    loop {
        block!(button.wait_for_ifg()).ok();
        led2_blue.toggle().ok();
        pin.set_ifg();
    }
}

// The port 2 interrupt vector, P2IFG.0 to P2IFG.7 through P2IV (FFD4h: SLASEO7C Table 9-2, p. 47).
// Reading P2IV "automatically resets the highest pending interrupt flag" (SLAU445I 8.2.6, p. 315).
#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut led1) = *LED1.borrow_ref_mut(cs) else { return; };
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else { return; };

        if let GpioVector::Pin7Isr = p2iv.get_interrupt_vector() {
            led1.toggle().ok();
        }
    });
}

// The watchdog interval mode vector, WDTIFG (FFE2h: SLASEO7C Table 9-2, p. 46). In interval mode
// "WDTIFG is reset automatically by servicing the interrupt" (SLAU445I Table 1-10, p. 63).
#[interrupt]
fn WDT() {
    with(|cs| {
        LED1.borrow_ref_mut(cs).as_mut().map(|led1| {
            led1.toggle().ok();
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
