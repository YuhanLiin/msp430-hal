//! XT1 bypass mode smoke test: an external square wave on XIN drives ACLK.
//!
//! Wiring (LP-MSP430FR2476, no soldering needed: XIN is routed to the header by default):
//! - Function generator output -> P2.1/XIN (J2 pin 18), generator ground -> GND (J2 pin 20)
//! - Generator: square wave, 32.768 kHz, 50 % duty, low 0 V / high 3.3 V (3.3 Vpp, +1.65 V
//!   offset), output load set to High-Z. Check the levels on the scope *before* connecting:
//!   a negative or >3.6 V signal can damage the pin.
//!
//! (XIN reaches J2 pin 18 through R1, while R2 and R3, which would connect the crystal Y1, are not
//! fitted: SLAU802 Figure 18, p. 24. Header pins: SLAU802 Figure 10, p. 13. The bypass input is a
//! logic-level square wave with a 40 % to 60 % duty cycle: SLASEO7C 8.12.3.1, p. 27. Any pin may see
//! –0.3 V to VCC + 0.3 V at most: SLASEO7C 8.1, p. 20, and the LaunchPad's VCC is 3.3 V:
//! SLAU802 2.3.1, p. 10.)
//!
//! Scope:
//! - P2.2/ACLK (J1 pin 5): follows the generator exactly
//! - P1.3/MCLK (J1 pin 9): 8 MHz from the DCO, independent of the generator
//! - P1.7/SMCLK (J3 pin 23): MCLK / 8 = 1 MHz
//!
//! LED1 turns on once `freeze()` has returned (LED1 on P1.0 is green: SLAU802 Figure 19, p. 25).
//! Change the generator frequency (e.g. 20 kHz): ACLK should follow while MCLK and SMCLK stay put.
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
    let mut led = p1.pin0;

    // Route the internal clocks to pins so they can be measured
    // (MCLK on P1.3, SMCLK on P1.7 and ACLK on P2.2, each with PxSEL = 10 and PxDIR = 1:
    // SLASEO7C Table 9-23, p. 65; SLASEO7C Table 9-24, p. 66)
    let _mclk_out = p1.pin3.to_output().to_alternate2();
    let _smclk_out = p1.pin7.to_output().to_alternate2();
    let _aclk_out = p2.pin2.to_output().to_alternate2();

    // Bypass mode only needs XIN; XOUT (P2.0) stays a normal GPIO
    // (SLAU445I 3.2.4, p. 103: "XT1OUT is configured as a general-purpose I/O". XIN is P2.1 with
    // P2SEL = 01: SLASEO7C Table 9-24, p. 66)
    let xin = p2.pin1.to_alternate1();

    // MCLK from DCOCLKDIV (SELMS = 000b) and ACLK from XT1CLK (SELA = 00b) (SLAU445I Table 3-8,
    // p. 117); SMCLK = MCLK / 8 (DIVS = 11b: SLAU445I Table 3-9, p. 118); XT1 in bypass mode
    // (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120)
    let (_smclk, _aclk, _xt1clk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .aclk_xt1clk()
        .freeze(&mut fram);

    led.set_high().ok();

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
