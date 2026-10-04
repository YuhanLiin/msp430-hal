//! A reset when the DCO runs too fast for the FLL (`ClockConfig::reset_on_fll_unlock`). XT1 in bypass
//! mode, from a function generator, is the FLL reference. Lowering the generator's frequency suddenly
//! leaves the DCO faster than the FLL can pull it down, which resets the device.
//! (The PUC comes when FLLULPUC = 1 and FLLUNLOCK = 10b, too fast: SLAU445I Figure 3-4, p. 106. The
//! reset cause: FLL unlock, SYSRSTIV 24h: SLASEO7C Table 9-10, p. 52.)
//!
//! LED1 (green) is on while the program runs, and the red part of LED2 shows that the last reset was an
//! FLL unlock reset. MCLK is on P1.3 and MCLK / 8 on P1.7, for the scope.
//! (LED1 is P1.0 and the red part of LED2 P5.1: SLAU802 Figure 19, p. 25. Header pins: SLAU802 Figure 10,
//! p. 13.)
//!
//! How to test (function generator and scope):
//! 1. Generator: square wave, 32.768 kHz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), 50 % duty, output load
//!    High-Z. Check the levels on the scope first, then connect it to P2.1/XIN (J2 pin 18), with its
//!    ground on GND (J2 pin 20). See `xt1_bypass_aclk.rs`.
//! 2. Flash this example and switch the generator output on. LED1 is on, and the scope on P1.7 (J3
//!    pin 23) shows 999.4 kHz: MCLK is 244 × 32.768 kHz = 7.995 MHz.
//! 3. Change the generator to 20 kHz. The board resets: the red LED2 turns on, and P1.7 shows 610 kHz,
//!    as the FLL locks MCLK to 244 × 20 kHz = 4.88 MHz after the restart.
//! 4. Change the generator back to 32.768 kHz. No reset this time: the DCO is now too slow, which only
//!    sets FLLUNLOCK. Press the reset button S3 to start again at 7.995 MHz; the red LED2 turns off.
//!
//! Don't set the generator above 32.768 kHz: MCLK would follow it above 8 MHz, the most the FRAM runs at
//! without wait states (SLASEO7C 8.3, p. 20).
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Xt1Config},
    fram::Fram,
    gpio::Batch,
    pmm::{Pmm, ResetCause},
    watchdog::Wdt,
};
use panic_msp430 as _;

/// The frequency the generator is set to at the start
const XT1_FREQ_HZ: u32 = 32_768;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // Read the first reset cause, then the rest, which also clears them for the next reset (reading
    // SYSRSTIV clears the highest pending flag: SLAU445I 1.3.7, p. 36)
    let cause = pmm.take_reset_cause();
    while pmm.take_reset_cause().is_some() {}

    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    let mut led1 = p1.pin0.to_output_low();
    let mut led2_red = p5.pin1.to_output_low();
    led2_red.set_state((cause == Some(ResetCause::FllUnlock)).into()).ok();

    // MCLK on P1.3 and SMCLK on P1.7, each with PxSEL = 10 and PxDIR = 1 (SLASEO7C Table 9-23, p. 65)
    let _mclk_out = p1.pin3.to_output().to_alternate2();
    let _smclk_out = p1.pin7.to_output().to_alternate2();
    // XIN is P2.1 with P2SEL = 01 (SLASEO7C Table 9-24, p. 66). In bypass mode XOUT stays a GPIO
    // (SLAU445I 3.2.4, p. 103).
    let xin = p2.pin1.to_alternate1();

    // MCLK from DCOCLKDIV (SELMS = 000b: SLAU445I Table 3-8, p. 117), the FLL locked to XT1CLK (SELREF:
    // SLAU445I 3.2.5, p. 104), SMCLK = MCLK / 8 (DIVS = 11b: SLAU445I Table 3-9, p. 118), and the FLL
    // unlock reset (FLLULPUC: SLAU445I Table 3-11, p. 121)
    let (_smclk, _aclk, _xt1clk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .fll_ref_xt1()
        .aclk_refoclk()
        .reset_on_fll_unlock()
        .freeze(&mut fram);

    led1.set_high().ok();

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
