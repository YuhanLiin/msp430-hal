#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

// The LED on P1.0 should flash rapidly, once per watchdog reset (LED1, red: SLAU680 Figure 18, p. 26)

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    // DON'T pause the watchdog
    //let _wdt = Wdt::constrain(periph.WDT_A);
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
