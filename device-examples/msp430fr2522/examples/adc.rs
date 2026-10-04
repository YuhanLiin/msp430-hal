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

// If pin 1.1 is between 1V and 2V, the LED on pin 1.0 should light up.
// No board document covers this LED: there is none for the MSP430FR25x2. P1.1 is ADC input A1
// (SLASEE4C Table 6-13, p. 55); P1.0 is a GPIO output, P1SELx = 00 and P1DIR = 1
// (SLASEE4C Table 6-15, p. 58).
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog. The watchdog runs from every PUC and must be halted, here
    // with WDTHOLD (SLAU445I 12.2.2, p. 363; SLAU445I Table 12-2, p. 366).
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO. Pmm::new clears LOCKLPM5, so the pins take on their configuration
    // (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();
    // Analog inputs are enabled through SYSCFG2.ADCPCTLx on this device. P1.1 is A1, enabled by
    // ADCPCTL1 = 1 (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-13, p. 55).
    let mut adc_pin = port1.pin1.to_adc_mode();

    // ADC setup: 16-cycle samples (ADCSHTx = 0010b, SLAU445I Table 21-3, p. 561). MODCLK runs at up to
    // 5.8 MHz (SLASEE4C Table 5-9, p. 28), so a sample lasts at least 16 / 5.8 MHz = 2.76 us, more than the
    // 2.0 us tSample at 3 V for a 1-kOhm source (SLASEE4C Table 5-21, p. 38). 4 cycles would be 0.69 us.
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
        // (the reference is AVCC, ADCSREFx = 000b: SLAU445I Table 21-8, p. 567; DVCC "supplies digital and
        // analog modules": SLASEE4C 1.4, p. 4)
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
