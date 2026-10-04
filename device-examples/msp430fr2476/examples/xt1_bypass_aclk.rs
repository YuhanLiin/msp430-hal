//! XT1 in bypass mode: a square wave from the function generator on XIN becomes XT1CLK, and ACLK runs from
//! it. LED1 turns on once the clocks are set up, and ACLK on P2.2 follows the generator's frequency.
//!
//! In bypass mode XT1's oscillator is off, and XIN takes a logic-level clock signal, so no crystal is needed.
//! The LaunchPad's crystal isn't connected anyway: XIN goes to the header instead. MCLK (about 8 MHz, from
//! the DCO) and SMCLK (MCLK / 8) don't depend on XT1; they come out on P1.3 and P1.7.
//! (Bypass mode: SLAU445I 3.2.4, p. 103. The input is a square wave with a 40 % to 60 % duty cycle:
//! SLASEO7C 8.12.3.1, p. 27. XIN reaches J2 pin 18 through R1, while R2 and R3, which would connect the
//! crystal Y1, are not fitted: SLAU802 Figure 18, p. 24. Any pin may see –0.3 V to VCC + 0.3 V at most:
//! SLASEO7C 8.1, p. 20, and the LaunchPad's VCC is 3.3 V: SLAU802 2.3.1, p. 10. LED1 on P1.0 is green:
//! SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator and the scope):
//! 1. Generator: square wave, 32.768 kHz, duty cycle 50 %, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load
//!    High-Z. Check the levels, and the frequency's unit (kHz, not Hz), on the scope before connecting: a
//!    negative or >3.6 V signal can damage the pin.
//! 2. Connect it to XIN, P2.1 (J2 pin 18), its ground to GND (J2 pin 20), and switch the output on.
//! 3. Flash this example. Expected: LED1 turns on.
//! 4. Scope on ACLK, P2.2 (J1 pin 5), ground clip on GND (J3 pin 22): its counter shows the generator's
//!    32.768 kHz. With a 10X probe, MCLK on P1.3 (J1 pin 9) shows about 8 MHz, and SMCLK on P1.7 (J3 pin 23)
//!    about 1 MHz.
//! 5. Set the generator to 20 kHz: ACLK follows, while MCLK and SMCLK stay the same.
//! (Header pins: SLAU802 Figure 10, p. 13.)
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
