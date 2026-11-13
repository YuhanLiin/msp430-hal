//! An ADC sequence: one start converts A3, A2, A1 and A0 in turn, with results in the signed format. The
//! backchannel UART prints the four results once a second, raw and in millivolts.
//!
//! The inputs on this LaunchPad:
//! - A3, P1.3 (J1 pin 9): the function generator, for example 1.000 V
//! - A2, P1.2 (J1 pin 10): a jumper wire to 3.3 V (J1 pin 1)
//! - A1, P1.1: the TMP235 temperature sensor, whose voltage rises with temperature
//! - A0, P1.0 (J3 pin 27): a jumper wire to GND (J3 pin 22). This is LED1's pin, which otherwise
//!   floats as an analog input (it read about 1.2 V on an MSP430FR2476 LaunchPad)
//!
//! In the signed format the 12-bit result is a two's complement number in the top 12 bits: the bottom of
//! the range, 0 V, reads as -32768 (8000h) and the reference, 3.3 V here, as 32752 (7FF0h). `Adc::count_to_mv` converts
//! these results too.
//!
//! How to test (function generator, a jumper wire, multimeter):
//! 1. Generator: the DC waveform, Offset 1.000 V, output load High-Z, on P1.3 (J1 pin 9), its ground on
//!    GND (J3 pin 22). Check the voltage with the multimeter first: at most 3.3 V.
//! 2. Connect P1.2 (J1 pin 10) to 3.3 V (J1 pin 1) and P1.0 (J3 pin 27) to GND (J3 pin 22) with jumper
//!    wires. Keep the J9 jumper on.
//! 3. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud.
//! 4. Expected, once a second: `A3: -12912 = 1000 mV, A2: 32752 = 3300 mV, A1: ..., A0: -32768 = 0 mV`,
//!    give or take a few millivolts. A1 is the sensor: 718 mV at room temperature on the LaunchPad this
//!    was written on. Touch the TMP235 and watch A1 rise.
#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_io::Write;
use msp430_rt::entry;
use msp430_hal::{
    adc::{
        AdcConfig, AdcInterruptFlags, ClockDivider, ConversionConfig, ConversionMode, DataFormat, Predivider,
        Resolution, SampleMode, SampleTime, SamplingRate, TriggerSource,
    },
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

/// The ADC's reference, AVCC (the LaunchPad supplies 3.3 V)
const AVCC_MV: u16 = 3300;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:; DIVM,
    // DIVS)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART: eUSCI_A0's TXD on P1.4, P1SEL = 01, 8N1
    let mut tx = SerialConfig::<_, _, DefaultMapping>::new(
        periph.e_usci_a0,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        9600,
    )
    .use_smclk(&smclk)
    .tx_only(p1.pin4.to_alternate1());

    // A sequence converts every channel from the selected one down to A0, so all four pins are analog
    // inputs, PxSEL = 11
    let mut a3 = p1.pin3.to_alternate3();
    let _a2 = p1.pin2.to_alternate3();
    let _a1 = p1.pin1.to_alternate3();
    let _a0 = p1.pin0.to_alternate3();

    // MODCLK clocks the ADC (ADCSSELx = 00b), with 12-bit results (ADCRES = 10b) in the signed format
    // (ADCDF = 1). 256 ADCCLK cycles of sampling (ADCSHTx = 1000b), about 67 µs at MODCLK's 3.8 MHz,
    // give the sensor's output filter time to charge the ADC, and the CPU time to read each result.
    let mut config = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::_12BIT,
        SamplingRate::_200KSPS,
        SampleTime::_256,
    );
    config.data_format = DataFormat::Signed;
    let mut adc = config.use_modclk().configure(periph.adc);

    // Sequence-of-channels mode (ADCCONSEQx = 01b), started once by software (ADCSC). ADCMSC = 1
    // converts the rest of the sequence without another start.
    let sequence = ConversionConfig {
        mode: ConversionMode::Sequence,
        trigger: TriggerSource::Software,
        sample_mode: SampleMode::RisingEdge,
        back_to_back: true,
    };

    loop {
        adc.start(&mut a3, sequence);
        let mut results = [0u16; 4];
        for result in results.iter_mut() {
            *result = block!(adc.result()).unwrap();
        }
        // A result that came before the one before it was read sets ADCOVIFG
        if adc.interrupt_flags().contains(AdcInterruptFlags::Overflow) {
            adc.clear_interrupt_flags(AdcInterruptFlags::Overflow);
            writeln!(tx, "missed a result\r").ok();
        }

        for (channel, result) in (0..4).rev().zip(results) {
            write!(tx, "A{}: {} = {} mV", channel, result as i16, adc.count_to_mv(result, AVCC_MV)).ok();
            write!(tx, "{}", if channel == 0 { "\r\n" } else { ", " }).ok();
        }
        delay.delay_ms(1000);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
