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

// Turn on P1.0 if temp between 20 and 25C (LED1, red: SLAU680 Figure 18, p. 26)
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();
    led.set_low().ok();

    // ADC setup.
    // Temp sensor needs >= 30 us sample time (SLAU445I 21.2.7.8, p. 556: the sample period must be
    // greater than 30 us).
    // MODCLK is < ~4.6MHz, so 256 cycles / 4.6 MHz = 55 us sample time (fMODOSC is 4.6 MHz at most:
    // SLASEC4D Table 5-9, p. 41).
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits12,
        SamplingRate::Max200ksps,
        SampleTime::Cycles256,
    )
    .use_modclk()
    .configure(periph.adc);

    let vref = pmm.enable_internal_reference(ReferenceVoltage::V1_5).unwrap();
    let mut t_sense = pmm.enable_internal_temp_sensor(&vref).unwrap();

    loop {
        // Get the voltage of the internal temp sensor, assuming the ADC reference voltage is 3300mV
        let reading_mv = block!( adc.read_voltage_mv(&mut t_sense, 3300) ).unwrap();

        // Equation 11 gives us this equation for calculating temperature from the temp sensor voltage:
        // T = 0.00355 × (V_t – V_30C) + 30C, and V_30C = 788 mV (Table 5-10).
        // Note integer division, so multiply first (beware overflow!), divide last to maximise accuracy
        let temp_celcius = (((355 * (reading_mv as i32 - 788)) + 30_000) / 1000) as i16;

        // Turn on LED if temp between 20 and 25C (LED1 on P1.0: SLAU680 Figure 18, p. 26)
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
