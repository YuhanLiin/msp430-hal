//! A polled ADC reading: LED1 is on while the voltage on P1.1 is between 1.0 V and 2.0 V, and off otherwise.
//!
//! The ADC converts P1.1, input A1, again and again, with 8-bit results against AVCC, the LaunchPad's
//! 3.3 V supply, and `read_voltage_mv()` turns each result into millivolts.
//! (A1 is P1.1: SLASEC4D Table 6-21, p. 77. AVCC is the reference after reset: SLAU445I Table 21-8, p. 567.
//! The supply: SLAU680 2.3.1, p. 12. LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test (function generator, or a jumper wire):
//! 1. Generator: the DC waveform, Offset 1.500 V, output load High-Z. Check the voltage with the multimeter
//!    first: 0 V to 3.3 V only (the analog input range: SLASEC4D Table 5-20, p. 51). Connect it to P1.1
//!    (J3 pin 28), its ground to GND (J3 pin 22).
//! 2. Flash this example: LED1 is on.
//! 3. Set the offset to 0.5 V, and then to 2.5 V: LED1 is off at both. It's on from about 1.0 V to 2.0 V.
//!
//! Without the generator, a jumper wire from P1.1 to GND (J3 pin 22) or to 3.3 V (J1 pin 1) turns LED1
//! off, and a potentiometer of about 10 kΩ between 3.3 V and GND, its wiper on P1.1, turns it on in the
//! middle of its range. (Header pins: SLAU680 Figure 10, p. 15.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, Predivider, Resolution, SampleTime, SamplingRate},
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use nb::block;
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
    // P1.1 as analog input A1: P1SELx = 11 (SLASEC4D Table 6-63, p. 96), ADCINCHx = 1 (SLASEC4D
    // Table 6-21, p. 77)
    let mut adc_pin = port1.pin1.to_alternate3();

    // ADC setup
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits8,
        SamplingRate::Max50ksps,
        SampleTime::Cycles4,
    )
    .use_modclk()
    .configure(periph.adc);

    loop {
        // Get ADC voltage, assuming the ADC reference voltage is 3300mV. The ADC measures against AVCC
        // (ADCSREFx = 000b after reset, SLAU445I Table 21-8, p. 567), and the LaunchPad supplies 3.3 V
        // to the target (SLAU680 2.3.1, p. 12).
        // It's infallible besides nb::WouldBlock, so it's safe to unwrap after block!()
        // If you want a raw count use adc.read_count() instead.
        let reading_mv = block!( adc.read_voltage_mv(&mut adc_pin, 3300) ).unwrap();

        // Turn on LED if voltage between 1000 and 2000mV (LED1 on P1.0: SLAU680 Figure 18, p. 26)
        if (1000..=2000).contains(&reading_mv) {
            led.set_high().ok();
        } else {
            led.set_low().ok();
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
