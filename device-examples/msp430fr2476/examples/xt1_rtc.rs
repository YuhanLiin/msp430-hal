//! The RTC clocked from XT1, directly from XT1CLK or through ACLK: it counts 16384 cycles of the generator's
//! signal on XIN between toggles of LED1, so at 32.768 kHz LED1 toggles every 0.5 s.
//!
//! `RTC_FROM_ACLK` picks the path. The fail-safe covers ACLK but not the RTC's own XT1CLK input: without a
//! signal the RTC stops on XT1CLK, while through ACLK it counts REFO instead. The main loop clears the fault
//! flag at each toggle, which moves ACLK back to XT1 once the signal is back.
//! (RTCSS = 10b selects XT1CLK, and 01b SMCLK or ACLK, chosen by RTCCKSEL in SYSCFG2: SLASEO7C 9.10.11,
//! p. 61; SLASEO7C Table 9-18, p. 61. SLAU445I 3.2.13, p. 109 to p. 110 describes the switch to REFO for
//! MCLK, SMCLK, ACLK and the FLL reference only. LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator, and optionally the scope):
//! 1. Generator: square wave, 32.768 kHz, duty cycle 50 %, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load
//!    High-Z. Check the levels, and the frequency's unit (kHz, not Hz), on the scope before connecting: a
//!    negative or >3.6 V signal can damage the pin.
//! 2. Connect it to XIN, P2.1 (J2 pin 18), its ground to GND (J2 pin 20), and switch the output on.
//! 3. Flash this example. Expected: LED1 toggles every 0.5 s, a period of 1 s. The scope on LED1, P1.0
//!    (J3 pin 27), ground clip on GND (J3 pin 22), shows it exactly.
//! 4. Set the generator to 16.384 kHz: LED1 toggles every second. Set it back to 32.768 kHz.
//! 5. Switch the generator output off: LED1 stops, because the fail-safe doesn't cover XT1CLK. Switch it back
//!    on: LED1 carries on.
//! 6. Set `RTC_FROM_ACLK` to true and flash again: LED1 toggles the same way, but the RTC counts ACLK
//!    (RTCSS = 01b, RTCCKSEL = 1). LED1 toggling every 16 ms instead would mean it counts SMCLK, 1 MHz.
//! 7. Switch the generator output off: LED1 keeps toggling at nearly the same rate, because ACLK falls back
//!    to REFO. Set `RTC_FROM_ACLK` back to false afterwards.
//! (Header pins: SLAU802 Figure 10, p. 13.)
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
