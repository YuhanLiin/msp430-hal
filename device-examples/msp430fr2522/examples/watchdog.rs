#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

// The LED on P1.0 should flash rapidly, once per watchdog reset
// No board document covers the LED: there is none for the MSP430FR25x2. P1.0 is a GPIO output,
// P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    // DON'T pause the watchdog: it runs from every PUC until halted (SLAU445I 12.2.2, p. 363)
    //let _wdt = Wdt::constrain(periph.WDT_A);
    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    let mut red_led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Every start switches the LED on, waits, and switches it off, instead of toggling it: P1OUT has no
    // defined value after a reset (SLAU445I Table 8-10, p. 334: the reset value of PxOUT is "Undefined").
    // After a reset MCLK runs at 1 MHz ("The FLL stabilizes MCLK and SMCLK to 1 MHz", SLAU445I 3.2,
    // p. 102), so 2000 loop passes of a few cycles each take far less than the 32-ms watchdog interval.
    red_led.set_high().ok();
    for _ in 0..2000 {
        msp430::asm::nop();
    }
    red_led.set_low().ok();

    // The watchdog will reset program execution after a few ms
    // (SLAU445I 12.2.2, p. 363: after a PUC the WDT runs "with an initial 32-ms (approximate) reset
    // interval using the SMCLK")
    loop {}
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
