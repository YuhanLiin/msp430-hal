#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Alternate GPIO mode demonstration

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // Convert P1.7 to SMCLK output: P1SEL = 10 with P1DIR = 1 (SLASEO7C Table 9-23, p. 65).
    // SMCLK runs from the DCO at 1 MHz after reset (SLAU802 2.5, p. 12; after a PUC, "MCLK and SMCLK
    // are sourced from DCOCLKDIV": SLAU445I 3.2.5.1, p. 104). There is no LED on P1.7
    // (SLAU802 Figure 19, p. 25): measure the 1 MHz on J3 pin 23 (SLAU802 Figure 10, p. 13).
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
