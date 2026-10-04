//! The backup memory keeps a count through the watchdog resets. The watchdog, left running, resets
//! the device about every 32 ms. Each start adds 1 to the count and switches LED1 on if the count is a
//! multiple of 10, so LED1 gives a short flash about three times a second.
//!
//! A reset doesn't change the backup memory: its reset value is "Undefined". It keeps its value as long as
//! it is powered, which it is in every mode but LPM4.5. A power cycle loses the count, but LED1 can't
//! show that: the count then starts from an unknown value.
//! (32 bytes: SLASEC4D 6.10.10, p. 76. Reset value: SLAU445I Table 7-1, p. 310. Powered: SLASEC4D
//! Table 6-1, p. 62. The watchdog runs after every PUC: SLAU445I 12.2.2, p. 363. LED1 on P1.0 is red:
//! SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 flashes briefly, about three times a second. Each flash lasts one watchdog interval,
//!    about 32 ms, and the nine starts in between leave LED1 off. If a reset cleared the count, LED1
//!    would stay off.
//! 3. About every 8 s two flashes come closer together: the 8-bit count wraps from 255 to 0, and 250 and
//!    0 are only 6 apart.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{bak_mem::BackupMemory, gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr2355::Peripherals::take().unwrap();

    // DON'T disable the watchdog. It will reset us after about 32 ms (SLAU445I 12.1, p. 361: after a
    // PUC the WDT runs in watchdog mode "with an initial approximately 32-ms reset interval using the
    // SMCLK").
    //let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Interpret register block as a &mut [u8;32] (32 bytes: SLASEC4D 6.10.10, p. 76)
    let bk_mem = BackupMemory::as_u8s(periph.bakmem);

    bk_mem[0] = bk_mem[0].wrapping_add(1);

    // Set the output pin high if the count is a multiple of 10
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
