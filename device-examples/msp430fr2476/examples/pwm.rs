//! PWM on two outputs of TB0, with one period and two duty cycles: P4.3 is high for 76 % of each
//! period, and P5.2 for 2 %.
//!
//! TB0 counts SMCLK, 32 × 32768 Hz = 1.049 MHz, from 0 to 5000 in up mode: a period of 5001 counts,
//! about 4.8 ms, or 210 Hz. Each output is high from the start of a period until the timer reaches its
//! duty cycle: 3795 counts (about 3.6 ms) for TB0.5 on P4.3, and 100 counts (about 0.1 ms) for TB0.4
//! on P5.2.
//! (SMCLK is DCOCLKDIV, (FLLN + 1) × 32768 Hz, with FLLN = 31 in the 1 MHz range: SLAU445I 3.2.5,
//! p. 104. Up mode: SLAU445I 14.2.3.1, p. 394. Reset/Set mode: SLAU445I Table 14-4, p. 401. TB0.4 and
//! TB0.5: SLASEO7C Table 9-15, p. 59.)
//!
//! How to test (scope, two channels; ground clips on GND, J3 pin 22):
//! 1. Flash this example.
//! 2. CH1 on P4.3 (J3 pin 24): about 210 Hz, high for about 3.6 ms of each 4.8 ms period (76 %).
//! 3. CH2 on P5.2 (J4 pin 40): the same frequency, high for about 0.1 ms of each period (2 %).
//! (Header pins: SLAU802 Figure 10, p. 13. The LaunchPad has no LEDs on these pins: SLAU802
//! Figure 19, p. 25.)
#![no_main]
#![no_std]

use embedded_hal::pwm::SetDutyCycle;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    pwm::{PwmParts7, TimerConfig},
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
    let p5 = Batch::new(periph.p5).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range (SELMS = 000b: SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I
    // Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the data sheet.
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // TB0 counts SMCLK (TBSSEL = 10b: SLASEO7C Table 9-8, p. 50) in up mode, where a period is
    // TBxCL0 + 1 counts (SLAU445I 14.2.3.1, p. 394)
    let pwm = PwmParts7::new(periph.tb0, TimerConfig::smclk(&smclk), 5000);
    // TB0.4 is P5.2 and TB0.5 is P4.3 (SLASEO7C Table 9-15, p. 59), each with PxSEL = 10 and PxDIR = 1
    // (SLASEO7C Table 9-27, p. 69; SLASEO7C Table 9-26, p. 68)
    let mut pwm4 = pwm.pwm4.init(p5.pin2.to_output().to_alternate2());
    let mut pwm5 = pwm.pwm5.init(p4.pin3.to_output().to_alternate2());

    pwm4.set_duty_cycle(100).unwrap();
    pwm5.set_duty_cycle(3795).unwrap();

    loop {}
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
