#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Alternate GPIO mode demonstration

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // Convert P1.0 to SMCLK output (P1SELx = 10 with P1DIR.0 = 1: SLASEC4D Table 6-63, p. 96)
    // Expect red LED to light up (LED1 is on P1.0: SLAU680 Figure 18, p. 26)
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
