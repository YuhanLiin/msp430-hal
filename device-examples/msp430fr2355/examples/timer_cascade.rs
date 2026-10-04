//! Timer cascading: one timer counts the periods of another.
//!
//! TB0 runs from ACLK (REFO, 32.768 kHz) with a period of 1 s, and red LED1 toggles every period.
//! The CCR2 output of TB0 clocks TB1, which counts those periods: green LED2 toggles every
//! 5 periods, so every 5 s. TB1 could count up to 65536 s like this, about 18 hours.
//! (REFO: SLASEC4D Table 5-7, p. 40. LED1 is on P1.0 and LED2 on P6.6: SLAU680 Figure 18, p. 26.)
//!
//! On the MSP430FR2x5x, TB1 can count the periods of TB0: TB1's INCLK input is the "Timer0_B3 CCR2B
//! output" (SLASEC4D Table 6-17, p. 74).
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
use panic_msp430 as _;

/// ACLK cycles per TB0 period: 1 s (ACLK is REFO, 32768 Hz: SLASEC4D Table 5-7, p. 40)
const ACLK_CYCLES: u16 = 32_768;
/// TB0 periods per LED2 toggle
const PERIODS: u16 = 5;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // LED1, red, on P1.0 and LED2, green, on P6.6 (SLAU680 Figure 18, p. 26)
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let p6 = Batch::new(periph.p6)
        .config_pin6(|p| p.to_output())
        .split(&pmm);
    let mut red_led = p1.pin0;
    let mut green_led = p6.pin6;

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let tb0 = TimerParts3::new(periph.tb0, TimerConfig::aclk(&aclk));
    // CCR2 of TB0 now pulses once per period, to clock TB1 (TB1's INCLK is the "Timer0_B3 CCR2B output":
    // SLASEC4D Table 6-17, p. 74)
    let tb0_periods = tb0.subtimer2.into_cascade_output();
    let mut tb0 = tb0.timer;
    let mut tb1 = TimerParts3::new(periph.tb1, TimerConfig::cascade(&tb0_periods)).timer;

    // The timers count from 0 up to and including the given value
    // ("The number of timer counts in the period is TBxCL0 + 1": SLAU445I 14.2.3.1, p. 394)
    tb1.start(PERIODS - 1);
    tb0.start(ACLK_CYCLES - 1);

    loop {
        if tb0.wait().is_ok() {
            red_led.toggle().unwrap();
        }
        if tb1.wait().is_ok() {
            green_led.toggle().unwrap();
        }
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
