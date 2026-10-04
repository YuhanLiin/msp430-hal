//! The watchdog, left running, resets the device about every 32 ms. Each start switches an LED on P1.0 on
//! for about a quarter of that time and then off, so the LED flashes about 30 times a second, so fast
//! that it looks dimly lit.
//!
//! After every PUC the watchdog runs in watchdog mode, with a reset interval of about 32 ms, until the
//! program stops or feeds it; this one does neither. To find out in code that the watchdog caused a reset,
//! use `Pmm::take_reset_cause()`: it returns `WatchdogTimeout`.
//! (Watchdog mode after a PUC: SLAU445I 12.2.2, p. 363. Watchdog time-out, SYSRSTIV 16h: SLASEE4C
//! Table 6-10, p. 52. No board document covers the LED: there is none for the MSP430FR25x2. P1.0 is a
//! GPIO output, P1SELx = 00 and P1DIR = 1: SLASEE4C Table 6-15, p. 58.)
//!
//! How to test (an LED and a resistor, and optionally the scope):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example. Expected: the LED looks dimly lit. Fully lit would mean the reset comes before
//!    the program switches it off; off for good, that the watchdog doesn't reset the device.
//! 3. With the scope on P1.0, at 10 ms per division: a pulse about every 32 ms, one per watchdog reset,
//!    each high for about a quarter of the time.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    // DON'T pause the watchdog: it runs from every PUC until halted (SLAU445I 12.2.2, p. 363)
    //let _wdt = Wdt::constrain(periph.wdt_a);
    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    let mut red_led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Every start switches the LED on, waits, and switches it off, instead of toggling it: P1OUT has no
    // defined value after a reset (SLAU445I Table 8-10, p. 334: the reset value of PxOUT is "Undefined").
    // The watchdog counts 2^15 = 32768 SMCLK cycles (WDTIS = 100b: SLAU445I Table 12-2, p. 366), and after
    // a reset SMCLK runs at the MCLK frequency, about 1 MHz (DIVM and DIVS: SLAU445I Table 3-9, p. 118;
    // "The FLL stabilizes MCLK and SMCLK to 1 MHz", SLAU445I 3.2, p. 102). A pass of this loop takes 18
    // cycles in the dev build, so 500 passes, 9000 cycles, keep the LED on for about a quarter of that.
    red_led.set_high().ok();
    for _ in 0..500 {
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
