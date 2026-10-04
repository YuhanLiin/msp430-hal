//! The ADC window comparator, with conversions started by the RTC and handled in the ADC interrupt.
//!
//! The RTC starts a conversion of P4.3 ten times a second. The window comparator sorts each result: below
//! 1.0 V, from 1.0 V to 2.0 V, or above 2.0 V, and the ADC interrupt shows which on LED2: blue below,
//! green inside, red above. The backchannel UART prints the latest voltage twice a second.
//! (RTC overflows trigger the ADC, ADCSHSx = 01b: SLASEO7C 9.10.11, p. 61; SLASEO7C Table 9-20, p. 62.
//! Window comparator: SLAU445I 21.2.7.7, p. 555. ADC interrupts: SLAU445I 21.2.7.10, p. 558. LED2 is red
//! on P5.1, green on P5.0 and blue on P4.7: SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator, or a jumper wire):
//! 1. Generator: the DC waveform, Offset 1.500 V, output load High-Z. Connect it to P4.3 (J3 pin 24), its
//!    ground to GND (J3 pin 22). Check the voltage with the multimeter first: at most 3.3 V.
//! 2. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9). LED2 is green, and the terminal shows about 1500 mV.
//! 3. Set the offset to 0.5 V: LED2 turns blue. Set it to 2.5 V: red.
//! 4. A slow sine wave shows all three in turn: 0.2 Hz, 3 Vpp, offset 1.5 V (0 V to 3 V).
//!
//! Without the generator, a jumper wire from P4.3 to GND (blue) or to 3.3 V on J1 pin 1 (red) works too.
//! (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::{Cell, RefCell};
use critical_section::with;
use embedded_hal::{delay::DelayNs, digital::*};
use embedded_io::Write;
use msp430::interrupt::{enable as enable_interrupts, Mutex};
use msp430_rt::entry;
use msp430_hal::{
    adc::{
        Adc, AdcConfig, AdcInterruptFlags, AdcVector, ClockDivider, ConversionConfig, ConversionMode, Predivider,
        Resolution, SampleMode, SampleTime, SamplingRate, TriggerSource,
    },
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    rtc::{Rtc, RtcDiv},
    serial::*,
    watchdog::Wdt,
};
use msp430fr247x::interrupt;
use panic_msp430 as _;

/// The ADC's reference, AVCC (SLAU802 2.3.1, p. 10: the LaunchPad supplies 3.3 V)
const AVCC_MV: u16 = 3300;
/// The window, in counts of the 12-bit result: 4095 × voltage / 3.3 V (SLAU445I 21.2.1, p. 541)
const LOW: u16 = 1241; // 1.0 V
const HIGH: u16 = 2482; // 2.0 V

static ADC: Mutex<RefCell<Option<Adc>>> = Mutex::new(RefCell::new(None));
/// The window the last result was in
static LAST: Mutex<Cell<AdcVector>> = Mutex::new(Cell::new(AdcVector::None));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    let mut red = p5.pin1.to_output_low();
    let mut green = p5.pin0.to_output_low();
    let mut blue = p4.pin7.to_output_low();

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, aclk, mut delay) = ClockConfig::new(periph.cs)
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

    // P4.3 is input A8 with P4SEL = 11 (SLASEO7C Table 9-26, p. 68). MODCLK clocks the ADC (ADCSSELx =
    // 00b: SLAU445I Table 21-4, p. 564), with 12-bit results (ADCRES = 10b: SLAU445I Table 21-5, p. 565)
    // and 16 ADCCLK cycles of sampling (ADCSHTx = 0010b: SLAU445I Table 21-3, p. 561).
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

    // ADCLO and ADCHI, in the unsigned format of the results (SLAU445I 21.2.7.7, p. 555). Each result sets
    // one of ADCLOIFG, ADCINIFG and ADCHIIFG, and ADCIE requests the interrupt for them (SLAU445I
    // Table 21-13, p. 570).
    adc.set_window(LOW, HIGH);
    adc.enable_interrupts(AdcInterruptFlags::BelowWindow | AdcInterruptFlags::InsideWindow | AdcInterruptFlags::AboveWindow);
    // Repeat-single-channel mode (ADCCONSEQx = 10b), each conversion started by an RTC overflow (ADCSHSx =
    // 01b: SLAU445I Table 21-4, p. 563)
    adc.start(
        &mut input,
        ConversionConfig {
            mode: ConversionMode::RepeatSingle,
            trigger: TriggerSource::Rtc,
            sample_mode: SampleMode::RisingEdge,
            back_to_back: false,
        },
    );
    with(|cs| ADC.borrow_ref_mut(cs).replace(adc));

    // The RTC counts ACLK, from REFO at 32.768 kHz, and overflows every 3277 counts: 10 times a second
    // (RTCSS = 01b: SLAU445I Table 15-2, p. 420; RTCMOD: SLAU445I 15.2.1, p. 417)
    let mut rtc = Rtc::new(periph.rtc).use_aclk(&aclk);
    rtc.set_clk_div(RtcDiv::_1);
    rtc.start(3276);

    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { enable_interrupts() };

    loop {
        let (last, count) = with(|cs| {
            let count = ADC.borrow_ref(cs).as_ref().map_or(0, |adc| adc.adc_get_result());
            (LAST.borrow(cs).get(), count)
        });
        red.set_state((last == AdcVector::AboveWindow).into()).ok();
        green.set_state((last == AdcVector::InsideWindow).into()).ok();
        blue.set_state((last == AdcVector::BelowWindow).into()).ok();

        // count × 3300 / 4095, as `Adc::count_to_mv` works it out
        let mv = (count as u32 * AVCC_MV as u32 / 4095) as u16;
        writeln!(tx, "{} mV: {:?}\r", mv, last).ok();
        delay.delay_ms(500);
    }
}

// The ADC vector (FFD8h: SLASEO7C Table 9-2, p. 47). Reading ADCIV returns the highest-priority pending
// flag and clears it (SLAU445I 21.2.7.10.1, p. 558).
#[interrupt]
fn ADC() {
    with(|cs| {
        if let Some(adc) = ADC.borrow_ref_mut(cs).as_mut() {
            loop {
                match adc.interrupt_source() {
                    AdcVector::None => break,
                    window @ (AdcVector::BelowWindow | AdcVector::InsideWindow | AdcVector::AboveWindow) => {
                        LAST.borrow(cs).set(window)
                    }
                    _ => {}
                }
            }
        }
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
