//! A pin in an alternate function: P1.7 outputs SMCLK, a square wave of about 1 MHz, instead of being a
//! GPIO.
//!
//! P1.7 with P1SEL = 10 and P1DIR = 1 is SMCLK. The program doesn't set the clocks up, so SMCLK keeps its
//! reset setting: DCOCLKDIV, locked by the FLL to 32 times REFO's 32.768 kHz, 1.048576 MHz.
//! (SMCLK on P1.7: SLASEO7C Table 9-23, p. 65. After a reset SMCLK uses DCOCLKDIV, "locked by the FLL
//! and referenced by REFO if XT1 is not available": SLAU445I 3.2, p. 102. fDCOCLKDIV = (FLLN + 1) ×
//! (fFLLREFCLK ÷ n), with FLLN = 31 and n = 1 after a reset: SLAU445I 3.2.5, p. 104; SLAU445I
//! Table 3-6, p. 115; SLAU445I Table 3-7, p. 116.)
//!
//! How to test (the scope):
//! 1. Flash this example.
//! 2. Put the probe on P1.7 (J3 pin 23), with its ground clip on GND (J3 pin 22).
//! 3. Expected: a square wave of about 1.05 MHz; Analysis > Counter measures it. REFO is accurate to
//!    ±3.5 % (SLASEO7C 8.12.3.4, p. 30), so anything from 1.01 MHz to 1.09 MHz is right.
//! (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // Convert P1.7 to SMCLK output: P1SEL = 10 with P1DIR = 1 (SLASEO7C Table 9-23, p. 65)
    p1.pin7.to_output().to_alternate2();

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
