//! Routes MCLK, SMCLK and ACLK to header pins, to measure the DCO with an oscilloscope or
//! multimeter.
//!
//! MCLK runs from the DCO, locked by the FLL to REFO. Every DCO frequency except 16 MHz is
//! trimmed in software at start-up; 16 MHz uses the factory trim. Change `DCO_FREQ` to check
//! each of them. LED1, which is green, turns on once the clocks are configured (SLAU802 Figure 19,
//! p. 25). TI recommends the factory trim for the highest DCO range and the software trim for other
//! frequencies (SLAU445I 3.2.11.1, p. 106).
//!
//! Pins (LP-MSP430FR2476), with the frequencies for `_8MHz`:
//! - P1.3/MCLK (J1 pin 9): 244 x REFO = 7.995 MHz nominal
//! - P1.7/SMCLK (J3 pin 23): MCLK / 8 = 999.4 kHz nominal, within a multimeter's 1 MHz range
//! - P2.2/ACLK (J1 pin 5): REFO, 32.768 kHz nominal
//! - GND: J2 pin 20 or J3 pin 22
//!
//! (Header pins: SLAU802 Figure 10, p. 13. Clock output pins: SLASEO7C Table 9-23, p. 65 and
//! SLASEO7C Table 9-24, p. 66.)
//!
//! REFO is only accurate to ±3.5 % (SLASEO7C 8.12.3.4, p. 30), and MCLK inherits that. The FLL
//! locks MCLK to an exact multiple of REFO though, fDCOCLKDIV = (FLLN + 1) × (fFLLREFCLK ÷ n) with
//! n = 1 by default (SLAU445I 3.2.5, p. 104), so MCLK / ACLK is exactly the FLL multiplier:
//!
//! | `DCO_FREQ` | multiplier | nominal MCLK |
//! |------------|------------|--------------|
//! | `_1MHz`    | 32         | 1.048576 MHz |
//! | `_2MHz`    | 61         | 1.998848 MHz |
//! | `_4MHz`    | 122        | 3.997696 MHz |
//! | `_8MHz`    | 244        | 7.995392 MHz |
//! | `_12MHz`   | 366        | 11.993088 MHz |
//! | `_16MHz`   | 488        | 15.990784 MHz |
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

/// DCO frequency to measure
const DCO_FREQ: DcoclkFreqSel = DcoclkFreqSel::_8MHz;

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
    led.set_low().ok();

    // MCLK on P1.3, SMCLK on P1.7 and ACLK on P2.2 each need PxSEL = 10 and PxDIR = 1
    // (SLASEO7C Table 9-23, p. 65; SLASEO7C Table 9-24, p. 66)
    let _mclk_out = p1.pin3.to_output().to_alternate2();
    let _smclk_out = p1.pin7.to_output().to_alternate2();
    let _aclk_out = p2.pin2.to_output().to_alternate2();

    // MCLK from DCOCLKDIV and ACLK from REFO (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8, p. 117).
    // SMCLK "directly derives from MCLK", here divided by 8 (DIVS = 11b: SLAU445I Table 3-9, p. 118).
    // MCLK above 8 MHz needs one FRAM wait state (fSYSTEM: SLASEO7C 8.3, p. 20), which freeze() sets.
    let (_smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DCO_FREQ, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .aclk_refoclk()
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
