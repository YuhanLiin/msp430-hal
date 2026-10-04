//! The enhanced comparator eCOMP0: LED1 is on while the voltage on P1.1 is below 1.2 V, and off above it.
//!
//! eCOMP0 compares its low-power 1.2 V reference, on the V+ input, with P1.1 (COMP0.1) on the V- input. Its
//! output is high while V+ is higher than V-, so while P1.1 is below 1.2 V. The output isn't routed to a
//! pin: the loop reads it and copies it to LED1.
//! (eCOMP0's inputs: SLASEC4D Table 6-23, p. 78. The output: SLAU445I 18.2.1, p. 505. The reference is
//! 1.20 V typical: SLASEC4D Table 5-10, p. 41. LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test (a jumper wire, and optionally the function generator):
//! 1. Flash this example.
//! 2. Connect P1.1 (J3 pin 28) to GND (J3 pin 22) with a jumper wire: LED1 is on. Move the wire to 3.3 V
//!    (J1 pin 1): LED1 is off. With nothing on P1.1 the input floats, and LED1 can be either.
//! 3. The generator on P1.1 instead of the wire: the DC waveform, output load High-Z, its ground to GND
//!    (J3 pin 22). Check the voltage with the multimeter first: 0 V to 3.3 V only (the input range:
//!    SLASEC4D Table 5-23, p. 53). At an offset of 1.100 V LED1 is on, at 1.300 V it's off.
//! (Header pins: SLAU680 Figure 10, p. 15.)
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

    let mut comparator = comp_conf.configure(
            // V+: the low-power 1.2 V reference, CPPSEL = 010b. V-: COMP0.1 on P1.1 in its P1SELx = 11
            // function, CPNSEL = 001b. (SLASEC4D Table 6-23, p. 78; SLASEC4D Table 6-63, p. 96)
            PositiveInput::_1V2,
            NegativeInput::COMPx_1(port1.pin1.to_alternate3()),
            OutputPolarity::Noninverted,
            PowerMode::LowPower,
            Hysteresis::Off,
            FilterStrength::Off,
        ).no_output_pin();

    // If P1.1 is less than 1.2V then LED turns on
    // (CPOUT is high when V+ is higher than V-: SLAU445I 18.2.1, p. 505; LED1 on P1.0: SLAU680 Figure 18,
    // p. 26)
    loop {
        led.set_state(comparator.value().into()).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
