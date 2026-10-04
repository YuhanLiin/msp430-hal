//! A timer clocked from the VLO, the internal very-low-power oscillator.
//!
//! The red LED on P1.0 toggles every 10000 VLO cycles. The VLO runs at about 10 kHz but is only
//! accurate to ±50 % (data sheet: SLASEE4C Table 6-8, p. 49, "10 kHz ±50%"), so that is anywhere
//! from 0.7 s to 2 s. With a scope on P1.0, the VLO frequency is 20000 divided by the period of the
//! LED signal. No board document covers the LED: there is none for the MSP430FR25x2. P1.0 is a GPIO
//! output, P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).
//!
//! The VLO needs no clock configuration: it starts when the timer requests it
//! (SLAU445I 3.2.2, p. 102: VLOCLK is active when "At least one peripheral requests VLO as clock
//! source"). On the MSP430FR25x2, only TA0 can be clocked from the VLO (SLASEE4C Table 6-8, p. 49:
//! VLOCLK is TASSEL = 11b for TA0 and not available for TA1; SLASEE4C Figure 6-2, p. 54).
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    timer::{TimerConfig, TimerParts3},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

/// VLO cycles per LED toggle
const VLO_CYCLES: u16 = 10_000;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = p1.pin0;

    let (_smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    let mut timer = TimerParts3::new(periph.ta0, TimerConfig::vloclk()).timer;
    // The timer counts from 0 up to and including the given value
    // (Up mode, SLAU445I 13.2.3.1, p. 371: "The number of timer counts in the period is TAxCCR0 + 1")
    timer.start(VLO_CYCLES - 1);

    loop {
        block!(timer.wait()).unwrap();
        red_led.toggle().unwrap();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
