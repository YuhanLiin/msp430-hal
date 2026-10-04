//! `try_freeze` with a timeout: if XT1 hasn't started within 1 s, the example falls back to the internal
//! oscillators. LED2 lights green when XT1 started, and LED1 turns on when the example gave up on it.
//!
//! ACLK comes out on P2.2: from XT1 it runs at the generator's 20 kHz, after the fallback from REFO at
//! 32.768 kHz, so the two are easy to tell apart.
//! (While XT1 has no signal, its fault flag XT1OFFG keeps being set again: SLAU445I 3.2.13, p. 109. REFO runs
//! at 32.768 kHz ±3.5 %: SLASEO7C 8.12.3.4, p. 30. LED1 on P1.0 is green, the green part of LED2 is P5.0, and
//! the reset button S3 pulls RST/SBWTDIO low: SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator, and optionally the scope):
//! 1. Generator: square wave, 20 kHz, duty cycle 50 %, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load
//!    High-Z. Check the levels, and the frequency's unit (kHz, not Hz), on the scope before connecting: a
//!    negative or >3.6 V signal can damage the pin.
//! 2. Connect it to XIN, P2.1 (J2 pin 18), its ground to GND (J2 pin 20), and switch the output on.
//! 3. Flash this example. Expected: LED2 lights green. The scope on ACLK, P2.2 (J1 pin 5), ground clip on GND
//!    (J3 pin 22), counts 20 kHz.
//! 4. Switch the generator output off and press S3 (reset). Expected: about 1 s later LED1 turns on (LED2
//!    stays off), and ACLK counts about 32.8 kHz, from REFO.
//! 5. With the output off, press S3 and switch the output on within half a second: LED2 lights green instead,
//!    since XT1 started before the timeout.
//! 6. To time the timeout: probes on RST (J2 pin 16) and on LED1, P1.0 (J3 pin 27), single-shot trigger on
//!    RST's rising edge, 200 ms/div. With the output off, press and release S3: LED1 rises about 1 s after
//!    RST. Take the probe off RST before flashing again: RST is also the Spy-Bi-Wire data line.
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
