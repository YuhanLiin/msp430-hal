#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, Predivider, Resolution, SampleTime, SamplingRate},
    gpio::Batch,
    pmm::{Pmm, ReferenceVoltage},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

// Turn on P1.0 if temp between 20 and 25C
// No board document covers an LED on P1.0: there is none for the MSP430FR25x2. P1.0 is a GPIO output,
// P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).
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
    let mut adc = AdcConfig::new(
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

    loop {
        // Get the voltage of the internal temp sensor, assuming the ADC reference voltage is 3300mV
        let reading_mv = block!(adc.read_voltage_mv(&mut t_sense, 3300)).unwrap();

        // Equation 11 gives us this equation for calculating temperature from the temp sensor voltage:
        // T = 0.00355 × (V_t – V_30C) + 30C, and V_30C = 788 mV (8.12.5.1).
        // Note integer division, so multiply first (beware overflow!), divide last to maximise accuracy
        let temp_celcius = (((355 * (reading_mv as i32 - 788)) + 30_000) / 1000) as i16;

        // Turn on LED if temp between 20 and 25C
        if (20..=25).contains(&temp_celcius) {
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
