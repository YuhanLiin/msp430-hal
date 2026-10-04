//! A pin in an alternate function: P1.2 outputs SMCLK, a square wave of about 1 MHz, instead of being a
//! GPIO.
//!
//! P1.2 with P1SELx = 10 and P1DIR = 1 is SMCLK. The program doesn't set the clocks up, so SMCLK keeps
//! its reset setting: DCOCLKDIV, locked by the FLL to 32 times REFO's 32.768 kHz, 1.048576 MHz.
//! (SMCLK on P1.2: SLASEE4C Table 6-15, p. 58. After a reset SMCLK uses DCOCLKDIV, "locked by the FLL
//! and referenced by REFO if XT1 is not available": SLAU445I 3.2, p. 102. fDCOCLKDIV = (FLLN + 1) ×
//! (fFLLREFCLK ÷ n), with FLLN = 31 and n = 1 after a reset: SLAU445I 3.2.5, p. 104; SLAU445I
//! Table 3-6, p. 115; SLAU445I Table 3-7, p. 116.)
//!
//! How to test (the scope, or an LED and a resistor):
//! 1. Flash this example.
//! 2. Put the probe on P1.2, with its ground clip on GND.
//! 3. Expected: a square wave of about 1.05 MHz; Analysis > Counter measures it. REFO is accurate to
//!    ±3.5 % (SLASEE4C Table 5-7, p. 27), so anything from 1.01 MHz to 1.09 MHz is right.
//! 4. Without the scope: an LED with a series resistor (about 1 kΩ) from P1.2 to GND lights.
#![no_main]
#![no_std]

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
    let p1 = Batch::new(periph.p1).split(&pmm);

    // Output SMCLK on P1.2: P1SELx = 10 with P1DIR = 1 (SLASEE4C Table 6-15, p. 58). (P1.7 has no SMCLK
    // function on this device: its alternate function 1 is UCA0STE, same table.)
    p1.pin2.to_output().to_alternate2();

    loop {
        msp430::asm::nop();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
