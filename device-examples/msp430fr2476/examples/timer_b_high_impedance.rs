//! Switching all TB0 outputs to high impedance with the TB0TRG pin, P3.5: while P3.5 is high, the PWM on
//! the outputs stops, for example to stop a motor driver on a fault.
//!
//! TB0 runs PWM at 1 kHz, 50 %, on P4.3 (TB0.5) for the scope and on P5.1 (TB0.3), which drives the red
//! part of LED2. P3.5 has its pulldown on, so with nothing connected it's low and the PWM runs: LED2
//! glows red at half brightness. Pulling P3.5 high makes all TB0 outputs high impedance: the LED goes
//! dark and P4.3 stops switching. Once P3.5 is low again, the PWM continues.
//! ("When the TBOUTH pin function is selected for the pin ... and when the pin is pulled high, all
//! Timer_B outputs are in a high-impedance state": SLAU445I 14.2.5, p. 401. TB0TRGSEL = 1 selects P3.5:
//! SLASEO7C Table 9-17, p. 61. TB0 outputs: SLASEO7C Table 9-15, p. 59. LED2's red part is P5.1:
//! SLAU802 Figure 19, p. 25. Header pins: SLAU802 Figure 10, p. 13.)
//!
//! After reset, eCOMP0's output is the trigger instead, so a TB0 PWM stops whenever eCOMP0's output is
//! high (TB0TRGSEL = 0: SLASEO7C Table 9-17, p. 61; SLAU445I Table 1-31, p. 82). `HighImpedanceTrigger::None`
//! switches the trigger off.
//!
//! How to test (a jumper wire, and the scope):
//! 1. Flash this example: LED2 glows red. Probe P4.3 (J3 pin 24) with the scope, ground on GND (J3
//!    pin 22): a 1 kHz square wave.
//! 2. Connect P3.5 (J1 pin 7) to 3.3 V (J1 pin 1) with a jumper wire: LED2 goes dark, and P4.3 stops
//!    switching. The pin floats now, so the scope may show a flat line at any level.
//! 3. Remove the wire: the LED and the square wave are back.
//!
//! Instead of the wire, the function generator can drive P3.5: a 1 Hz square wave from 0 V to 3.3 V
//! (output load High-Z; check the levels on the scope first). The PWM then stops every other half
//! second.
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
    timer::HighImpedanceTrigger,
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
    let p4 = Batch::new(periph.p4).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);

    // MCLK = DCOCLKDIV in the 8 MHz range, 244 × 32.768 kHz, and SMCLK = MCLK / 8, 999.4 kHz, for the
    // timer: the 1 MHz range, 32 × 32.768 kHz, is 5 % faster (SLAU445I 3.2.5, p. 104). ACLK from REFO
    // (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .aclk_refoclk()
        .freeze(&mut fram);

    // P3.5 is TB0TRG with P3SEL = 10 and P3DIR = 0 (SLASEO7C Table 9-25, p. 67), with its pulldown on
    // (PxREN = 1, PxOUT = 0: SLAU445I Table 8-1, p. 313), so it's low while nothing drives it
    let trigger = p3.pin5.pulldown().to_alternate2();

    // TB0 counts SMCLK (TBSSEL = 10b: SLAU445I Table 14-6, p. 409) in up mode: 1000 counts per period
    // (SLAU445I 14.2.3.1, p. 394). TB0TRGSEL = 1 selects P3.5 as the trigger (SLAU445I Table 1-31, p. 82).
    let config = TimerConfig::smclk(&smclk).high_impedance_trigger(HighImpedanceTrigger::Pin(&trigger));
    let pwm = PwmParts7::new(periph.tb0, config, 999);
    // TB0.5 is P4.3 and TB0.3 is P5.1, with PxSEL = 10 (SLASEO7C Table 9-26, p. 68; SLASEO7C Table 9-27,
    // p. 69)
    let mut scope_out = pwm.pwm5.init(p4.pin3.to_output_low().to_alternate2());
    let mut led_out = pwm.pwm3.init(p5.pin1.to_output_low().to_alternate2());
    scope_out.set_duty_cycle(500).unwrap();
    led_out.set_duty_cycle(500).unwrap();

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
