//! A GPIO input and output: the green part of LED2 lights while S2 is held down.
//!
//! The program polls P2.3, which S2 pulls low, and drives P5.0, the green part of LED2, high while it
//! reads low.
//! (S2 on P2.3, pulled up by R10, 47 kΩ, and LED2's green part on P5.0: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: the green part of LED2 is off, and lights while S2 is pressed.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // P2.3 is an input with the internal pullup as well (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I
    // Table 8-1, p. 313), so it reads high until S2 is pressed
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p5 = Batch::new(periph.p5)
        .config_pin0(|p| p.to_output())
        .split(&pmm);

    let mut p2_3 = p2.pin3;
    let mut led2_green = p5.pin0;

    loop {
        if p2_3.is_high().unwrap() {
            led2_green.set_low().ok();
        } else {
            led2_green.set_high().ok();
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
