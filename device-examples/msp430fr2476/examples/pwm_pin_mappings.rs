//! PWM on the remapped pins of TA2 and the default pins of TA3 at the same time, the period output of
//! TA3, and switching a PWM output off and on again with button S1.
//!
//! On the MSP430FR247x, TA2 and TA3 can each use one of two pin sets, selected by TA2RMP and TA3RMP in
//! SYSCFG3 (SLASEO7C Table 9-16, p. 60; SLAU445I Table 1-32, p. 83). Here TA2 uses its remapped pins and
//! TA3 its default ones:
//! - TA2, 1 kHz: TA2.1 on P5.7 at 25 % and TA2.2 on P6.0 at 75 %
//! - TA3, 500 Hz: TA3.2 on P3.7 at 25 %, and TA3.0, the period output, on P4.1: a 250 Hz square wave
//!
//! Each press of S1 switches P3.7 between PWM and its GPIO level, low.
//! (TA2 remapped and TA3 default outputs: SLASEO7C Table 9-16, p. 60; SLASEO7C Table 9-14, p. 58. Header
//! pins: SLAU802 Figure 10, p. 13. S1 is P4.0: SLAU802 Figure 19, p. 25.)
//!
//! How to test (scope, four channels; ground clips on GND, J3 pin 22):
//! 1. CH1 on P5.7 (J3 pin 29), CH2 on P6.0 (J4 pin 36), CH3 on P3.7 (J3 pin 30), CH4 on P4.1 (J4 pin 32).
//! 2. Flash this example. CH1: 1 kHz, high for 250 µs of each 1 ms. CH2: 1 kHz, high for 750 µs. CH3:
//!    500 Hz, high for 500 µs of each 2 ms. CH4: a 250 Hz square wave, 2 ms high and 2 ms low.
//! 3. Press S1: CH3 stays low. Press again: the PWM is back.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*, pwm::SetDutyCycle};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::{DefaultMapping, RemappedMapping},
    pmm::Pmm,
    pwm::{PwmParts3, TimerConfig},
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p3 = Batch::new(periph.p3).split(&pmm);
    // S1 pulls P4.0 low, with the internal pullup on (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I
    // Table 8-1, p. 313)
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    let p6 = Batch::new(periph.p6).split(&pmm);
    let mut s1 = p4.pin0;

    // MCLK = DCOCLKDIV in the 8 MHz range, 244 × 32.768 kHz, and SMCLK = MCLK / 8, 999.4 kHz, for the
    // timers: the 1 MHz range, 32 × 32.768 kHz, is 5 % faster (SLAU445I 3.2.5, p. 104). ACLK from REFO
    // (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .aclk_refoclk()
        .freeze(&mut fram);

    // Both timers count SMCLK (TASSEL = 10b: SLAU445I Table 13-4, p. 384) in up mode, where a period is
    // the given count + 1 (SLAU445I 13.2.3.1, p. 371)
    // TA2 with TA2RMP = 1: TA2.1 is P5.7 (P5SEL = 01, SLASEO7C Table 9-27, p. 69) and TA2.2 is P6.0
    // (P6SEL = 01, SLASEO7C Table 9-28, p. 70)
    let ta2 = PwmParts3::<_, RemappedMapping>::new(periph.ta2, TimerConfig::smclk(&smclk), 999);
    let mut ta2_1 = ta2.pwm1.init(p5.pin7.to_output_low().to_alternate1());
    let mut ta2_2 = ta2.pwm2.init(p6.pin0.to_output_low().to_alternate1());
    ta2_1.set_duty_cycle(250).unwrap();
    ta2_2.set_duty_cycle(750).unwrap();

    // TA3 with TA3RMP = 0: TA3.2 is P3.7 (P3SEL = 01, SLASEO7C Table 9-25, p. 67) and TA3.0 is P4.1
    // (P4SEL = 01, SLASEO7C Table 9-26, p. 68). Setting up TA3 leaves TA2's remap bit as it is.
    let ta3 = PwmParts3::<_, DefaultMapping>::new(periph.ta3, TimerConfig::smclk(&smclk), 1999);
    let mut ta3_2 = ta3.pwm2.init(p3.pin7.to_output_low().to_alternate1());
    let _ta3_0 = ta3.period_output.init(p4.pin1.to_output_low().to_alternate1());
    ta3_2.set_duty_cycle(500).unwrap();

    let mut enabled = true;
    loop {
        // Wait for a press and release of S1, and for the bouncing to stop
        while s1.is_high().unwrap() {}
        while s1.is_low().unwrap() {}
        delay.delay_ms(20);

        // Disconnect the pin from the timer, which drives its GPIO level instead, or connect it again
        // (PxSEL: SLAU445I 8.2.5, p. 314)
        enabled = !enabled;
        if enabled {
            ta3_2.enable();
        } else {
            ta3_2.disable();
        }
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
