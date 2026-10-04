//! PWM on two outputs of TB3, with one period and two duty cycles: P6.4 is high for 76 % of each
//! period, and P6.3 for 2 %.
//!
//! TB3 counts SMCLK, 32 × 32768 Hz = 1.049 MHz, from 0 to 5000 in up mode: a period of 5001 counts,
//! about 4.8 ms, or 210 Hz. Each output is high from the start of a period until the timer reaches its
//! duty cycle: 3795 counts (about 3.6 ms) for TB3.5 on P6.4, and 100 counts (about 0.1 ms) for TB3.4
//! on P6.3.
//! (SMCLK is DCOCLKDIV, (FLLN + 1) × 32768 Hz, with FLLN = 31 in the 1 MHz range: SLAU445I 3.2.5,
//! p. 104. Up mode: SLAU445I 14.2.3.1, p. 394. Reset/Set mode: SLAU445I Table 14-4, p. 401. TB3.4 and
//! TB3.5: SLASEC4D Table 6-19, p. 75.)
//!
//! How to test (scope, two channels; ground clips on GND, J3 pin 22):
//! 1. Flash this example.
//! 2. CH1 on P6.4 (J4 pin 35): about 210 Hz, high for about 3.6 ms of each 4.8 ms period (76 %).
//! 3. CH2 on P6.3 (J4 pin 36): the same frequency, high for about 0.1 ms of each period (2 %).
//! (Header pins: SLAU680 Figure 10, p. 15. The LaunchPad's LEDs are on other pins, P1.0 and P6.6:
//! SLAU680 Figure 18, p. 26.)
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
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p6 = Batch::new(periph.p6).split(&pmm);

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let pwm = PwmParts7::new(periph.tb3, TimerConfig::smclk(&smclk), 5000);
    // TB3.4 on P6.3 and TB3.5 on P6.4, P6SELx = 01 with P6DIR = 1 (SLASEC4D Table 6-68, p. 106;
    // SLASEC4D Table 6-19, p. 75)
    let mut pwm4 = pwm.pwm4.init(p6.pin3.to_output().to_alternate1());
    let mut pwm5 = pwm.pwm5.init(p6.pin4.to_output().to_alternate1());

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
