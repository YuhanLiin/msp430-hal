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

// P6.4 LED should be bright, P6.3 LED should be dim
// (LEDs connected externally: the LaunchPad's LEDs are on P1.0 and P6.6 (SLAU680 Figure 18, p. 26), and
// P6.4 and P6.3 are pins 35 and 36 of the BoosterPack header (SLAU680 Figure 10, p. 15).)
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
