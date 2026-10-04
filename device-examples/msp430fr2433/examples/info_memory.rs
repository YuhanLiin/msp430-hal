//! The information memory is FRAM, which keeps its contents without power. Each start toggles a byte in
//! it between 0 and 1, and LED1 shows the byte, so every reset or power-up switches LED1 from on to off,
//! or from off to on.
//!
//! The byte is the first one of the information memory, and LED1 is on when it is 0.
//! `InfoMemory::into_unprotected` turns the write protection, DFWP, off until the next reset.
//! (Information memory: 512 bytes of FRAM, 1800h to 19FFh: SLASE59F Table 6-23, p. 61. FRAM is
//! nonvolatile: SLAU445I 6.1, p. 301. DFWP: SLAU445I 1.9.3, p. 45; set again by every PUC: SLAU445I
//! 1.12.2.1, p. 50. LED1 on P1.0 is red, and S3 is the reset button: SLAU739 Figure 18, p. 23.)
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
    let periph = msp430fr2433::Peripherals::take().unwrap();
    // Hold the watchdog (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2,
    // p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (pmm, nv_mem) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Wait a little bit to 'debounce' any power cycles.
    for _ in 0..100 {
        asm::nop();
    }

    // Disable write protection and get the information memory as an array type
    // (DFWP in SYSCFG0, set again by every PUC: SLAU445I 1.12.2.1, p. 50)
    // See also: .write() method, which keeps the write protection active except during write operations.
    let nv_mem = nv_mem.into_unprotected();

    // Toggle the first byte between 0 and 1.
    nv_mem[0] = (nv_mem[0].wrapping_add(1)) & 1;

    // Turn the LED on if 0
    led.set_state((nv_mem[0] == 0).into()).ok();

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
