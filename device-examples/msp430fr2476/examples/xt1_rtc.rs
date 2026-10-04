//! RTC clocked from XT1, either directly (XT1CLK) or through ACLK.
//!
//! The RTC counts 16384 XT1 cycles per period and LED1 toggles every period, so with a
//! 32.768 kHz signal LED1 blinks at 1 Hz (0.5 s on, 0.5 s off).
//! (LED1 on P1.0 is green: SLAU802 Figure 19, p. 25)
//!
//! Wiring: function generator -> P2.1/XIN (J2 pin 18), ground -> J2 pin 20. Square wave,
//! 32.768 kHz, 0 V to 3.3 V, 50 % duty, output load High-Z (see `xt1_bypass_aclk.rs`).
//!
//! Scope: P1.0/LED1 (J3 pin 27). (Header pins: SLAU802 Figure 10, p. 13.)
//!
//! What to try:
//! 1. The LED1 period is 1.000 s at 32.768 kHz. At 16.384 kHz it should be 2 s.
//! 2. Set `RTC_FROM_ACLK` to true and repeat. The result should be identical, but now the RTC
//!    runs from ACLK (RTCSS plus the SYSCFG2.RTCCKSEL mux). If the mux were wrong the RTC would
//!    count SMCLK (1 MHz) instead and LED1 would toggle every ~16 ms. (RTCSS = 01 selects SMCLK
//!    or ACLK, chosen in SYSCFG2, and RTCSS = 10 is XT1CLK: SLASEO7C 9.10.11, p. 61;
//!    SLASEO7C Table 9-18, p. 61.)
//! 3. Switch the generator off. Through ACLK the RTC keeps running on REFO (the fail-safe
//!    covers ACLK). On XT1CLK it should stop, because the fail-safe does not cover the RTC's
//!    direct XT1CLK input. (SLAU445I 3.2.13, p. 109 to p. 110 describes the switch to REFO for
//!    MCLK, SMCLK, ACLK and the FLL reference only.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Xt1Config, Xt1clk},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    rtc::{Rtc, RtcClockSrc, RtcDiv},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

/// Frequency the function generator is set to
const XT1_FREQ_HZ: u32 = 32_768;
/// Clock the RTC from ACLK (sourced from XT1) instead of XT1CLK directly
const RTC_FROM_ACLK: bool = false;
/// XT1 cycles per LED toggle
const TICKS_PER_TOGGLE: u16 = 16_384;

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

    // P2.1 = XIN with P2SEL = 01 (SLASEO7C Table 9-24, p. 66)
    let xin = p2.pin1.to_alternate1();

    // MCLK from DCOCLKDIV (SELMS = 000b) and ACLK from XT1CLK (SELA = 00b) (SLAU445I Table 3-8,
    // p. 117); SMCLK = MCLK / 8 (DIVS = 11b: SLAU445I Table 3-9, p. 118); XT1 in bypass mode
    // (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120)
    let (_smclk, aclk, mut xt1clk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .aclk_xt1clk()
        .freeze(&mut fram);

    let rtc = Rtc::new(periph.rtc);
    if RTC_FROM_ACLK {
        // RTCSS = 01b with RTCCKSEL = 1 selects ACLK (SLASEO7C Table 9-18, p. 61; RTCSS: SLAU445I
        // Table 15-2, p. 420; RTCCKSEL in SYSCFG2: SLAU445I Table 1-31, p. 82)
        let rtc = rtc.use_aclk(&aclk);
        run(rtc, &mut led, &mut xt1clk)
    } else {
        // RTCSS = 10b selects XT1CLK (SLASEO7C Table 9-18, p. 61; SLAU445I Table 15-2, p. 420)
        let rtc = rtc.use_xt1clk(&xt1clk);
        run(rtc, &mut led, &mut xt1clk)
    }
}

fn run<SRC: RtcClockSrc>(
    mut rtc: Rtc<SRC>,
    led: &mut impl StatefulOutputPin,
    xt1clk: &mut Xt1clk,
) -> ! {
    // RTCPS = 000b divides by 1 (SLAU445I Table 15-2, p. 420)
    rtc.set_clk_div(RtcDiv::_1);
    // A period lasts `count + 1` ticks
    // (The counter resets to 0 after reaching the modulo value: SLAU445I 15.2.1, p. 417;
    // SLAU445I Figure 15-2, p. 418)
    rtc.start(TICKS_PER_TOGGLE - 1);
    loop {
        // Waits for RTCIFG, which "can be cleared by reading RTCIV register" (SLAU445I Table 15-2, p. 420)
        block!(rtc.wait()).ok();
        led.toggle().ok();
        // Clearing the sticky fault flag lets ACLK move back from REFO to XT1 once the signal
        // is healthy again (SLAU445I 3.2.13, p. 109 to p. 110)
        xt1clk.clear_fault();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
