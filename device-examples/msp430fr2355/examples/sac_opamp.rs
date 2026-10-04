//! The op-amp of the Smart Analog Combo SAC0 on its own pins, with no feedback inside: P1.3 is its +
//! input, P1.2 its - input and P1.1 its output. Wired up on the header, it's a comparator, a voltage
//! follower or an amplifier.
//!
//! In general-purpose mode SAC0's OA0+ and OA0- pins are the op-amp's inputs, and it drives its output
//! OA0O. The LaunchPad's light sensor uses SAC2, not SAC0, so its jumpers J7 to J9 can stay on.
//! (SACs are on the MSP430FR235x only: SLASEC4D 6.10.15, p. 79. General-purpose mode: SLAU445I 20.2.2.1,
//! p. 522. SAC0's pins: SLASEC4D Table 6-27, p. 79; SLASEC4D Table 6-63, p. 96. The light sensor, on SAC2's
//! OA2O, OA2- and OA2+ (P3.1 to P3.3) through J7 to J9: SLAU680 2.2.5.1, p. 11; SLAU680 Figure 18, p. 26.)
//!
//! How to test (jumper wires and the multimeter; optionally the function generator and two 10 kΩ resistors):
//! 1. Flash this example.
//! 2. A comparator: jumper wires from P1.3 (J1 pin 9) to 3.3 V (J1 pin 1) and from P1.2 (J1 pin 10) to GND
//!    (J3 pin 22). The multimeter on P1.1 (J3 pin 28) reads close to 3.3 V. Swap the two wires: close to
//!    0 V. The output swings to within 0.1 V of the supply (SLASEC4D Table 5-25, p. 55).
//! 3. A voltage follower: a wire from P1.2 to P1.1, and the generator on P1.3: the DC waveform, Offset
//!    1.000 V, output load High-Z, its ground to GND, checked with the multimeter first: 0 V to 3.3 V only
//!    (the input range: SLASEC4D Table 5-25, p. 55). P1.1 reads 1.00 V too, and follows the offset.
//! 4. An amplifier with a gain of 2: replace the wire from P1.2 to P1.1 with a 10 kΩ resistor, and add one
//!    from P1.2 to GND. P1.1 reads twice the voltage on P1.3: 2.00 V at 1.000 V.
//! (Header pins: SLAU680 Figure 10, p. 15.)
#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{
    gpio::Batch, pmm::Pmm, sac::{NegativeInput, PositiveInput, PowerMode, SacConfig}, watchdog::Wdt
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

    // OA0+ on P1.3, OA0- on P1.2 and OA0O on P1.1, P1SELx = 11 (SLASEC4D Table 6-63, p. 96); OA0+ and
    // OA0- are the SAC0 inputs PSEL = 00 and NSEL = 00 (SLASEC4D Table 6-27, p. 79).
    let p1_3 = port1.pin3.to_alternate3();
    let p1_2 = port1.pin2.to_alternate3();
    let p1_1 = port1.pin1.to_alternate3();

    // Each Smart Analog Combo unit contains a DAC and amplifier.
    // (SLASEC4D 6.10.15, p. 79: an operational amplifier, and in SAC-L3 a "12-bit voltage reference DAC")
    let (_dac_config, amp_config) = SacConfig::begin(periph.sac0);

    // Set the Smart Analog Combo to a general-purpose opamp.  There is no internal feedback in this mode.
    // (In GP mode "OAx+ and OAx- pins are dedicated as noninverting and inverting inputs": SLAU445I
    // 20.2.2.1, p. 522)
    let _amp = amp_config.opamp(PositiveInput::ExtPin(p1_3), NegativeInput::ExtPin(p1_2), PowerMode::HighPerformance)
        .output_pin(p1_1);

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
