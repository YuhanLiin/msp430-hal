//! A polled ADC reading: LED1 is on while the voltage on P1.2 is between 1.0 V and 2.0 V, and off otherwise.
//!
//! The ADC converts P1.2, input A2, again and again, with 8-bit results against AVCC, the LaunchPad's
//! 3.3 V supply, and `read_voltage_mv()` turns each result into millivolts.
//! (A2 is P1.2: SLASE59F Table 6-15, p. 53. AVCC is the reference after reset: SLAU445I Table 21-8, p. 567.
//! The supply: SLAU739 2.3.1, p. 10. LED1 on P1.0 is red: SLAU739 Figure 18, p. 23.)
//!
//! How to test (function generator, or a jumper wire):
//! 1. Generator: the DC waveform, Offset 1.500 V, output load High-Z. Check the voltage with the multimeter
//!    first: 0 V to 3.3 V only (the analog input range: SLASE59F Table 5-20, p. 35). Connect it to P1.2
//!    (J1 pin 10), its ground to GND (J2 pin 20).
//! 2. Flash this example: LED1 is on.
//! 3. Set the offset to 0.5 V, and then to 2.5 V: LED1 is off at both. It's on from about 1.0 V to 2.0 V.
//!
//! Without the generator, a jumper wire from P1.2 to GND (J2 pin 20) or to 3.3 V (J1 pin 1) turns LED1
//! off, and a potentiometer of about 10 kΩ between 3.3 V and GND, its wiper on P1.2, turns it on in the
//! middle of its range. (Header pins: SLAU739 Figure 18, p. 23.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, Predivider, Resolution, SampleTime, SamplingRate}, gpio::Batch, pmm::Pmm, watchdog::Wdt
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
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut red_led = port1.pin0.to_output();
    red_led.set_low().ok();
    let mut adc_pin = port1.pin2.to_adc_mode(); // A2: ADCPCTL2 = 1 (SLASE59F Table 6-17, p. 55)

    // ADC setup
    // ADCCLK = MODCLK undivided (ADCSSELx = 00b, ADCDIVx = 000b: SLAU445I Table 21-4, p. 563 to p. 564;
    // ADCPDIVx = 00b: SLAU445I Table 21-5, p. 565), 8-bit results (ADCRES = 00b) with the 50-ksps buffer
    // (ADCSR = 1) (SLAU445I Table 21-5, p. 565), 16-cycle samples (ADCSHTx = 0010b, SLAU445I Table 21-3,
    // p. 561). MODCLK runs at up to 5.8 MHz (SLASE59F Table 5-9, p. 26), so a sample lasts at least
    // 16 / 5.8 MHz = 2.76 us, more than the tSample of 1.5 us at 2 V and 2.0 us at 3 V that SLASE59F
    // Table 5-21, p. 35 gives for a 1-kOhm source (10 bits; its note 2 needs less for 8 bits).
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits8,
        SamplingRate::Max50ksps,
        SampleTime::Cycles16,
    )
    .use_modclk()
    .configure(periph.adc);

    loop {
        // Get ADC voltage, assuming the ADC reference voltage is 3300mV
        // (the reference is AVCC after reset, SLAU445I 21.3.6, Table 21-8, p. 567, which the LaunchPad
        // supplies at 3.3 V, SLAU739 2.3.1, p. 10)
        // It's infallible besides nb::WouldBlock, so it's safe to unwrap after block!()
        // If you want a raw count use adc.read_count() instead.
        let reading_mv = block!( adc.read_voltage_mv(&mut adc_pin, 3300) ).unwrap();

        // Turn on LED if voltage between 1000 and 2000mV
        if (1000..=2000).contains(&reading_mv) {
            red_led.set_high().ok();
        } else {
            red_led.set_low().ok();
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
