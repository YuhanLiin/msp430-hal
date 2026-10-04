//! A pin in an alternate function: P1.0 outputs SMCLK, a square wave of about 1 MHz, instead of being a
//! GPIO, and LED1, which is on P1.0, lights.
//!
//! P1.0 with P1SELx = 10 and P1DIR = 1 is SMCLK. The program doesn't set the clocks up, so SMCLK keeps
//! its reset setting: DCOCLKDIV, locked by the FLL to 32 times REFO's 32.768 kHz, 1.048576 MHz.
//! (SMCLK on P1.0: SLASEC4D Table 6-63, p. 96. After a reset SMCLK uses DCOCLKDIV, "locked by the FLL
//! and referenced by REFO if XT1 is not available": SLAU445I 3.2, p. 102. fDCOCLKDIV = (FLLN + 1) ×
//! (fFLLREFCLK ÷ n), with FLLN = 31 and n = 1 after a reset: SLAU445I 3.2.5, p. 104; SLAU445I
//! Table 3-6, p. 115; SLAU445I Table 3-7, p. 116. LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 lights.
#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // Convert P1.0 to SMCLK output (P1SELx = 10 with P1DIR.0 = 1: SLASEC4D Table 6-63, p. 96)
    p1.pin0.to_output().to_alternate2();

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
