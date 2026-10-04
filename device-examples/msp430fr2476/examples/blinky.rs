#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

// Onboard LED1 (P1.0), which is green, should blink at a steady period (SLAU802 Figure 19, p. 25).
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    // (WDTHOLD = 1 stops it: SLAU445I Table 12-2, p. 366; after a PUC it runs: SLAU445I 12.2.2, p. 363)
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut p1_0 = port1.pin0.to_output();

    // Configure clocks to get accurate delay timing
    // (MCLK from DCOCLKDIV, SELMS = 000b: SLAU445I Table 3-8, p. 117; the 8 MHz DCO range, DCORSEL =
    // 011b: SLAU445I Table 3-5, p. 114; dividers DIVM and DIVS: SLAU445I Table 3-9, p. 118. Up to 8 MHz
    // MCLK needs no FRAM wait state: SLASEO7C 8.3, p. 20.)
    let mut fram = Fram::new(periph.frctl);
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    loop {
        // `toggle()` returns a `Result` because of embedded_hal, but the result is always `Ok` with MSP430 GPIO.
        // Rust complains about unused Results, so we 'use' the Result by calling .ok()
        p1_0.toggle().ok();
        delay.delay_ms(500);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
