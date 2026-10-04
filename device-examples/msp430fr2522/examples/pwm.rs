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

// P1.5 LED should be bright, P1.4 LED should be dim
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let pwm = PwmParts3::new(periph.ta0, TimerConfig::smclk(&smclk), 5000);
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
