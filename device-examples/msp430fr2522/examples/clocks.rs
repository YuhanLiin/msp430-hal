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

// Red LED should blink 1 second on, 1 second off
// No board document covers this LED (on P1.0 here): there is none for the MSP430FR25x2. P1.0 is a GPIO
// output, P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut p1_0 = p1.pin0;

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // 2^23 clock cycles, WDTIS = 010b (SLAU445I Table 12-2, p. 366): 8 MHz / 2^23 = 1.05 s
    const DELAY: WdtClkPeriods = WdtClkPeriods::_8192k;

    // blinks should be 1 second on, 1 second off
    // First an interval-mode wait from SMCLK (WDTTMSEL = 1, WDTSSEL = 00: SLAU445I Table 12-2, p. 366),
    // then watchdog mode, whose expiry resets the device and restarts the blink (SLAU445I 12.2.2, p. 363)
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
