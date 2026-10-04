//! XT1 as the FLL reference: the FLL locks the DCO to the generator's signal on XIN instead of REFO, so MCLK
//! on P1.3 runs at 244 times the generator's frequency and follows it.
//!
//! LED1 shows an XT1 fault, which makes the FLL fall back to REFO, and the blue part of LED2 shows that the
//! FLL is unlocked: the DCO can't follow the reference any more. ACLK on P2.2 runs from REFO, as a fixed
//! reference. The HAL sets no FRAM wait states for this 7.995 MHz MCLK, so the generator must not go above
//! 32.768 kHz: MCLK would pass the 8 MHz that FRAM allows without them.
//! (SELREF picks XT1CLK or REFOCLK as the FLL reference, and fDCOCLKDIV = (FLLN + 1) × (fFLLREFCLK ÷ n):
//! SLAU445I 3.2.5, p. 104. FLLUNLOCK: SLAU445I 3.2.9, p. 105. An XT1 fault switches the FLL reference to
//! REFO: SLAU445I 3.2.13, p. 109. MCLK up to 8 MHz without FRAM wait states: SLASEO7C 8.3, p. 20. LED1 on
//! P1.0 is green, the blue part of LED2 is P4.7: SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator and the scope):
//! 1. Generator: square wave, 32.768 kHz, duty cycle 50 %, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load
//!    High-Z. Check the levels, and the frequency's unit (kHz, not Hz), on the scope before connecting: a
//!    negative or >3.6 V signal can damage the pin.
//! 2. Connect it to XIN, P2.1 (J2 pin 18), its ground to GND (J2 pin 20), and switch the output on.
//! 3. Flash this example, and put the scope (10X probe) on MCLK, P1.3 (J1 pin 9), ground clip on GND
//!    (J3 pin 22). Expected: 244 × 32.768 kHz = 7.995 MHz, and LED1 and LED2 are off. SMCLK on P1.7
//!    (J3 pin 23) is MCLK / 8, 999.4 kHz, which is easier to measure precisely.
//! 4. Lower the generator's frequency: MCLK follows, 7.32 MHz at 30 kHz and 6.59 MHz at 27 kHz. Only go down,
//!    never above 32.768 kHz.
//! 5. Keep lowering it until LED2 lights blue: the DCO can't go any lower, and the FLL is unlocked.
//! 6. Set the generator back to 32.768 kHz, and switch its output off: LED1 turns on, and MCLK runs at about
//!    8 MHz from REFO. Switch the output back on: LED1 turns off, and MCLK is 7.995 MHz again.
//! (Header pins: SLAU802 Figure 10, p. 13.)
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
