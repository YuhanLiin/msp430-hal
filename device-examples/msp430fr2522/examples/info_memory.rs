#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430::asm;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Use the non-volatile information memory to toggle the red onboard LED.
// Resetting or power cycling the board toggles the red LED.
// Information memory: 256 bytes of FRAM at 1800h to 18FFh (SLASEE4C Table 6-19, p. 62). No board document
// covers the LED (on P1.0 here): there is none for the MSP430FR25x2. P1.0 is a GPIO output, P1SELx = 00
// and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO. Pmm::new clears LOCKLPM5, so the pins take on their configuration
    // (SLAU445I 8.3.1, p. 316).
    let (pmm, mut nv_mem) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Wait a little bit to 'debounce' any power cycles.
    for _ in 0..100 {
        asm::nop();
    }

    // The write method provides a mutable reference to the memory, automatically managing write protection.
    // The protection is the DFWP bit in SYSCFG0 (SLASEE4C Table 6-19 note 2, p. 62;
    // SLAU445I Table 1-29, p. 80).
    nv_mem.write(|mem|
        // Toggle the first byte between 1 and 0
        mem[0] = (mem[0].wrapping_add(1)) & 1);

    // Reads needn't worry about write protection, so can be done directly by indexing. (DFWP = 1 only makes
    // the memory "not writable": SLAU445I Table 1-29, p. 80)
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
