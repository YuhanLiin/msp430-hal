//! The information memory is FRAM, which keeps its contents without power. Each start toggles a byte in
//! it between 0 and 1, and LED1 shows the byte, so every reset or power-up switches LED1 from on to off,
//! or from off to on.
//!
//! The byte is the first one of the information memory, and LED1 is on when it is 0.
//! `InfoMemory::write` lifts the write protection, DFWP, only for the write.
//! (Information memory: 512 bytes of FRAM, 1800h to 19FFh: SLASEC4D Table 6-4, p. 65. FRAM is
//! nonvolatile: SLAU445I 6.1, p. 301. DFWP: SLAU445I 1.9.3, p. 45. LED1 on P1.0 is red, and S3 is the
//! reset button: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example. LED1 is on or off, depending on what the byte held before.
//! 2. Press the reset button S3: LED1 toggles.
//! 3. Unplug the USB cable and plug it back in: LED1 toggles again, so the byte kept its value without
//!    power.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430::asm;
use msp430_rt::entry;
use msp430_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, mut nv_mem) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Wait a little bit to 'debounce' any power cycles.
    for _ in 0..100 {
        asm::nop();
    }

    // The write method provides a mutable reference to the memory, automatically managing write protection.
    // ("The information FRAM can be write protected by setting DFWP bit in SYSCFG0 register":
    // SLASEC4D Table 6-4, note 2, p. 65)
    nv_mem.write(|mem| 
        // Toggle the first byte between 1 and 0
        mem[0] = (mem[0].wrapping_add(1)) & 1
    );

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
