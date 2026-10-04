//! The watchdog, left running, resets the device about every 32 ms. Each start switches LED1 on for about
//! a quarter of that time and then off, so LED1 flashes about 30 times a second, so fast that it looks
//! dimly lit.
//!
//! After every PUC the watchdog runs in watchdog mode, with a reset interval of about 32 ms, until the
//! program stops or feeds it; this one does neither. To find out in code that the watchdog caused a reset,
//! use `Pmm::take_reset_cause()`: it returns `WatchdogTimeout`.
//! (Watchdog mode after a PUC: SLAU445I 12.2.2, p. 363. Watchdog time-out, SYSRSTIV 16h: SLASEC4D
//! Table 6-12, p. 70. LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 looks dimly lit. Fully lit would mean the reset comes before the program switches it
//!    off; off for good, that the watchdog doesn't reset the device.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    // DON'T pause the watchdog
    //let _wdt = Wdt::constrain(periph.wdt_a);
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

    // The watchdog will reset program execution after about 32 ms (SLAU445I 12.1, p. 361: after a PUC
    // the WDT runs in watchdog mode "with an initial approximately 32-ms reset interval using the SMCLK")
    loop {}
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
