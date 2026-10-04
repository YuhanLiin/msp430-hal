#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Green onboard LED should go on when P2.3 button is pressed
// No board document covers the LED (on P1.6 here) or the button: there is none for the MSP430FR25x2.
// P2.3 only exists on the 20-pin RHL package (SLASEE4C Table 4-2, p. 14). Both pins are GPIO, PxSELx = 00
// (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60): P1.6 an output, P2.3 an input with its
// pullup, P2DIR = 0, P2REN = 1, P2OUT = 1 (SLAU445I Table 8-1, p. 313).
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
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
