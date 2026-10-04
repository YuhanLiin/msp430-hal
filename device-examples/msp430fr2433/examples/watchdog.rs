#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::{Wdt, WdtClkPeriods}};
use panic_msp430 as _;

// The LED on P1.0 should toggle about once per second

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();

    // Configure watchdog for ~1 sec timeout
    Wdt::constrain(periph.watchdog_timer)
        .set_vloclk() // ~10kHz
        .set_interval_and_start(WdtClkPeriods::_8192); // ~10kHz / 8192 ~= 1 sec

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut red_led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    red_led.toggle().ok();

    // The watchdog will reset program execution when it times out
    loop {}
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
