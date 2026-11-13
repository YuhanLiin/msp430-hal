//! A timer clocked from the VLO, the internal very-low-power oscillator.
//! LED1 toggles every 10000 VLO cycles. The VLO runs at about 10 kHz but is only accurate to ±50 %
//! so that is anywhere from 0.7 s to 2 s. The VLO of the LP-MSP430FR2476 used
//! to test this ran at 8.0 kHz: a toggle every 1.25 s. With a scope on P1.0 (J3 pin 27), the VLO
//! frequency is 20000 divided by the period of the LED signal. (LED1 on P1.0 is green. J3 pin 27.)
//! The VLO needs no clock configuration: it starts when the timer requests it. On the MSP430FR247x, TA0
//! and TA2 can be clocked from the VLO (it is their INCLK).
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
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut led1 = p1.pin0;

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range (SELMS = 000b:; DIVM, DIVS)
    let (_smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    // TA0's INCLK is the VLO (TASSEL = 11b)
    let mut timer = TimerParts3::new(periph.ta0, TimerConfig::vloclk()).timer;
    // The timer counts from 0 up to and including the given value ("The number of timer counts in the
    // period is TAxCCR0 + 1")
    timer.start(VLO_CYCLES - 1);

    // TAIFG is set when the timer counts from TAxCCR0 to zero
    loop {
        block!(timer.wait()).unwrap();
        led1.toggle().unwrap();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
