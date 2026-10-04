//! XT1 as the FLL reference: the DCO is locked to the external signal on XIN instead of REFO.
//! (The FLL reference can be XT1CLK or REFOCLK, selected by SELREF: SLAU445I 3.2.5, p. 104)
//!
//! Wiring: function generator -> P2.1/XIN (J2 pin 18), ground -> J2 pin 20. Square wave,
//! 32.768 kHz, 0 V to 3.3 V, 50 % duty, output load High-Z (see `xt1_bypass_aclk.rs`).
//!
//! Scope:
//! - P1.3/MCLK (J1 pin 9): 244 x 32.768 kHz = 7.995 MHz
//! - P1.7/SMCLK (J3 pin 23): MCLK / 8 = 999.4 kHz, easier to measure precisely
//! - P2.2/ACLK (J1 pin 5): REFO at 32.768 kHz, as a fixed reference
//!
//! (Header pins: SLAU802 Figure 10, p. 13.)
//!
//! What to try:
//! 1. Detune the generator, e.g. to 30 kHz: MCLK should follow proportionally
//!    (244 x 30 kHz = 7.32 MHz, SMCLK 915 kHz). If the FLL were still referenced to REFO,
//!    MCLK would not move at all. (fDCOCLKDIV = (FLLN + 1) × (fFLLREFCLK ÷ n): SLAU445I 3.2.5,
//!    p. 104)
//! 2. Keep detuning until the FLL can no longer follow: the blue LED2 shows the FLL is unlocked
//!    (FLLUNLOCK: SLAU445I 3.2.9, p. 105).
//! 3. Switch the generator off: the FLL reference falls back to REFO, MCLK returns to 7.995 MHz
//!    and LED1 reports the XT1 fault (SLAU445I 3.2.13, p. 109).
//! 4. Switch the generator back on: the fault clears and MCLK tracks the generator again
//!    (SLAU445I 3.2.13, p. 110).
//!
//! (LED1 on P1.0 is green, the blue part of LED2 is P4.7: SLAU802 Figure 19, p. 25.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{fll_status, ClockConfig, DcoclkFreqSel, FllStatus, MclkDiv, SmclkDiv, Xt1Config},
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
    let p4 = Batch::new(periph.p4)
        .config_pin7(|p| p.to_output())
        .split(&pmm);
    let mut led1 = p1.pin0;
    let mut led2_blue = p4.pin7;

    // MCLK on P1.3, SMCLK on P1.7 and ACLK on P2.2, each with PxSEL = 10 and PxDIR = 1; XIN on P2.1
    // with P2SEL = 01 (SLASEO7C Table 9-23, p. 65; SLASEO7C Table 9-24, p. 66)
    let _mclk_out = p1.pin3.to_output().to_alternate2();
    let _smclk_out = p1.pin7.to_output().to_alternate2();
    let _aclk_out = p2.pin2.to_output().to_alternate2();
    let xin = p2.pin1.to_alternate1();

    // MCLK from DCOCLKDIV (SELMS = 000b) and ACLK from REFO (SELA = 01b) (SLAU445I Table 3-8,
    // p. 117); SMCLK = MCLK / 8 (DIVS = 11b: SLAU445I Table 3-9, p. 118); XT1 in bypass mode
    // (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120); XT1CLK as the FLL reference (SELREF = 00b:
    // SLAU445I Table 3-7, p. 116)
    let (_smclk, _aclk, mut xt1clk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .fll_ref_xt1()
        .aclk_refoclk()
        .freeze(&mut fram);

    loop {
        // Clearing the sticky fault flag also lets the FLL move back to XT1 once the signal is
        // healthy again (SLAU445I 3.2.13, p. 110)
        xt1clk.clear_fault();
        led1.set_state(xt1clk.is_faulted().into()).ok();
        // The FLL reports the DCO as too fast, too slow or out of range (CSCTL7.FLLUNLOCK: SLAU445I
        // Table 3-11, p. 121)
        led2_blue.set_state((fll_status() != FllStatus::Locked).into()).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
