#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::{Wdt, WdtClkPeriods},
};
use panic_msp430 as _;

// The LED on P1.0 (red LED1, SLAU739 Figure 18, p. 23) should flash once per watchdog reset, about once
// per second: each start switches it on for 250 ms, switches it off, and waits for the watchdog to reset
// the device. Every start sets the LED itself, because P1OUT has no defined value after a reset
// (SLAU445I Table 8-10, p. 334: the reset value of PxOUT is "Undefined").

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();

    // Hold the watchdog while the pins and clocks are set up (WDTHOLD, SLAU445I Table 12-2, p. 366: after
    // a PUC the WDT runs, SLAU445I 12.2.2, p. 363)
    let mut wdt = Wdt::constrain(periph.watchdog_timer);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut red_led = Batch::new(periph.p1).split(&pmm).pin0.to_output();

    // MCLK = SMCLK = about 1 MHz for the delay: DCORSEL = 000b with the FLL locked to REFO (SLAU445I
    // Table 3-5, p. 114; SLAU445I 3.2.5, p. 104), DIVM and DIVS /1 (SLAU445I Table 3-9, p. 118).
    let mut fram = Fram::new(periph.fram);
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    // Configure watchdog for ~1 sec timeout: 8192 cycles of VLOCLK (WDTSSEL = 10b: SLASE59F Table 6-8,
    // p. 47; WDTIS = 101b: SLAU445I 12.3.1, Table 12-2, p. 366). VLOCLK is 10 kHz +-50% (SLASE59F
    // Table 6-7, p. 46; typically 10 kHz, SLASE59F Table 5-8, p. 26), so the interval is 0.55 s to 1.64 s,
    // 0.82 s typically.
    wdt.set_vloclk().set_interval_and_start(WdtClkPeriods::_8192);

    // On for 250 ms, well inside the shortest interval, then off until the reset
    red_led.set_high().ok();
    delay.delay_ms(250);
    red_led.set_low().ok();

    // The watchdog will reset program execution when it times out (a PUC in watchdog mode: SLAU445I
    // 12.2.2, p. 363)
    loop {}
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
