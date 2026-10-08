#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    ecomp::{ECompConfig, FilterStrength, Hysteresis, NegativeInput, OutputPolarity, PositiveInput, PowerMode},
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

// Configure one of the enhanced comparator (eCOMP) modules for use: If P1.1 is less than 1.2V then LED turns on

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();

    // eCOMP configuration
    let (_dac_conf, comp_conf) = ECompConfig::begin(periph.e_comp0);

    // TODO: fix later
    // let mut comparator = comp_conf.configure(
    //         PositiveInput::_1V2,
    //         NegativeInput::COMPx_1(port1.pin1.to_alternate3()),
    //         OutputPolarity::Noninverted,
    //         PowerMode::LowPower,
    //         Hysteresis::Off,
    //         FilterStrength::Off,
    //     ).no_output_pin();

    // If P1.1 is less than 1.2V then LED turns on
    loop {
        // led.set_state(comparator.value().into()).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
