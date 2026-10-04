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

// If pin 4.3 is between 1V and 2V, the LED on pin 1.0 should light up.
// LED1 is on P1.0 (SLAU802 Figure 19, p. 25). P4.3 is J3 pin 24 (SLAU802 Figure 10, p. 13), an analog
// header pin wired to nothing else on the LaunchPad (net P4.3_A8: SLAU802 Figure 18, p. 24). P1.1 is
// not used, because the TMP235 temperature sensor drives it (SLAU802 2.2.5.1, p. 10).
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    // (WDTHOLD = 1 stops it: SLAU445I Table 12-2, p. 366; after a PUC it runs: SLAU445I 12.2.2, p. 363)
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let port4 = Batch::new(periph.p4).split(&pmm);
    let mut led = port1.pin0.to_output();
    // P4.3 = analog input A8 with P4SEL = 11 (SLASEO7C Table 9-26, p. 68), ADC channel 8
    // (SLASEO7C Table 9-19, p. 62)
    let mut adc_pin = port4.pin3.to_alternate3();

    // ADC setup
    // (ADCDIVx: SLAU445I Table 21-4, p. 563; ADCSSELx = 00b is MODCLK: SLAU445I Table 21-4, p. 564;
    // ADCPDIVx, ADCRES = 00b for 8 bits and ADCSR = 1 for up to about 50 ksps: SLAU445I Table 21-5,
    // p. 565; ADCSHTx = 0000b for 4 ADCCLK cycles: SLAU445I Table 21-3, p. 561)
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::_8BIT,
        SamplingRate::_50KSPS,
        SampleTime::_4,
    )
    .use_modclk()
    .configure(periph.adc);

    loop {
        // Get ADC voltage, assuming the ADC reference voltage is 3300mV
        // (the ADC measures against AVCC after reset, ADCSREFx = 000b: SLAU445I 21.3.6, p. 567; the
        // LaunchPad supplies the MSP430 with 3.3 V: SLAU802 2.3.1, p. 10)
        // It's infallible besides nb::WouldBlock, so it's safe to unwrap after block!()
        // If you want a raw count use adc.read_count() instead.
        let reading_mv = block!(adc.read_voltage_mv(&mut adc_pin, 3300)).unwrap();

        // Turn on LED if voltage between 1000 and 2000mV
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
