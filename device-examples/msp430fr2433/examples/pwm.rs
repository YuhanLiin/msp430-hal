#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, pwm::SetDutyCycle};
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

// P1.1 LED should breathe from 0 to 100% brightness (green LED2, SLAU739 Figure 18, p. 23)
#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Hold the watchdog (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2,
    // p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // MCLK = SMCLK = about 1 MHz: DCORSEL = 000b with the FLL locked to REFO (SLAU445I Table 3-5, p. 114;
    // SLAU445I 3.2.5, p. 104), DIVM and DIVS /1 (SLAU445I Table 3-9, p. 118). ACLK = REFO: SELA = 01b
    // (SLAU445I Table 3-8, p. 117).
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // TA0 counts SMCLK (TASSEL = 10b, SLAU445I Table 13-4, p. 384) from 0 up to 5000 in Up mode, a period of
    // 5001 cycles, about 210 Hz (SLAU445I 13.2.3.1, p. 371)
    let pwm = PwmParts3::new(periph.ta0, TimerConfig::smclk(&smclk), 5000);
    // TA0.1 on P1.1: P1SELx = 10, P1DIR = 1 (SLASE59F Table 6-17, p. 55; SLASE59F Table 6-11, p. 50)
    let mut pwm1 = pwm.pwm1.init(p1.pin1.to_output().to_alternate2());

    loop {
        for percent in (0..=100).chain((0..100).rev()) {
            pwm1.set_duty_cycle_percent(percent);
            delay.delay_ms(5);
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
