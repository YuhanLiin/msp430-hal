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
// P1.0 drives the red LED1 (SLAU739 Figure 18, p. 23).
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    // (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2, p. 363)
    let periph = msp430fr2433::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.watchdog_timer);

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
    // MODCLK in active mode also avoids SLAZ664S ADC50, which makes temperature sensor results wrong
    // with ACLK as the ADC clock in LPM3.
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::_10BIT,
        SamplingRate::_200KSPS,
        SampleTime::_256,
    )
    .use_modclk()
    .configure(periph.adc);

    // The 1.5-V reference: REFVSEL = 00b and INTREFEN = 1 in PMMCTL2 (SLAU445I Table 2-4, p. 93 to p. 94).
    // The sensor: TSENSOREN in PMMCTL2 "must be set to turn on the sensor" (SLAU445I 2.2.9, p. 89), and it
    // is ADC channel 12 (SLASE59F Table 6-15, p. 53).
    let vref = pmm.enable_internal_reference(ReferenceVoltage::_1V5).unwrap();
    let mut t_sense = pmm.enable_internal_temp_sensor(&vref).unwrap();

    loop {
        // Get the voltage of the internal temp sensor, assuming the ADC reference voltage is 3300mV
        let reading_mv = block!( adc.read_voltage_mv(&mut t_sense, 3300) ).unwrap();

        // Using the equation and values in datasheet Table 5-22:
        // (Temp in C) = (Vadc - VSENSOR) / TCSENSOR
        let temp_celcius = (reading_mv as i32 - 917) * 100 / 335;

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
