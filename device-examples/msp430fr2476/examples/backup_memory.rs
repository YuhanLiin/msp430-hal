#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{bak_mem::BackupMemory, gpio::Batch, pmm::Pmm};
use panic_msp430 as _;

// Use the value of backup memory to toggle the onboard LED1 (P1.0), which is green (SLAU802 Figure 19,
// p. 25). LED1 should flash.
// Backup memory maintains it's value through a system reset. Power loss *will* reset the backup memory, however.
// (SLASEO7C 9.10.10, p. 61: 32 bytes "retained during LPM3.5"; SLAU445I Table 7-1, p. 310 gives the
// reset value of the backup memory registers as "Undefined".)

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr247x::Peripherals::take().unwrap();

    // DON'T disable the watchdog. It will reset us after a few ms (about 32 ms, clocked by SMCLK, after
    // a PUC: SLAU445I 12.2.2, p. 363).
    //let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // Interpret register block as a &mut [u8;32] (32 bytes: SLASEO7C 9.10.10, p. 61; BAKMEM0 to
    // BAKMEM15 at 0660h: SLASEO7C Table 9-54, p. 81)
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
