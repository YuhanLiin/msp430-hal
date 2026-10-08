#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{
    gpio::Batch, pmm::Pmm, sac::{NegativeInput, PositiveInput, PowerMode, SacConfig}, watchdog::Wdt
};
use panic_msp430 as _;

// Configure one of the Smart Analog Combo (SAC) units into a general-purpose 3-pin operational amplifier.

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);

    let p1_3 = port1.pin3.to_alternate3();
    let p1_2 = port1.pin2.to_alternate3();
    let p1_1 = port1.pin1.to_alternate3();

    // Each Smart Analog Combo unit contains a DAC and amplifier.
    let (_dac_config, amp_config) = SacConfig::begin(periph.sac0);

    // Set the Smart Analog Combo to a general-purpose opamp.  There is no internal feedback in this mode.
    let _amp = amp_config.opamp(PositiveInput::ExtPin(p1_3), NegativeInput::ExtPin(p1_2), PowerMode::HighPerformance)
        .output_pin(p1_1);

    // As-is the opamp behaves as a comparator - if the positive input (P1.3) is larger than the negative input (P1.2) the output (P1.1) goes high, otherwise its low.

    // We can make a voltage follower by shorting P1.2 (-ve in) to P1.1 (output). The voltage of P1.1/P1.2 will equal the voltage presented to P1.3 (+ve in).

    // A non-inverting amplifier can be made by placing a resistor (i.e. 10k) between P1.1 and P1.2, and another (10k) between P1.2 and GND. The output (P1.1)
    // will be double the voltage presented to P1.3.
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
