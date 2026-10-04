//! `try_freeze`: give up on XT1 after a timeout and fall back to the internal oscillators.
//!
//! Wiring: function generator -> P2.1/XIN (J2 pin 18), ground -> J2 pin 20. Square wave,
//! 20 kHz, 0 V to 3.3 V, 50 % duty, output load High-Z (see `xt1_bypass_aclk.rs`). The
//! generator runs at 20 kHz so ACLK from XT1 is easy to tell apart from REFO (32.768 kHz).
//!
//! Scope: CH1 on RST (J2 pin 16), CH2 on P1.0/LED1 (J3 pin 27), CH3 on P2.2/ACLK (J1 pin 5).
//! Trigger on the rising edge of RST, when the reset button is released.
//! (Header pins: SLAU802 Figure 10, p. 13. The reset button S3 pulls RST low, R11 pulls it up:
//! SLAU802 Figure 19, p. 25.)
//!
//! What to try:
//! 1. Generator on, reset the board: XT1 starts, green LED2 turns on and ACLK is 20 kHz.
//! 2. Generator off, reset the board: about 1 s later `try_freeze` gives up, the fallback
//!    configuration runs ACLK from REFO (32.768 kHz) and LED1 turns on. Measure the time
//!    from reset to LED1 to check the timeout.
//! 3. Generator off, reset, and switch the generator on within the second: XT1 still starts.
//!
//! (LED1 on P1.0 is green as well; the green part of LED2 is P5.0: SLAU802 Figure 19, p. 25.
//! REFO runs at 32.768 kHz: SLASEO7C 8.12.3.4, p. 30.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Xt1Config},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

/// Frequency the function generator is set to
const XT1_FREQ_HZ: u32 = 20_000;
/// How long to wait for XT1 before falling back to REFO
const XT1_TIMEOUT_MS: u16 = 1000;

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
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut led1 = p1.pin0;
    let mut led2_green = p5.pin0;
    led1.set_low().ok();
    led2_green.set_low().ok();

    // ACLK on P2.2 with P2SEL = 10 and P2DIR = 1, XIN on P2.1 with P2SEL = 01 (SLASEO7C Table 9-24, p. 66)
    let _aclk_out = p2.pin2.to_output().to_alternate2();
    let xin = p2.pin1.to_alternate1();

    // MCLK = SMCLK = DCOCLKDIV (SELMS = 000b) and ACLK from XT1CLK (SELA = 00b) (SLAU445I Table 3-8,
    // p. 117); XT1 in bypass mode (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120)
    let clocks = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .aclk_xt1clk();

    match clocks.try_freeze(&mut fram, XT1_TIMEOUT_MS) {
        Ok((_smclk, _aclk, _xt1clk, _delay)) => {
            led2_green.set_high().ok();
        }
        Err(clocks) => {
            // XT1 did not start: run everything that was sourced from XT1 from REFO instead
            // (XT1OFFG kept being set again while the fault lasted: SLAU445I 3.2.13, p. 109. ACLK from
            // REFO is SELA = 01b: SLAU445I Table 3-8, p. 117.)
            let (_smclk, _aclk, _delay) = clocks.xt1clk_off().freeze(&mut fram);
            led1.set_high().ok();
        }
    }

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
