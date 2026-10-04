//! An LED on P1.0 blinks, on for 0.5 s and off for 0.5 s: a GPIO output, timed by the delay of the clock
//! system.
//!
//! The program toggles P1.0 and waits 500 ms with the delay that `ClockConfig::freeze()` returns, which
//! counts cycles of MCLK, about 8 MHz from the DCO.
//! (P1.0 is a GPIO output, P1SELx = 00 and P1DIR = 1: SLASEE4C Table 6-15, p. 58. No board document
//! covers the LED: there is none for the MSP430FR25x2.)
//!
//! How to test (an LED and a resistor):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example.
//! 3. Expected: the LED blinks, on for 0.5 s and off for 0.5 s.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog. The watchdog runs from every PUC and must be halted, here
    // with WDTHOLD (SLAU445I 12.2.2, p. 363; SLAU445I Table 12-2, p. 366).
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO. Pmm::new clears LOCKLPM5, so the pins take on their configuration
    // (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut p1_0 = port1.pin0.to_output();

    // Configure clocks to get accurate delay timing
    let mut fram = Fram::new(periph.frctl);
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    loop {
        // `toggle()` returns a `Result` because of embedded_hal, but the result is always `Ok` with MSP430 GPIO.
        // Rust complains about unused Results, so we 'use' the Result by calling .ok()
        p1_0.toggle().ok();
        delay.delay_ms(500);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
