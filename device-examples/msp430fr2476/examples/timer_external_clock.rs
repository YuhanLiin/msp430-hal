//! TB0 clocked from its external clock pin, TB0CLK on P6.1: the timer counts the rising edges of a signal
//! from the function generator. LED1 toggles every 1000 edges.
//! (TB0CLK is P6.1: SLASEO7C Table 9-15, p. 59. It's only bonded out in the 48-pin package, which this
//! LaunchPad has: SLASEO7C Table 7-2, p. 18; the MSP430FR2476TPT in SLAU802 Figure 4, p. 6. LED1 on P1.0
//! is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator, and optionally the scope):
//! 1. Generator: square wave, 1 kHz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load High-Z. Check the
//!    levels on the scope before connecting: a negative or >3.6 V signal can damage the pin.
//! 2. Connect the generator to P6.1 (J4 pin 34), its ground to GND (J3 pin 22), and switch it on.
//! 3. Flash this example: LED1 toggles once per second, on for 1 s and off for 1 s.
//! 4. Change the frequency: LED1 follows, toggling every 1000 edges. With the scope on P1.0 (J3 pin 27),
//!    the LED's signal has 1/2000 of the generator's frequency: 50 Hz at 100 kHz, for example. Without
//!    a signal, the LED stops.
//! (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    timer::{TimerConfig, TimerParts7},
    watchdog::Wdt,
};
use panic_msp430 as _;

/// Edges of the generator's signal per LED1 toggle
const EDGES: u16 = 1000;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p6 = Batch::new(periph.p6).split(&pmm);
    let mut led1 = p1.pin0.to_output_low();

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118). The timer doesn't use them.
    let (_smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // P6.1 is TB0CLK with P6SEL = 01 and P6DIR = 0 (SLASEO7C Table 9-28, p. 70). TBSSEL = 00b selects it
    // (SLAU445I Table 14-6, p. 409), and the timer counts on its rising edges (SLAU445I 14.2.1, p. 393).
    let tb0clk = p6.pin1.to_alternate1();
    let mut timer = TimerParts7::new(periph.tb0, TimerConfig::tbclk(tb0clk)).timer;

    // Up mode counts 0 to EDGES - 1, so a period is EDGES edges (SLAU445I 14.2.3.1, p. 394)
    timer.start(EDGES - 1);

    loop {
        // TBIFG is set when the timer counts from TBxCL0 to zero (SLAU445I 14.2.3.1, p. 394)
        if timer.wait().is_ok() {
            led1.toggle().ok();
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
