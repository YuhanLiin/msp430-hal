//! The internal temperature sensor: an LED on P1.0 is on while the chip is between 20.0 °C and 25.0 °C, and
//! off otherwise.
//!
//! The ADC converts the sensor, channel 12, against the internal 1.5 V reference. The device descriptors
//! (TLV) hold the factory's readings of the sensor against that reference at 30 °C and 85 °C, and
//! `TempSensorCalibration` turns each new reading into tenths of a degree with them.
//! (Channel 12: SLASEE4C Table 6-13, p. 55. The readings at 30 °C and 85 °C: SLASEE4C Table 6-18, p. 61;
//! their use: SLAU445I 1.13.3.3, p. 60. P1.0 is a GPIO output, P1SELx = 00 and P1DIR = 1: SLASEE4C
//! Table 6-15, p. 58. No board document covers the LED: there is none for the MSP430FR25x2.)
//!
//! How to test (an LED and a resistor):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example. In a room at 20 °C to 25 °C, the LED is on.
//! 3. Hold a fingertip on the MSP430FR2522: as the chip warms past 25 °C, the LED turns off. Take the
//!    finger away, and the LED turns on again as the chip cools.
//!
//! In a room below 20 °C the LED starts off: the finger turns it on as the chip passes 20 °C, and off again
//! past 25 °C.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, NegativeReference, PositiveReference, Predivider, Resolution, SampleTime, SamplingRate},
    gpio::Batch,
    pmm::{Pmm, ReferenceVoltage},
    tlv::TempSensorCalibration,
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog. The watchdog runs from every PUC and must be halted, here
    // with WDTHOLD (SLAU445I 12.2.2, p. 363; SLAU445I Table 12-2, p. 366).
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO. Pmm::new clears LOCKLPM5, so the pins take on their configuration
    // (SLAU445I 8.3.1, p. 316).
    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();
    led.set_low().ok();

    // ADC setup.
    // Temp sensor needs >= 30 us sample time (SLASEE4C Table 5-22, p. 39: tSENSOR(sample) 30 µs minimum;
    // SLAU445I 21.2.7.8, p. 556).
    // MODCLK is at most 5.8 MHz, so 256 cycles take at least 44 us (SLASEE4C Table 5-9, p. 28).
    // MODCLK in active mode also avoids erratum ADC50, which makes temperature sensor results wrong
    // with ACLK as the ADC clock in LPM3 (SLAZ705H ADC50, p. 5).
    let adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits10,
        SamplingRate::Max200ksps,
        SampleTime::Cycles256,
    )
    .use_modclk()
    .configure(periph.adc);

    // The temperature sensor is ADC channel 12 (SLASEE4C Table 6-13, p. 55)
    let vref = pmm.enable_internal_reference(ReferenceVoltage::V1_5).unwrap();
    let mut t_sense = pmm.enable_internal_temp_sensor(&vref).unwrap();

    // The device descriptors (TLV) hold the sensor readings measured in the factory at two temperatures,
    // against the internal 1.5 V reference at full resolution, so measure the same way. This is much more
    // accurate than the typical sensor voltage and slope from the data sheet.
    // SLASEE4C Table 6-18, p. 61: "ADC 1.5-V reference, temperature 30°C" and "85°C";
    // SLAU445I 1.13.3.3, p. 60. Full resolution is 10 bits on this device (SLASEE4C 6.10.12, p. 55).
    // Typical sensor voltage and slope: VSENSOR and TCSENSOR, SLASEE4C Table 5-22, p. 39.
    let mut adc = adc.with_reference(PositiveReference::Internal(&vref), NegativeReference::Avss);
    let calibration = TempSensorCalibration::new(ReferenceVoltage::V1_5);

    loop {
        let count = block!(adc.read_count(&mut t_sense)).unwrap();
        let temp_decicelsius = calibration.decicelsius(count);

        // Turn on LED if temp between 20 and 25C
        if (200..=250).contains(&temp_decicelsius) {
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
