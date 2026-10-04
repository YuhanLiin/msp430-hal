//! A timer and a sub-timer: LED1 blinks, on for 0.5 s and off for 0.5 s, timed by TB0 and its CCR2.
//!
//! TB0 counts ACLK from the VLO, typically 10 kHz, divided by 2 and by 5: about 1 kHz. In up mode it
//! counts from 0 to 1000, a period of about 1 s, and LED1 goes off each time the timer wraps around to
//! 0. CCR2 is a sub-timer in compare mode: it fires at count 500, halfway through the period, and LED1
//! goes on.
//! (VLO: SLASEC4D Table 5-8, p. 40. ID and TBIDEX: SLAU445I 14.2.1.2, p. 393. Up mode: SLAU445I
//! 14.2.3.1, p. 394. Compare mode: SLAU445I 14.2.4.2, p. 399. LED1 on P1.0 is red: SLAU680 Figure 18,
//! p. 26.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 blinks about once a second, on for about 0.5 s and off for about 0.5 s. The VLO is
//!    only accurate to ±50 %, so each half can last anywhere from 0.33 s to 1 s (VLOCLK "10 kHz ±50%":
//!    SLASEC4D Table 6-9, p. 68).
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    timer::{CapCmp, SubTimer, Timer, TimerConfig, TimerDiv, TimerExDiv, TimerParts3, TimerPeriph},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut p1_0 = p1.pin0;

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let parts = TimerParts3::new(
        periph.tb0,
        TimerConfig::aclk(&aclk).clk_div(TimerDiv::_2, TimerExDiv::_5),
    );
    let mut timer = parts.timer;
    let mut subtimer = parts.subtimer2;

    set_time(&mut timer, &mut subtimer, 500);
    loop {
        block!(subtimer.wait()).unwrap();
        p1_0.set_high().unwrap();
        // first 0.5 s of timer countdown expires while subtimer expires, so this should only block
        // for 0.5 s
        // (In up mode TBxR counts from 0 to TBxCL0; CCR2's CCIFG is set when TBxR reaches TBxCL2, SLAU445I
        // 14.2.4.2, p. 399, and TBIFG when the timer counts from TBxCL0 to zero, SLAU445I 14.2.3.1, p. 394.)
        block!(timer.wait()).unwrap();
        p1_0.set_low().unwrap();
    }
}

fn set_time<T: TimerPeriph + CapCmp<C>, C>(
    timer: &mut Timer<T>,
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
