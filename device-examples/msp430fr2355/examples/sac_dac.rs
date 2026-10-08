#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{
    gpio::Batch, pmm::{Pmm, ReferenceVoltage}, sac::{BufferInput, LoadTrigger, PowerMode, SacConfig, VRef}, watchdog::Wdt
};
use panic_msp430 as _;

// Configure one of the Smart Analog Combo (SAC) units into a Digital to Analog Converter (DAC).

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);

    let p1_1 = port1.pin1.to_alternate3();

    // Each Smart Analog Combo unit contains a DAC and amplifier.
    let (dac_config, amp_config) = SacConfig::begin(periph.sac0);

    // Configure the DAC within SAC0. Let's use the internal voltage reference too.
    let vref = pmm.enable_internal_reference(ReferenceVoltage::_1V5).unwrap();
    let mut dac = dac_config.configure(VRef::Internal(&vref), LoadTrigger::Immediate);

    // To see the DAC output on a GPIO pin, we must set the SAC amplifier into buffer mode and set the DAC as the buffer input
    let _amp = amp_config.buffer(BufferInput::Dac(&dac), PowerMode::LowPower)
        .output_pin(p1_1);

    loop {
        for val in 0..4095 {
            dac.set_count(val);
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
