//! XT1 bypass mode as the CPU clock: MCLK and SMCLK both run from the external signal on XIN.
//!
//! Wiring: function generator -> P2.1/XIN (J2 pin 18), ground -> J2 pin 20. Square wave,
//! 32.768 kHz, 0 V to 3.3 V, 50 % duty, output load High-Z (see `xt1_bypass_aclk.rs`).
//! Switch the generator on *before* resetting the board.
//!
//! Scope: P1.3/MCLK (J1 pin 9) and P1.7/SMCLK (J3 pin 23) both equal the generator frequency.
//! (Header pins: SLAU802 Figure 10, p. 13. LED1 on P1.0 is green, the red part of LED2 is P5.1:
//! SLAU802 Figure 19, p. 25. REFO runs at 32.768 kHz: SLASEO7C 8.12.3.4, p. 30.)
//!
//! What to try:
//! 1. LED1 blinks roughly 0.5 s on / 0.5 s off (`SysDelay` is a nop loop, so it is coarse
//!    at 32 kHz). The delay is calculated from the configured 32.768 kHz.
//! 2. Set the generator to 16.384 kHz: MCLK follows and the blink becomes exactly twice as slow.
//! 3. Switch the generator output off: the fail-safe moves MCLK to REFO (32.768 kHz), so the CPU
//!    keeps running, the blink returns to its original speed and red LED2 reports the XT1 fault
//!    (SLAU445I 3.2.13, p. 109).
//! 4. Switch the generator back on: the loop clears the fault flag at the next blink, LED2
//!    turns off and MCLK follows the generator again. Bypass mode leaves the start counter off,
//!    so XT1 counts as healthy as soon as the signal is back. (Once no fault remains, clearing the
//!    flags switches the clocks back: SLAU445I 3.2.13, p. 110. Start counter, ENSTFCNT1:
//!    SLAU445I Table 3-11, p. 121.)
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, MclkDiv, SmclkDiv, Xt1Config},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

/// Frequency the function generator is set to
const XT1_FREQ_HZ: u32 = 32_768;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let p5 = Batch::new(periph.p5)
        .config_pin1(|p| p.to_output())
        .split(&pmm);
    let mut led1 = p1.pin0;
    let mut led2_red = p5.pin1;

    // MCLK on P1.3 and SMCLK on P1.7 with P1SEL = 10 and P1DIR = 1 (SLASEO7C Table 9-23, p. 65);
    // XIN on P2.1 with P2SEL = 01 (SLASEO7C Table 9-24, p. 66)
    let _mclk_out = p1.pin3.to_output().to_alternate2();
    let _smclk_out = p1.pin7.to_output().to_alternate2();
    let xin = p2.pin1.to_alternate1();

    // XT1 in bypass mode (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120) sources MCLK and SMCLK
    // (SELMS = 010b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (_smclk, _aclk, mut xt1clk, mut delay) = ClockConfig::new(periph.cs)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .mclk_xt1clk(MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    loop {
        led1.toggle().ok();
        delay.delay_ms(500);
        // The fault flag is sticky and the fail-safe stays engaged until it is cleared, so
        // clear it and see whether it comes straight back
        // (The fault bits "remain set until software resets them": SLAU445I 3.2.13, p. 109. XT1OFFG:
        // SLAU445I Table 3-11, p. 122; OFIFG: SLAU445I Table 1-10, p. 63.)
        xt1clk.clear_fault();
        led2_red.set_state(xt1clk.is_faulted().into()).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
