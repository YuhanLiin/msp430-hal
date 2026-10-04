//! Timer cascading: one timer counts the periods of another. An LED on P1.0 toggles every 5 s.
//!
//! TA0 counts ACLK (REFO, 32.768 kHz) with a period of 1 s. The CCR2 output of TA0 clocks TA1, which
//! counts those periods: the LED toggles every 5 periods, so every 5 s. TA1 could count up to 65536 s
//! like this, about 18 hours. On the MSP430FR25x2, TA1 can count the periods of TA0.
//! (REFO: SLASEE4C Table 5-7, p. 27. TA0's CCR2 output is the TASSEL = 11 clock of TA1: SLASEE4C
//! Figure 6-2, p. 54. Both timers are 16 bits: SLASEE4C 6.10.8, p. 54. P1.0 is a GPIO output with
//! P1SELx = 00 and P1DIR = 1: SLASEE4C Table 6-15, p. 58. No board document covers the LED: there is
//! none for the MSP430FR25x2.)
//!
//! How to test (an LED and a resistor):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example.
//! 3. Expected: the LED is on for 5 s, then off for 5 s.
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

/// ACLK cycles per TA0 period: 1 s
const ACLK_CYCLES: u16 = 32_768;
/// TA0 periods per LED toggle
const PERIODS: u16 = 5;

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

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let ta0 = TimerParts3::new(periph.ta0, TimerConfig::aclk(&aclk));
    // CCR2 of TA0 now pulses once per period, to clock TA1 (SLASEE4C Figure 6-2, p. 54)
    let ta0_periods = ta0.subtimer2.into_cascade_output();
    let mut ta0 = ta0.timer;
    let mut ta1 = TimerParts3::new(periph.ta1, TimerConfig::cascade(&ta0_periods)).timer;

    // The timers count from 0 up to and including the given value
    // (Up mode, SLAU445I 13.2.3.1, p. 371: "The number of timer counts in the period is TAxCCR0 + 1")
    ta1.start(PERIODS - 1);
    ta0.start(ACLK_CYCLES - 1);

    loop {
        block!(ta1.wait()).unwrap();
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
