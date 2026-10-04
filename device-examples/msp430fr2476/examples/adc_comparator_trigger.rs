//! ADC conversions started by the comparator: eCOMP0 compares P2.2 with its 1.2 V reference, and each
//! time the input rises through 1.2 V, its output starts a conversion of P4.3. With the same signal on
//! both pins, the ADC samples it just as it crosses 1.2 V, so every result is close to 1.2 V. Once a
//! second the backchannel UART prints how many conversions there were and their average.
//! (eCOMP0's output is the ADC's comparator trigger, ADCSHSx = 11b: SLASEO7C Table 9-20, p. 62. COMP0.1
//! is P2.2: SLASEO7C Table 9-21, p. 63. A8 is P4.3: SLASEO7C Table 9-19, p. 62. Header pins: SLAU802
//! Figure 10, p. 13.)
//!
//! How to test (function generator):
//! 1. Generator: sine wave, 10 Hz, amplitude 2 Vpp, offset 1.5 V (0.5 V to 2.5 V), output load High-Z.
//!    Check it on the scope first: it must stay between 0 V and 3.3 V.
//! 2. Connect it to both P2.2 (J1 pin 5) and P4.3 (J3 pin 24), with the BNC T-piece and two leads, its
//!    ground to GND (J3 pin 22).
//! 3. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9).
//! 4. Expected: `10 conversions, average 1200 mV` once a second, give or take a few tens of millivolts:
//!    the 1.2 V reference is typical, and the input moves on a little during the sampling. Change the
//!    frequency and the count follows. Change the offset between 1.0 V and 2.0 V: the average stays near
//!    1.2 V. At an offset of 2.3 V the sine stays above 1.2 V (1.3 V to 3.3 V), and the count drops to 0.
//!    Never let the signal go below 0 V or above 3.3 V.
//! (The low-power 1.2 V reference, VeCOMP,LP: 1.20 V typical: SLASEO7C 8.12.5.1, p. 33.)
#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_io::Write;
use msp430_rt::entry;
use msp430_hal::{
    adc::{AdcConfig, ClockDivider, ConversionConfig, ConversionMode, Predivider, Resolution, SampleMode, SampleTime, SamplingRate, TriggerSource},
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    ecomp::{ECompConfig, FilterStrength, Hysteresis, NegativeInput, OutputPolarity, PositiveInput, PowerMode},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use panic_msp430 as _;

/// The ADC's reference, AVCC (SLAU802 2.3.1, p. 10: the LaunchPad supplies 3.3 V)
const AVCC_MV: u16 = 3300;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART: eUSCI_A0's TXD on P1.4, P1SEL = 01, 8N1 (SLAU802 2.2.4, p. 9; SLASEO7C
    // Table 9-23, p. 65; SLAU445I Table 22-8, p. 593)
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

    // V+ is COMP0.1 on P2.2, with P2SEL = 11 (SLASEO7C Table 9-24, p. 66), and V- the low-power 1.2 V
    // reference, so the output rises as P2.2 rises through 1.2 V (SLAU445I 18.2.1, p. 505). High-speed
    // mode (CPMSEL = 0), with 20 mV of hysteresis against noise (CPHSEL = 10b): SLAU445I Table 18-3,
    // p. 510.
    let (_dac_config, comparator_config) = ECompConfig::begin(periph.e_comp0);
    let _comparator = comparator_config
        .configure(
            PositiveInput::COMPx_1(p2.pin2.to_alternate3()),
            NegativeInput::_1V2,
            OutputPolarity::Noninverted,
            PowerMode::HighSpeed,
            Hysteresis::_20mV,
            FilterStrength::Off,
        )
        .no_output_pin();

    // P4.3 is input A8 with P4SEL = 11 (SLASEO7C Table 9-26, p. 68). MODCLK clocks the ADC (ADCSSELx =
    // 00b: SLAU445I Table 21-4, p. 564), with 12-bit results (ADCRES = 10b: SLAU445I Table 21-5, p. 565)
    // and 16 ADCCLK cycles of sampling (ADCSHTx = 0010b: SLAU445I Table 21-3, p. 561), about 4 µs.
    let mut input = p4.pin3.to_alternate3();
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::Bits12,
        SamplingRate::Max200ksps,
        SampleTime::Cycles16,
    )
    .use_modclk()
    .configure(periph.adc);

    // Repeat-single-channel mode (ADCCONSEQx = 10b), each conversion started by a rising edge of the
    // comparator's output (ADCSHSx = 11b, ADCSHP = 1: SLAU445I Table 21-4, p. 563)
    adc.start(
        &mut input,
        ConversionConfig {
            mode: ConversionMode::RepeatSingle,
            trigger: TriggerSource::Comparator,
            sample_mode: SampleMode::RisingEdge,
            back_to_back: false,
        },
    );

    loop {
        // Collect the results of one second, in steps of 1 ms
        let (mut conversions, mut sum) = (0u16, 0u32);
        for _ in 0..1000 {
            if let Ok(count) = adc.result() {
                conversions += 1;
                sum += count as u32;
            }
            delay.delay_ms(1);
        }

        if conversions == 0 {
            writeln!(tx, "0 conversions\r").ok();
        } else {
            let average = (sum / conversions as u32) as u16;
            writeln!(tx, "{} conversions, average {} mV\r", conversions, adc.count_to_mv(average, AVCC_MV)).ok();
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
