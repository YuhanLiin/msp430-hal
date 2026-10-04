#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{bak_mem::BackupMemory, gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

// Use the value of backup memory to toggle the red onboard LED. The red LED should flash.
// Backup memory maintains it's value through a system reset. Power loss *will* reset the backup memory, however.
// No board document covers the LED: there is none for the MSP430FR25x2. The backup memory registers have
// no reset value (SLAU445I Table 7-1, p. 310: reset "Undefined"); the device keeps them in every mode but
// LPM4.5 (SLASEE4C Table 6-1, p. 45). P1.0 is a GPIO output, P1SELx = 00 and P1DIR = 1
// (SLASEE4C Table 6-15, p. 58).

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    // DON'T disable the watchdog. It will reset us after a few ms (SLAU445I 12.2.2, p. 363: after a PUC
    // the WDT runs "with an initial 32-ms (approximate) reset interval using the SMCLK").
    //let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO. Pmm::new clears LOCKLPM5, so the pins take on their configuration
    // (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Interpret register block as a &mut [u8;32] (32 bytes: SLASEE4C 6.10.10, p. 55)
    let bk_mem = BackupMemory::as_u8s(periph.bakmem);

    bk_mem[0] = bk_mem[0].wrapping_add(1);

    // Set the output pin high if nv_mem is a multiple of 10
    led.set_state((bk_mem[0] % 10 == 0).into()).ok();

    // Loop until the watchdog resets us
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
