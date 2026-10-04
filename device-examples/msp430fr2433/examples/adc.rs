#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, Predivider, Resolution, SampleTime, SamplingRate}, gpio::Batch, pmm::Pmm, watchdog::Wdt
};
use nb::block;
use panic_msp430 as _;

// If pin 1.2 is between 1V and 2V, the LED on pin 1.0 should light up.
// P1.2 is ADC input A2 (SLASE59F Table 6-15, p. 53). On the LaunchPad it is header pin J1.10, and P1.0
// drives the red LED1 (both: SLAU739 Figure 18, p. 23).
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
