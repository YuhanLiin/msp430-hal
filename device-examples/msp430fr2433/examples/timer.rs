//! A timer on TA2, one of the two Timer_A modules with only two capture/compare registers. TA2 and TA3
//! aren't connected to any pins, so they have no PWM output or capture pins, and they are clocked from
//! ACLK or SMCLK.
//! The red LED on P1.0 (LED1) turns on when sub-timer 1 fires halfway through each 1 s period, and off
//! when the main timer wraps around: 0.5 s off, 0.5 s on.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    timer::{TimerConfig, TimerParts2},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

/// ACLK cycles per period: 1 s, with ACLK from REFO at 32768 Hz
const ACLK_CYCLES: u16 = 32_768;

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.watchdog_timer);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = p1.pin0;

    let mut fram = Fram::new(periph.fram);
    // MCLK = SMCLK = about 1 MHz: DCORSEL = 000b with the FLL locked to REFO, DIVM and DIVS /1. ACLK =
    // REFO: SELA = 01b.
    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // TA2 counts ACLK: TASSEL = 01b. The main timer is CCR0, the sub-timer CCR1.
    let parts = TimerParts2::new(periph.timer_2_a2, TimerConfig::aclk(&aclk));
    let mut timer = parts.timer;
    let mut subtimer = parts.subtimer1;

    // The timer counts from 0 up to and including the given value ("The number of timer counts in the
    // period is TAxCCR0 + 1.")
    timer.start(ACLK_CYCLES - 1);
    subtimer.set_count(ACLK_CYCLES / 2);

    loop {
        block!(subtimer.wait()).unwrap();
        red_led.set_high().unwrap();
        block!(timer.wait()).unwrap();
        red_led.set_low().unwrap();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
