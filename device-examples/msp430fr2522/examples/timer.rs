//! A timer and a sub-timer: an LED on P1.0 blinks, on for 0.5 s and off for 0.5 s, timed by TA0 and its
//! CCR2.
//!
//! TA0 counts ACLK from REFO, 32768 Hz, divided by 8 and by 4: 1024 Hz. In up mode it counts from 0 to
//! 1024, a period of 1 s, and the LED goes off each time the timer wraps around to 0. CCR2 is a sub-timer
//! in compare mode: it fires at count 512, halfway through the period, and the LED goes on.
//! (REFO: SLASEE4C Table 5-7, p. 27. ID and TAIDEX: SLAU445I 13.2.1.1, p. 370. Up mode: SLAU445I
//! 13.2.3.1, p. 371. Compare mode: SLAU445I 13.2.4.2, p. 376. P1.0 is a GPIO output with P1SELx = 00
//! and P1DIR = 1: SLASEE4C Table 6-15, p. 58. No board document covers the LED: there is none for the
//! MSP430FR25x2.)
//!
//! How to test (an LED and a resistor):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example.
//! 3. Expected: the LED blinks once a second, on for 0.5 s and off for 0.5 s.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::PinMap,
    pmm::Pmm,
    timer::{CapCmp, SubTimer, Timer, TimerConfig, TimerDiv, TimerExDiv, TimerParts3, TimerPeriph},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

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
    let mut p1_0 = p1.pin0;

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let parts = TimerParts3::new(
        periph.ta0,
        // ACLK (REFO, 32768 Hz) / 32 = 1024 Hz
        // (REFO: SLASEE4C Table 5-7, p. 27. ID divides by 8 and TAIDEX by 4: SLAU445I 13.2.1.1, p. 370)
        TimerConfig::aclk(&aclk).clk_div(TimerDiv::_8, TimerExDiv::_4),
    );
    let mut timer = parts.timer;
    let mut subtimer = parts.subtimer2;

    set_time(&mut timer, &mut subtimer, 512);
    loop {
        block!(subtimer.wait()).unwrap();
        p1_0.set_high().unwrap();
        // first 0.5 s of timer countdown expires while subtimer expires, so this should only block
        // for 0.5 s
        block!(timer.wait()).unwrap();
        p1_0.set_low().unwrap();
    }
}

fn set_time<T: TimerPeriph<M> + CapCmp<C>, C, M: PinMap>(
    timer: &mut Timer<T, M>,
    subtimer: &mut SubTimer<T, C>,
    delay: u16,
) {
    timer.start(delay + delay);
    subtimer.set_count(delay);
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
