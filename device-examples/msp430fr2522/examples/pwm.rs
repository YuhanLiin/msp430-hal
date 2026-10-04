//! PWM on two outputs of TA0, with one period and two duty cycles: P1.5 is high for 76 % of each
//! period, and P1.4 for 2 %.
//!
//! TA0 counts SMCLK, 32 × 32768 Hz = 1.049 MHz, from 0 to 5000 in up mode: a period of 5001 counts,
//! about 4.8 ms, or 210 Hz. Each output is high from the start of a period until the timer reaches its
//! duty cycle: 3795 counts (about 3.6 ms) for TA0.2 on P1.5, and 100 counts (about 0.1 ms) for TA0.1
//! on P1.4.
//! (SMCLK is DCOCLKDIV, (FLLN + 1) × 32768 Hz, with FLLN = 31 in the 1 MHz range: SLAU445I 3.2.5,
//! p. 104. Up mode: SLAU445I 13.2.3.1, p. 371. Reset/Set mode: SLAU445I Table 13-2, p. 376. TA0.1 on
//! P1.4 and TA0.2 on P1.5: SLASEE4C Table 6-15, p. 58; SLASEE4C Figure 6-2, p. 54. No board document
//! covers the parts to connect: there is none for the MSP430FR25x2.)
//!
//! How to test (scope, two channels; or two LEDs and resistors):
//! 1. Flash this example.
//! 2. CH1 on P1.5, ground clip on GND: about 210 Hz, high for about 3.6 ms of each 4.8 ms period (76 %).
//! 3. CH2 on P1.4: the same frequency, high for about 0.1 ms of each period (2 %).
//! 4. Without the scope, connect an LED with a series resistor (about 1 kΩ) from each pin to GND: the
//!    LED on P1.5 is bright, the one on P1.4 dim.
#![no_main]
#![no_std]

use embedded_hal::pwm::SetDutyCycle;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    pwm::{PwmParts3, TimerConfig},
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let pwm = PwmParts3::new(periph.ta0, TimerConfig::smclk(&smclk), 5000);
    // TA0.1 on P1.4 and TA0.2 on P1.5: P1SELx = 10 with P1DIR = 1 (SLASEE4C Table 6-15, p. 58)
    let mut pwm4 = pwm.pwm1.init(p1.pin4.to_output().to_alternate2());
    let mut pwm5 = pwm.pwm2.init(p1.pin5.to_output().to_alternate2());

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
