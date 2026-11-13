//! Timer cascading: one timer counts the periods of another.
//! TA0 runs from ACLK (REFO, 32.768 kHz) with a period of 1 s, and LED1 toggles every period. The CCR2
//! output of TA0 clocks TA1, which counts those periods: red LED2 toggles every 5 periods, so every 5
//! s. TA1 could count up to 65536 s like this, about 18 hours. (LED1 on P1.0 is green, the red part of
//! LED2 is P5.1.)
//! On the MSP430FR247x, TA1 can count the periods of TA0, and TA3 those of TA2. (The INCLK input of TA1
//! is the CCR2 output of TA0, and that of TA3 the CCR2 output of TA2.)
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

/// ACLK cycles per TA0 period: 1 s
const ACLK_CYCLES: u16 = 32_768;
/// TA0 periods per LED2 toggle
const PERIODS: u16 = 5;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let p5 = Batch::new(periph.p5)
        .config_pin1(|p| p.to_output())
        .split(&pmm);
    let mut led1 = p1.pin0;
    let mut led2_red = p5.pin1;

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:; DIVM,
    // DIVS)
    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // TA0 counts ACLK (TASSEL = 01b) and TA1 its INCLK (TASSEL = 11b)
    let ta0 = TimerParts3::new(periph.ta0, TimerConfig::aclk(&aclk));
    // CCR2 of TA0 now pulses once per period, to clock TA1 (Reset/Set output mode)
    let ta0_periods = ta0.subtimer2.into_cascade_output();
    let mut ta0 = ta0.timer;
    let mut ta1 = TimerParts3::new(periph.ta1, TimerConfig::cascade(&ta0_periods)).timer;

    // The timers count from 0 up to and including the given value ("The number of timer counts in the
    // period is TAxCCR0 + 1")
    ta1.start(PERIODS - 1);
    ta0.start(ACLK_CYCLES - 1);

    // Each timer sets its TAIFG when it counts from TAxCCR0 to zero
    loop {
        if ta0.wait().is_ok() {
            led1.toggle().unwrap();
        }
        if ta1.wait().is_ok() {
            led2_red.toggle().unwrap();
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
