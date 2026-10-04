//! The internal temperature sensor: LED1 is on while the chip is between 20.0 °C and 25.0 °C, and off
//! otherwise.
//!
//! The ADC converts the sensor, channel 12, against the internal 1.5 V reference. The device descriptors
//! (TLV) hold the factory's readings of the sensor against that reference at 30 °C and 105 °C, and
//! `TempSensorCalibration` turns each new reading into tenths of a degree with them.
//! (Channel 12: SLASEO7C Table 9-19, p. 62. The readings at 30 °C and 105 °C: SLASEO7C Table 9-30, p. 72;
//! their use: SLAU445I 1.13.3.3, p. 60. LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example. In a room at 20 °C to 25 °C, LED1 is on.
//! 2. Hold a fingertip on the MSP430FR2476 (MSP1: SLAU802 Figure 2, p. 4): as the chip warms past 25 °C,
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
    // (WDTHOLD = 1 stops it: SLAU445I Table 12-2, p. 366; after a PUC it runs: SLAU445I 12.2.2, p. 363)
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();
    led.set_low().ok();

    // ADC setup.
    // Temp sensor needs >= 30 us sample time (SLAU445I 21.2.7.8, p. 556: "the sample period must be
    // greater than 30 µs").
    // MODCLK is < ~4.6MHz, so 256 cycles / 4.6 MHz = 55 us sample time (SLASEO7C 8.12.3.6, p. 30:
    // fMODOSC is 4.6 MHz at most).
    // (ADCSHTx = 1000b for 256 ADCCLK cycles: SLAU445I Table 21-3, p. 561; ADCSSELx = 00b is MODCLK:
    // SLAU445I Table 21-4, p. 564; ADCRES = 10b for 12 bits and ADCSR = 0 for up to about 200 ksps:
    // SLAU445I Table 21-5, p. 565)
    let adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits12,
        SamplingRate::Max200ksps,
        SampleTime::Cycles256,
    )
    .use_modclk()
    .configure(periph.adc);

    // REFVSEL = 00b selects 1.5 V, and TSENSOREN = 1 turns the sensor on (SLAU445I Table 2-4, p. 93)
    let vref = pmm.enable_internal_reference(ReferenceVoltage::V1_5).unwrap();
    // The sensor is ADC channel 12 (SLASEO7C Table 9-19, p. 62)
    let mut t_sense = pmm.enable_internal_temp_sensor(&vref).unwrap();

    // The device descriptors (TLV) hold the sensor readings measured in the factory at two temperatures,
    // against the internal 1.5 V reference at full resolution, so measure the same way. This is much more
    // accurate than the typical sensor voltage and slope from the data sheet.
    // (SLASEO7C Table 9-30, p. 72: 1.5-V reference readings at 30°C and 105°C; SLAU445I 1.13.3.3, p. 60.
    // The typical values are VSENSOR and TCSENSOR in SLASEO7C 8.12.5.1, p. 33. The sensor's offset error
    // "can be large and must be calibrated": SLAU445I 21.2.7.8, p. 556.)
    // ADCSREFx = 001b: VR+ = VREF and VR- = AVSS (SLAU445I 21.3.6, p. 567)
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
