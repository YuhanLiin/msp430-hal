#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430::asm;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Use the non-volatile information memory to toggle the red onboard LED.
// Resetting or power cycling the board toggles the red LED.

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, mut nv_mem) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Wait a little bit to 'debounce' any power cycles.
    for _ in 0..100 {
        asm::nop();
    }

    // The write method provides a mutable reference to the memory, automatically managing write protection.
    nv_mem.write(|mem|
        // Toggle the first byte between 1 and 0
        mem[0] = (mem[0].wrapping_add(1)) & 1);

    // Reads needn't worry about write protection, so can be done directly by indexing.
    // Turn the LED on if 0
    led.set_state((nv_mem[0] == 0).into()).ok();

    // If you don't care about write protection then nv_mem.into_unprotected() will
    // disable write protection and return the underlying array directly.

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
