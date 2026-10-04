//! The 12-bit DAC of the Smart Analog Combo SAC0: its output on P1.1 ramps from 0 V up to 1.5 V and drops
//! back, over and over, a sawtooth on the scope.
//!
//! The DAC runs from the internal 1.5 V reference, giving 1.5 V × count / 4096, and SAC0's op-amp, as a
//! buffer, drives that onto its output OA0O, P1.1. The loop writes the counts 0 to 4094 as fast as it can,
//! which at the default MCLK of 1 MHz makes about 30 ramps a second. The LaunchPad's light sensor uses
//! SAC2, not SAC0, so its jumpers J7 to J9 can stay on.
//! (SACs are on the MSP430FR235x only: SLASEC4D 6.10.15, p. 79. The DAC's reference: SLASEC4D Table 6-31,
//! p. 80; 1.5 V ±1.8 %: SLASEC4D Table 5-10, p. 41. Its output: SLAU445I Table 20-3, p. 529. SAC0's pins:
//! SLASEC4D Table 6-27, p. 79; SLASEC4D Table 6-63, p. 96. The default MCLK: SLAU680 2.5, p. 13. The light
//! sensor, on SAC2's OA2O, OA2- and OA2+ (P3.1 to P3.3) through J7 to J9: SLAU680 2.2.5.1, p. 11;
//! SLAU680 Figure 18, p. 26.)
//!
//! How to test (the scope):
//! 1. Flash this example.
//! 2. Scope probe on P1.1 (J3 pin 28), its ground clip on GND (J3 pin 22), 500 mV/div and 10 ms/div.
//! 3. Expected: a sawtooth that rises from 0 V to 1.5 V in about 30 ms, then drops back to 0 V.
//! (Header pins: SLAU680 Figure 10, p. 15.)
#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430_hal::{
    gpio::Batch, pmm::{Pmm, ReferenceVoltage}, sac::{BufferInput, LoadTrigger, PowerMode, SacConfig, VRef}, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);

    // OA0O on P1.1, P1SELx = 11 (SLASEC4D Table 6-63, p. 96)
    let p1_1 = port1.pin1.to_alternate3();

    // Each Smart Analog Combo unit contains a DAC and amplifier.
    // (SLASEC4D 6.10.15, p. 79: an operational amplifier, and in SAC-L3 a "12-bit voltage reference DAC")
    let (dac_config, amp_config) = SacConfig::begin(periph.sac0);

    // Configure the DAC within SAC0. Let's use the internal voltage reference too.
    // (DACSREF = 1 selects the internal shared reference: SLASEC4D Table 6-31, p. 80)
    let vref = pmm.enable_internal_reference(ReferenceVoltage::V1_5).unwrap();
    let mut dac = dac_config.configure(VRef::Internal(&vref), LoadTrigger::Immediate);

    // To see the DAC output on a GPIO pin, we must set the SAC amplifier into buffer mode and set the DAC as the buffer input
    // (In buffer mode the noninverting input comes "from the external OAx+, DAC, or the output of paired OA":
    // SLAU445I 20.2.2.3, p. 524. For SAC0, PSEL = 01 is the SAC0 12-bit DAC: SLASEC4D Table 6-27, p. 79.)
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
