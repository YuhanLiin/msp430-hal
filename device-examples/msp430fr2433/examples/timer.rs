//! A timer on TA2, one of the two Timer_A modules with only two capture/compare registers. TA2
//! and TA3 aren't connected to any pins, so they have no PWM output or capture pins, and they are
//! clocked from ACLK or SMCLK (SLASE59F 6.10.8, p. 51; SLASE59F Table 6-13, p. 51; SLASE59F
//! Table 6-7, p. 46).
//!
//! The red LED on P1.0 (LED1, SLAU739 Figure 18, p. 23) turns on when sub-timer 1 fires halfway
//! through each 1 s period, and off when the main timer wraps around: 0.5 s off, 0.5 s on.
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
