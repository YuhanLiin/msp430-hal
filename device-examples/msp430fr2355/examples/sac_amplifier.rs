//! The Smart Analog Combo SAC0 as a non-inverting amplifier with a gain of 5: the voltage on P1.3 comes out
//! five times larger on P1.1.
//!
//! SAC0's op-amp amplifies its OA0+ input, P1.3, with its internal feedback resistors set to a gain of 5,
//! and drives its output OA0O, P1.1. The LaunchPad's light sensor uses SAC2, not SAC0, so its jumpers J7 to
//! J9 can stay on.
//! (SACs are on the MSP430FR235x only: SLASEC4D 6.10.15, p. 79. SAC0's pins: SLASEC4D Table 6-27, p. 79;
//! SLASEC4D Table 6-63, p. 96. Gain 5: SLAU445I Table 20-2, p. 523; 4.95 to 5.05: SLASEC4D Table 5-25,
//! p. 56. The light sensor, on SAC2's OA2O, OA2- and OA2+ (P3.1 to P3.3) through J7 to J9: SLAU680
//! 2.2.5.1, p. 11; SLAU680 Figure 18, p. 26.)
//!
//! How to test (function generator, and the multimeter or the scope):
//! 1. Generator: the DC waveform, Offset 0.300 V, output load High-Z. Check the voltage with the multimeter
//!    first: 0 V to 3.3 V only (the input range: SLASEC4D Table 5-25, p. 55). Connect it to P1.3 (J1 pin 9),
//!    its ground to GND (J3 pin 22).
//! 2. Flash this example.
//! 3. Expected: the multimeter on P1.1 (J3 pin 28) reads about 1.50 V. At an offset of 0.100 V it reads
//!    0.50 V, at 0.600 V 3.00 V. Above about 0.65 V the output stops within 0.1 V of the 3.3 V supply
//!    (SLASEC4D Table 5-25, p. 55).
//! 4. Generator: sine, 1 kHz, 0.4 Vpp, offset 0.3 V (0.1 V to 0.5 V). The scope on P1.1 shows the same sine,
//!    from 0.5 V to 2.5 V.
//! (Header pins: SLAU680 Figure 10, p. 15.)
#![no_main]
#![no_std]

use msp430::asm;
use msp430_rt::entry;
use msp430_hal::{
    gpio::Batch, pmm::Pmm, sac::{NoninvertingGain, PositiveInput, PowerMode, SacConfig}, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);

    // OA0+ on P1.3 and OA0O on P1.1, P1SELx = 11 (SLASEC4D Table 6-63, p. 96); OA0+ is the SAC0 input
    // PSEL = 00 (SLASEC4D Table 6-27, p. 79).
    let p1_3 = port1.pin3.to_alternate3();
    let p1_1 = port1.pin1.to_alternate3();

    // Each Smart Analog Combo unit contains a DAC and amplifier.
    // (SLASEC4D 6.10.15, p. 79: an operational amplifier, and in SAC-L3 a "12-bit voltage reference DAC")
    let (_dac_config, amp_config) = SacConfig::begin(periph.sac0);

    // Set the Smart Analog Combo to a non-inverting amplifier with a gain of 5. The voltage at P1.3 will be multiplied by 5 and output on P1.1
    // (Noninverting mode with GAIN = 011b gives a gain of 5: SLAU445I Table 20-2, p. 523)
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
