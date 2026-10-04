//! A timer clocked from the VLO, the internal very-low-power oscillator: an LED on P1.0 toggles every
//! 10000 VLO cycles, about once a second.
//!
//! The VLO needs no clock configuration: it starts when the timer requests it. On the MSP430FR25x2, only
//! TA0 can be clocked from the VLO. The VLO runs at about 10 kHz but is only accurate to ±50 %, so a
//! toggle can come anywhere from every 0.7 s to every 2 s.
//! (SLAU445I 3.2.2, p. 102: VLOCLK is active when "At least one peripheral requests VLO as clock
//! source". VLOCLK "10 kHz ±50%", TASSEL = 11b for TA0 and not available for TA1: SLASEE4C Table 6-8,
//! p. 49; SLASEE4C Figure 6-2, p. 54. P1.0 is a GPIO output with P1SELx = 00 and P1DIR = 1: SLASEE4C
//! Table 6-15, p. 58. No board document covers the LED: there is none for the MSP430FR25x2.)
//!
//! How to test (an LED and a resistor, optionally the scope):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example.
//! 3. Expected: the LED toggles about once a second.
//! 4. To measure the VLO, probe P1.0: the VLO frequency is 20000 divided by the period of the signal.
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
