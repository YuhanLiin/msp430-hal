#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{bak_mem::BackupMemory, gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

// Use the value of backup memory to toggle the red onboard LED. The red LED should flash.
// Backup memory maintains it's value through a system reset. Power loss *will* reset the backup memory, however.

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    // DON'T disable the watchdog. It will reset us after a few ms.
    //let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Interpret register block as a &mut [u8;32]
    let bk_mem = BackupMemory::as_u8s(periph.bkmem);

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
