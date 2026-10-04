#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Alternate GPIO mode demonstration: SMCLK on P1.2

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // Output SMCLK on P1.2: P1SELx = 10 with P1DIR = 1 (SLASEE4C Table 6-15, p. 58). (P1.7 has no SMCLK
    // function on this device: its alternate function 1 is UCA0STE, same table.) After a reset "The FLL
    // stabilizes MCLK and SMCLK to 1 MHz" (SLAU445I 3.2, p. 102), so expect a 1 MHz square wave on P1.2,
    // or an LED there to light up (no board document covers one: there is none for the MSP430FR25x2).
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
