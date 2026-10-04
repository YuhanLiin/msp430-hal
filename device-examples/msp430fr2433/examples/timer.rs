//! A timer and a sub-timer: LED1 blinks, on for 0.5 s and off for 0.5 s, timed by TA2 and its CCR1.
//!
//! TA2 is one of the two Timer_A modules with only two capture/compare registers. TA2 and TA3 aren't
//! connected to any pins, so they have no PWM outputs or capture inputs, and they count ACLK or SMCLK.
//! Here TA2 counts ACLK from REFO, 32768 Hz, from 0 to 32767 in up mode: a period of 1 s. LED1 goes off
//! each time the timer wraps around to 0. CCR1 is a sub-timer in compare mode: it fires at count 16384,
//! halfway through the period, and LED1 goes on.
//! (TA2 and TA3: SLASE59F 6.10.8, p. 51; SLASE59F Table 6-13, p. 51; SLASE59F Table 6-7, p. 46. REFO:
//! SLASE59F Table 5-7, p. 25. Up mode: SLAU445I 13.2.3.1, p. 371. Compare mode: SLAU445I 13.2.4.2,
//! p. 376. LED1 on P1.0 is red: SLAU739 Figure 18, p. 23.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 blinks once a second, on for 0.5 s and off for 0.5 s.
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

/// ACLK cycles per period: 1 s, with ACLK from REFO at 32768 Hz (SLASE59F Table 5-7, p. 25)
const ACLK_CYCLES: u16 = 32_768;

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();
    // Hold the watchdog (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2,
    // p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = p1.pin0;

    let mut fram = Fram::new(periph.frctl);
    // MCLK = SMCLK = about 1 MHz: DCORSEL = 000b with the FLL locked to REFO (SLAU445I Table 3-5, p. 114;
    // SLAU445I 3.2.5, p. 104), DIVM and DIVS /1 (SLAU445I Table 3-9, p. 118). ACLK = REFO: SELA = 01b
    // (SLAU445I Table 3-8, p. 117).
    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // TA2 counts ACLK: TASSEL = 01b (SLAU445I Table 13-4, p. 384; SLASE59F Table 6-7, p. 46). The main
    // timer is CCR0, the sub-timer CCR1 (SLASE59F Table 6-13, p. 51).
    let parts = TimerParts2::new(periph.ta2, TimerConfig::aclk(&aclk));
    let mut timer = parts.timer;
    let mut subtimer = parts.subtimer1;

    // The timer counts from 0 up to and including the given value (SLAU445I 13.2.3.1, p. 371: "The
    // number of timer counts in the period is TAxCCR0 + 1.")
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
