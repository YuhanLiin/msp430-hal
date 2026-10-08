#![no_main]
#![no_std]

use msp430::asm;
use msp430_rt::entry;
use msp430_hal::{
    gpio::Batch, pmm::Pmm, sac::{NoninvertingGain, PositiveInput, PowerMode, SacConfig}, watchdog::Wdt
};
use panic_msp430 as _;

// Configure one of the Smart Analog Combo (SAC) units into a non-inverting amplifier.

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);

    let p1_3 = port1.pin3.to_alternate3();
    let p1_1 = port1.pin1.to_alternate3();

    // Each Smart Analog Combo unit contains a DAC and amplifier.
    let (_dac_config, amp_config) = SacConfig::begin(periph.sac0);

    // Set the Smart Analog Combo to a non-inverting amplifier with a gain of 5. The voltage at P1.3 will be multiplied by 5 and output on P1.1
    let _amp = amp_config
        .noninverting_amplifier(
            PositiveInput::ExtPin(p1_3),
            NoninvertingGain::_5,
            PowerMode::HighPerformance)
        .output_pin(p1_1);

    loop {
        asm::nop();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
