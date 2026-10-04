//! Center-aligned PWM on TB0, with one output active low, as for the two switches of a half bridge, and
//! the period output (CCR0's output).
//!
//! TB0 counts up to 100 and back down (up/down mode), from SMCLK at 1 MHz, so a PWM period is 200 µs
//! (5 kHz). The pulses are centered on the moments the timer passes 0:
//! - P4.3 (TB0.5), duty 40, active high: high for 80 µs of each period.
//! - P4.4 (TB0.6), duty 44, active low: low for 88 µs, around the high pulse of P4.3, and high for the
//!   rest of the period. The two outputs are never high together, with 4 µs between one going low and
//!   the other going high: the dead time.
//! - P6.2 (TB0.0, the period output): a square wave at half the PWM frequency, 2.5 kHz.
//!
//! Each press of button S1 widens both pulses by 10 µs, up to a high pulse of 190 µs on P4.3, then
//! starts again at 10 µs. TB0.5 and TB0.6 share a compare latch group, so both new pulses start in the
//! same period: a period with one new and one old pulse could have both outputs high together.
//! (Center-aligned PWM and the dead time: SLAU445I 14.2.3.5 and Figure 14-9, p. 397. Up/down mode:
//! SLAU445I 14.2.3.4, p. 396. Compare latch groups: SLAU445I 14.2.4.2.2, p. 400. TB0 outputs: SLASEO7C
//! Table 9-15, p. 59. Header pins: SLAU802 Figure 10, p. 13. S1 is P4.0: SLAU802 Figure 19, p. 25.)
//!
//! How to test (scope, three channels; ground clips on GND, J3 pin 22):
//! - CH1 on P4.3 (J3 pin 24), CH2 on P4.4 (J3 pin 25), CH3 on P6.2 (J4 pin 33). Trigger on CH1, rising.
//! - CH1: 80 µs high pulses every 200 µs. CH2: 88 µs low pulses, centered on CH1's pulses. Zoom in on the
//!   edges: CH2 goes low 4 µs before CH1 goes high, and goes high 4 µs after CH1 goes low.
//! - CH3: a 2.5 kHz square wave, which toggles once per PWM period.
//! - Press S1 a few times: both pulses widen, and CH1 and CH2 are still never high together.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*, pwm::SetDutyCycle};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    pwm::{CompareLatchGroups, Polarity, PwmParts7, TimerConfig},
    watchdog::Wdt,
};
use panic_msp430 as _;

/// The timer counts from 0 up to this and back down, so a period is twice this many SMCLK cycles
const PERIOD: u16 = 100;
/// The dead time, in timer counts: P4.4's low pulse is this much longer on each side
const DEAD_TIME: u16 = 4;
/// The first duty cycle of P4.3, and the step for each press of S1, in timer counts on each side of 0
const DUTY: u16 = 40;
const STEP: u16 = 5;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // S1 pulls P4.0 low, with the internal pullup on (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I
    // Table 8-1, p. 313)
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .split(&pmm);
    let p6 = Batch::new(periph.p6).split(&pmm);
    let mut s1 = p4.pin0;

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // TB0 counts SMCLK (TBSSEL = 10b: SLAU445I Table 14-6, p. 409) in up/down mode. Its compare latches
    // load in pairs (TBCLGRP = 01b), TB0CL5 with TB0CL6 among them: the two duty cycles load together, once
    // both are written, when the timer next reaches 0 or the top, as TB0CCR5 sets (SLAU445I 14.2.4.2.2,
    // p. 400; SLAU445I Table 14-3, p. 400).
    let config = TimerConfig::smclk(&smclk).compare_latch_groups(CompareLatchGroups::Pairs);
    let pwm = PwmParts7::new_center_aligned(periph.tb0, config, PERIOD);
    // TB0.5 is P4.3 and TB0.6 is P4.4, with P4SEL = 10 (SLASEO7C Table 9-26, p. 68); TB0.0 is P6.2, with
    // P6SEL = 01 (SLASEO7C Table 9-28, p. 70). Their GPIO level is low.
    let mut high_side = pwm.pwm5.init(p4.pin3.to_output_low().to_alternate2());
    let mut low_side = pwm.pwm6.init(p4.pin4.to_output_low().to_alternate2());
    let _period_out = pwm.period_output.init(p6.pin2.to_output_low().to_alternate1());

    // The outputs start with a duty cycle of 0. Each output loads a new duty cycle when the timer next
    // reaches 0 or the top (CLLD = 10b: SLAU445I Table 14-2, p. 400), and one that loads at 0 starts in
    // the middle of a pulse. Measured on an MSP430FR2476: after the first duty cycles, both outputs were
    // high together for a moment. So the pins stay at their GPIO level, low, until both duty cycles have
    // loaded, and the duty cycles never go back to 0.
    high_side.disable();
    low_side.disable();
    // The low side is active low: its output is low for its duty cycle (output mode toggle/set instead
    // of toggle/reset: SLAU445I Table 13-2, p. 376)
    low_side.set_polarity(Polarity::ActiveLow);
    low_side.set_duty_cycle(DUTY + DEAD_TIME).unwrap();
    high_side.set_duty_cycle(DUTY).unwrap();
    // Two PWM periods of 200 µs
    delay.delay_us(400);
    low_side.enable();
    high_side.enable();

    let mut duty = DUTY;
    loop {
        // Wait for a press and release of S1, and for the bouncing to stop
        while s1.is_high().unwrap() {}
        while s1.is_low().unwrap() {}
        delay.delay_ms(20);

        // The compare latch group loads both duty cycles in the same period, whatever the order of the
        // writes. Measured on an MSP430FR2476 without the group, with the writes in the order that lets the
        // pulses overlap and 20 µs between them: 433 of 1000 changes left both outputs high together for a
        // moment. With the group: none.
        duty = if duty + STEP + DEAD_TIME < PERIOD { duty + STEP } else { STEP };
        high_side.set_duty_cycle(duty).unwrap();
        low_side.set_duty_cycle(duty + DEAD_TIME).unwrap();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
