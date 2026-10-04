//! A GPIO input and output: an LED on P1.6 lights while a button on P2.3 is held down.
//!
//! The program polls P2.3, which the button pulls low, and drives P1.6 high while it reads low.
//! (P1.6 and P2.3 are GPIO with PxSELx = 00: SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60.
//! P2.3 only exists on the 20-pin RHL package: SLASEE4C Table 4-2, p. 14. No board document covers the
//! LED or the button: there is none for the MSP430FR25x2.)
//!
//! How to test (an LED, a resistor, and a button or a jumper wire):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.6 to GND, and a button from P2.3 to
//!    GND (the internal pullup is on). A wire from P2.3 that you touch to GND works as the button too.
//! 2. Flash this example.
//! 3. Expected: the LED is off, and lights while the button is pressed.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // P2.3 is an input with its pullup, P2DIR = 0, P2REN = 1, P2OUT = 1 (SLAU445I Table 8-1, p. 313)
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p1 = Batch::new(periph.p1)
        .config_pin6(|p| p.to_output())
        .split(&pmm);

    let mut p2_3 = p2.pin3;
    let mut p1_6 = p1.pin6;

    loop {
        if p2_3.is_high().unwrap() {
            p1_6.set_low().ok();
        } else {
            p1_6.set_high().ok();
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
