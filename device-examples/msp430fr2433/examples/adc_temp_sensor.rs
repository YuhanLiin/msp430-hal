//! The internal temperature sensor: LED1 is on while the chip is between 20.0 °C and 25.0 °C, and off
//! otherwise.
//!
//! The ADC converts the sensor, channel 12, against the internal 1.5 V reference. The device descriptors
//! (TLV) hold the factory's readings of the sensor against that reference at 30 °C and 85 °C, and
//! `TempSensorCalibration` turns each new reading into tenths of a degree with them.
//! (Channel 12: SLASE59F Table 6-15, p. 53. The readings at 30 °C and 85 °C: SLASE59F Table 6-22, p. 60;
//! their use: SLAU445I 1.13.3.3, p. 60. LED1 on P1.0 is red: SLAU739 Figure 18, p. 23.)
//!
//! How to test:
//! 1. Flash this example. In a room at 20 °C to 25 °C, LED1 is on.
//! 2. Hold a fingertip on the MSP430FR2433 (MSP1: SLAU739 Figure 2, p. 5): as the chip warms past 25 °C,
//!    LED1 turns off. Take the finger away, and LED1 turns on again as the chip cools.
//!
//! In a room below 20 °C LED1 starts off: the finger turns it on as the chip passes 20 °C, and off again
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
    // Take peripherals and disable watchdog
    // (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2, p. 363)
    let periph = msp430fr2433::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();
    led.set_low().ok();

    // ADC setup.
    // Temp sensor needs >= 30 us sample time (SLASE59F Table 5-22, p. 36: tSENSOR(sample), AM).
    // MODCLK is at most 5.8 MHz (SLASE59F Table 5-9, p. 26), so 256 cycles / 5.8 MHz = 44 us sample time.
    // (ADCSSELx = 00b MODCLK: SLAU445I Table 21-4, p. 564; ADCSHTx = 1000b, 256 cycles: SLAU445I
    // Table 21-3, p. 561; ADCRES = 01b, 10 bits, and ADCSR = 0, 200 ksps: SLAU445I Table 21-5, p. 565.)
    // MODCLK in active mode also avoids erratum ADC50, which makes temperature sensor results wrong
    // with ACLK as the ADC clock in LPM3 (SLAZ664S ADC50, p. 6).
    let adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits10,
        SamplingRate::Max200ksps,
        SampleTime::Cycles256,
    )
    .use_modclk()
    .configure(periph.adc);

    // The 1.5-V reference: REFVSEL = 00b and INTREFEN = 1 in PMMCTL2 (SLAU445I Table 2-4, p. 93 to p. 94).
    // The sensor: TSENSOREN in PMMCTL2 "must be set to turn on the sensor" (SLAU445I 2.2.9, p. 89), and it
    // is ADC channel 12 (SLASE59F Table 6-15, p. 53).
    let vref = pmm.enable_internal_reference(ReferenceVoltage::V1_5).unwrap();
    let mut t_sense = pmm.enable_internal_temp_sensor(&vref).unwrap();

    // The device descriptors (TLV) hold the sensor readings measured in the factory at two temperatures,
    // against the internal 1.5 V reference at full resolution, so measure the same way. This is much more
    // accurate than the typical sensor voltage and slope from the data sheet.
    // (TLV entries "ADC 1.5-V reference temperature 30 C" and "85 C": SLASE59F Table 6-22, p. 60; their use:
    // SLAU445I 1.13.3.3, Equation 9, p. 60; typical VSENSOR and TCSENSOR: SLASE59F Table 5-22, p. 36.)
    // VR+ = VREF and VR- = AVSS: ADCSREFx = 001b (SLAU445I Table 21-8, p. 567)
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
