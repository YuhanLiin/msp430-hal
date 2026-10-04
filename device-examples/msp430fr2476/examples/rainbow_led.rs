//! PWM on the three colors of LED2, the RGB LED: each fades up and down, out of step with the others,
//! so LED2 slowly cycles through mixed colors.
//!
//! TB0 counts SMCLK, 32 × 32768 Hz = 1.049 MHz, from 0 to 5000 in up mode, a PWM period of about 4.8 ms.
//! TB0.3 drives red on P5.1, TB0.2 green on P5.0 and TB0.1 blue on P4.7. Each duty cycle follows a
//! triangle wave from 0 to 100 % and back, the three a sixth of a cycle apart.
//! (SMCLK is DCOCLKDIV, (FLLN + 1) × 32768 Hz, with FLLN = 31 in the 1 MHz range: SLAU445I 3.2.5,
//! p. 104. TB0.1 to TB0.3: SLASEO7C Table 9-15, p. 59. LED2 is red on P5.1, green on P5.0 and blue on
//! P4.7, through jumper block J8: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED2 changes color slowly and smoothly, over and over, a full cycle every few seconds.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, pwm::SetDutyCycle};
use msp430_rt::entry;
use msp430_hal::{
    clock::*, fram::Fram, gpio::*, pmm::Pmm, pwm::*, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    // (WDTHOLD = 1 stops it: SLAU445I Table 12-2, p. 366; after a PUC it runs: SLAU445I 12.2.2, p. 363)
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);

    // Configure clocks to get accurate delay timing
    // (MCLK = SMCLK = DCOCLKDIV in the 1 MHz range: SELMS = 000b, SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118. TB0 counts SMCLK, TBSSEL = 10b: SLASEO7C Table 9-8, p. 50, in up
    // mode, where a period is TBxCL0 + 1 counts: SLAU445I 14.2.3.1, p. 394.)
    let mut fram = Fram::new(periph.frctl);
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);
    
    let pwm = PwmParts7::new(periph.tb0, TimerConfig::smclk(&smclk), 5000);
    
    // Map PWM channels to RGB LED pins
    // NOTE: This pin mapping is specific to the MSP430FR2476 LaunchPad.
    // PWM3 -> P5.1 (Red LED)
    // PWM2 -> P5.0 (Green LED)
    // PWM1 -> P4.7 (Blue LED)
    // (LED2: SLAU802 Figure 19, p. 25. TB0.3, TB0.2 and TB0.1: SLASEO7C Table 9-15, p. 59, each with
    // PxSEL = 10 and PxDIR = 1: SLASEO7C Table 9-27, p. 69 and SLASEO7C Table 9-26, p. 68.)
    let mut red = pwm.pwm3.init(p5.pin1.to_output().to_alternate2());
    let mut green = pwm.pwm2.init(p5.pin0.to_output().to_alternate2());
    let mut blue = pwm.pwm1.init(p4.pin7.to_output().to_alternate2());

    let max = red.max_duty_cycle(); // 5001: the period, CCR0 + 1
    let mut phase: u16 = 0;

    /// Simple triangle waveform generator for PWM duty cycle
    fn triangle(phase: u16, max: u16) -> u16 {
      let wrapped = phase % (2*max);
      if wrapped < max { wrapped } else { 2*max - wrapped }
    }

    loop {
      // Calculate duty cycle for each color, max/3 apart: a sixth of the triangle's period of 2*max
      let red_duty = triangle(phase, max);
      let green_duty = triangle(phase + max/3, max);
      let blue_duty = triangle(phase + 2*max/3, max);

      // Each new duty cycle loads when TB0 counts to the old one, so the running period ends with the
      // old one (CLLD = 11b: SLAU445I Table 14-2, p. 400)
      red.set_duty_cycle(red_duty).unwrap();
      green.set_duty_cycle(green_duty).unwrap();
      blue.set_duty_cycle(blue_duty).unwrap();

      // Increment phase and wrap around
      phase = (phase + 1) % (2*max);

      // Delay to slow down color change for visible rainbow effect
      delay.delay_us(100);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
