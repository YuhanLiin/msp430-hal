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

// LED1 (P1.0), which is green, should blink 1 second on, 1 second off (SLAU802 Figure 19, p. 25)
#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog for now (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut p1_0 = p1.pin0;

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range (SELMS = 000b: SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I
    // Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the data sheet.
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    const DELAY: WdtClkPeriods = WdtClkPeriods::_8192k;

    // blinks should be 1 second on, 1 second off: 2^23 SMCLK cycles (WDTIS = 010b: SLAU445I 12.3.1,
    // p. 366) take 1.05 s at 8 MHz
    // Interval timer mode sets WDTIFG at the end of the interval instead of resetting (WDTTMSEL = 1:
    // SLAU445I 12.2.3, p. 363); SMCLK is WDTSSEL = 00b (SLAU445I Table 12-2, p. 366)
    let mut wdt = wdt.to_interval();
    p1_0.set_high().ok();
    wdt.set_smclk(&smclk).set_interval_and_start(DELAY);

    block!(wdt.wait()).ok();
    p1_0.set_low().ok();

    // In watchdog mode the end of the interval resets the device (a PUC: SLAU445I 12.2.2, p. 363),
    // which starts the next blink
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
