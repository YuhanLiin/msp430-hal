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
// (P1.0 drives LED1: SLAU802 Figure 19, p. 25)
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
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::_12BIT,
        SamplingRate::_200KSPS,
        SampleTime::_256,
    )
    .use_modclk()
    .configure(periph.adc);

    // REFVSEL = 00b selects 1.5 V, and TSENSOREN = 1 turns the sensor on (SLAU445I Table 2-4, p. 93)
    let vref = pmm.enable_internal_reference(ReferenceVoltage::_1V5).unwrap();
    // The sensor is ADC channel 12 (SLASEO7C Table 9-19, p. 62)
    let mut t_sense = pmm.enable_internal_temp_sensor(&vref).unwrap();

    loop {
        // Get the voltage of the internal temp sensor, assuming the ADC reference voltage is 3300mV
        let reading_mv = block!( adc.read_voltage_mv(&mut t_sense, 3300) ).unwrap();

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
