//! The enhanced comparator eCOMP0: LED1 is on while the voltage on P2.2 is below 1.2 V, and off above it.
//!
//! eCOMP0 compares its low-power 1.2 V reference, on the V+ input, with P2.2 (COMP0.1) on the V- input. Its
//! output is high while V+ is higher than V-, so while P2.2 is below 1.2 V. The output isn't routed to a
//! pin: the loop reads it and copies it to LED1.
//! (eCOMP0's inputs: SLASEO7C Table 9-21, p. 63. The output: SLAU445I 18.2.1, p. 505. The reference is
//! 1.20 V typical: SLASEO7C 8.12.5.1, p. 33. LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test (a jumper wire, and optionally the function generator):
//! 1. Flash this example.
//! 2. Connect P2.2 (J1 pin 5) to GND (J3 pin 22) with a jumper wire: LED1 is on. Move the wire to 3.3 V
//!    (J1 pin 1): LED1 is off. With nothing on P2.2 the input floats, and LED1 can be either.
//! 3. The generator on P2.2 instead of the wire: the DC waveform, output load High-Z, its ground to GND
//!    (J3 pin 22). Check the voltage with the multimeter first: 0 V to 3.3 V only (the input range:
//!    SLASEO7C 8.12.9.1, p. 42). At an offset of 1.100 V LED1 is on, at 1.300 V it's off.
//! (Header pins: SLAU802 Figure 10, p. 13.)
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
    // (WDTHOLD = 1 stops it: SLAU445I Table 12-2, p. 366; after a PUC it runs: SLAU445I 12.2.2, p. 363)
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let mut led = p1.pin0.to_output();

    // eCOMP configuration
    let (_dac_conf, comp_conf) = ECompConfig::begin(periph.e_comp0);

    // V+ is the low-power 1.2-V reference and V- is COMP0.1 on P2.2 (SLASEO7C Table 9-21, p. 63), with
    // P2SEL = 11 (SLASEO7C Table 9-24, p. 66). The output is high while V+ > V- (SLAU445I 18.2.1, p. 505).
    // CPPSEL and CPNSEL pick the inputs (SLAU445I Table 18-2, p. 509); CPINV = 0 (noninverted output),
    // CPMSEL = 1 (low-power mode), CPHSEL = 00b (no hysteresis) and CPFLT = 0 (no filter) are in
    // SLAU445I Table 18-3, p. 510.
    let mut comparator = comp_conf.configure(
            PositiveInput::_1V2,
            NegativeInput::COMPx_1(p2.pin2.to_alternate3()),
            OutputPolarity::Noninverted,
            PowerMode::LowPower,
            Hysteresis::Off,
            FilterStrength::Off,
        ).no_output_pin();

    // If P2.2 is less than 1.2V then LED turns on
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
