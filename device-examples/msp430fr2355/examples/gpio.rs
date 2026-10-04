//! A GPIO input and output: LED2 lights while S2 is held down.
//!
//! The program polls P2.3, which S2 pulls low, and drives P6.6, LED2, high while it reads low.
//! (S2 on P2.3, and LED2 on P6.6 is green: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED2 is off, and lights while S2 is pressed.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // S2 connects P2.3 to GND and the board has no pull-up for it (SLAU680 Figure 18, p. 26)
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p6 = Batch::new(periph.p6)
        .config_pin6(|p| p.to_output())
        .split(&pmm);

    let mut p2_3 = p2.pin3;
    let mut p6_6 = p6.pin6;

    loop {
        if p2_3.is_high().unwrap() {
            p6_6.set_low().ok();
        } else {
            p6_6.set_high().ok();
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
