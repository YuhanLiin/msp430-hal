//! The 1.2 V reference on the VREF+ pin, P1.4. The board measures the pin with its ADC against the
//! internal 2.5 V reference, and turns LED1 on while the result is within the data sheet's 1.16 V to
//! 1.24 V. A multimeter checks it independently.
//! (VREF+ is P1.4 with P1SEL = 11: SLASEO7C Table 9-23, p. 65. Its voltage: SLASEO7C 8.12.5.1, p. 33.
//! LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! P1.4 is also the backchannel UART's TXD, which only reaches the J101 jumper block, not the headers
//! (SLAU802 Table 2, p. 8; SLAU802 Figure 10, p. 13). So this example prints nothing.
//!
//! How to test (multimeter):
//! 1. Remove the TXD jumper from J101, so the debug probe's UART input doesn't load the pin.
//! 2. Flash this example. LED1 should turn on.
//! 3. Set the multimeter to DC volts. Put the black probe on GND (J3 pin 22) and the red probe on each of
//!    the two TXD pins of J101 in turn (GND on the header: SLAU802 Figure 10, p. 13).
//!
//! Expected: one TXD pin, the MSP430 side, reads 1.16 V to 1.24 V (1.20 V typical). Put the jumper back
//! afterwards, for the examples that print.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, NegativeReference, PositiveReference, Predivider, Resolution, SampleTime, SamplingRate},
    gpio::Batch,
    pmm::{Pmm, ReferenceVoltage},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

/// The data sheet's range for VREF+ (SLASEO7C 8.12.5.1, p. 33: 1.16 V min, 1.24 V max)
const VREF_MIN_MV: u16 = 1160;
const VREF_MAX_MV: u16 = 1240;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (mut pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut led1 = p1.pin0.to_output_low();

    // EXTREFEN buffers the 1.2 V bandgap onto VREF+ (SLASEO7C 9.10.1, p. 49; SLAU445I Table 2-4, p. 94)
    let mut vref_out = pmm.enable_vref_output(p1.pin4.to_alternate3());

    // The ADC measures the pin, which is input A4 (SLASEO7C Table 9-19, p. 62), against the internal 2.5 V
    // reference, more precise than the supply (2.5 V ±1.5 %: SLASEO7C 8.12.5.1, p. 33). MODCLK clocks it
    // (ADCSSELx = 00b: SLAU445I Table 21-4, p. 564), with 12-bit results (ADCRES = 10b: SLAU445I
    // Table 21-5, p. 565) and 16 ADCCLK cycles of sampling (ADCSHTx = 0010b: SLAU445I Table 21-3, p. 561).
    let vref = pmm.enable_internal_reference(ReferenceVoltage::_2V5).unwrap();
    let adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::_12BIT,
        SamplingRate::_200KSPS,
        SampleTime::_16,
    )
    .use_modclk()
    .configure(periph.adc);
    // ADCSREFx = 001b: VR+ = VREF and VR- = AVSS (SLAU445I Table 21-8, p. 567)
    let mut adc = adc.with_reference(PositiveReference::Internal(&vref), NegativeReference::Avss);

    loop {
        let count = block!(adc.read_count(&mut vref_out)).unwrap();
        let mv = adc.count_to_mv(count, 2500);
        led1.set_state((VREF_MIN_MV..=VREF_MAX_MV).contains(&mv).into()).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
