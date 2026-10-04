//! The clock system and the watchdog in both its modes: LED1 blinks, on for about 1 s and off for about
//! 1 s, and each off time ends with a watchdog reset that starts the program again.
//!
//! MCLK and SMCLK run from the DCO at about 8 MHz. With LED1 on, the watchdog counts 2^23 SMCLK cycles,
//! 1.05 s, as an interval timer. Then LED1 goes off, and the watchdog counts them again in watchdog
//! mode, which resets the device at the end, so the program starts over.
//! (Interval timer mode: SLAU445I 12.2.3, p. 363. Watchdog mode resets the device with a PUC: SLAU445I
//! 12.2.2, p. 363. LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 blinks, on for about 1 s and off for about 1 s.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::{Wdt, WdtClkPeriods},
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    let wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut p1_0 = p1.pin0;

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    const DELAY: WdtClkPeriods = WdtClkPeriods::_8192k;

    // 2^23 SMCLK cycles at 8 MHz, about 1.05 s (WDTIS = 010b: SLAU445I Table 12-2, p. 366)
    let mut wdt = wdt.to_interval();
    p1_0.set_high().ok();
    wdt.set_smclk(&smclk).set_interval_and_start(DELAY);

    block!(wdt.wait()).ok();
    p1_0.set_low().ok();

    let mut wdt = wdt.to_watchdog();
    wdt.set_interval_and_start(DELAY);

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
