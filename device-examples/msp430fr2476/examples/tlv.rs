//! The device descriptors (TLV): which device this is, and the calibration values measured in the
//! factory. The board checks the descriptors against their CRC and prints them on the backchannel UART.
//! LED1 turns on if the CRC matches. It doesn't print the hardware revision: the data sheet lists it, but
//! "This device does not support reading the hardware revision from memory" (SLAZ726B 5.3, p. 4).
//! (The descriptors: SLASEO7C Table 9-30, p. 71 to p. 72. LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example, with the TXD jumper of J101 on (SLAU802 Table 2, p. 8).
//! 2. In the Windows Device Manager, find the COM port of "MSP Application UART1", and open it at
//!    9600 baud in a serial terminal such as PuTTY (connection type Serial) (SLAU802 2.2.4, p. 9).
//! 3. Press the reset button S3 to print the descriptors again.
//!
//! Expected: `CRC: matches`, and device ID 832A on an MSP430FR2476 or 832B on an MSP430FR2475
//! (SLASEO7C Table 9-29, p. 71). The other values are measured per chip ("Per unit" in SLASEO7C
//! Table 9-30, p. 71 to p. 72), so each board prints its own. Reference factors and the ADC gain factor
//! are close to 32768 (8000h), which means no correction (SLAU445I 1.13.3, p. 59 and SLAU445I
//! 1.13.3.2, p. 60: results are multiplied by the factor and divided by 2^15).
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use embedded_io::Write;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    crc::Crc,
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::{Pmm, ReferenceVoltage},
    serial::*,
    tlv::{self, TempSensorCalibration},
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut led1 = p1.pin0.to_output_low();

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART is eUSCI_A0's TXD on P1.4, P1SEL = 01, in the default mapping (SLAU802
    // 2.2.4, p. 9; SLASEO7C Table 9-23, p. 65; USCIA0RMP = 0: SLAU445I Table 1-32, p. 83). 8N1: LSB
    // first, 8 data bits, one stop bit, no parity (UCMSB, UC7BIT, UCSPB, UCPEN: SLAU445I Table 22-8,
    // p. 593).
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

    // The CRC module checks the descriptors: the CRC-CCITT of 1A04h to 1AF7h, stored at 1A02h
    // (SLASEO7C Table 9-30 note 1, p. 72; SLASEO7C Table 9-30, p. 71)
    let mut crc = Crc::new(periph.crc, 0xFFFF);
    let crc_ok = tlv::crc_matches(&mut crc);

    // The information block, without the hardware revision, and the die record (SLASEO7C Table 9-30, p. 71)
    let die = tlv::die_record();
    writeln!(tx, "\r\nDevice descriptors (TLV)\r").ok();
    writeln!(tx, "Device ID:         {:04X}\r", tlv::device_id()).ok();
    writeln!(tx, "Firmware revision: {:02X}\r", tlv::firmware_revision()).ok();
    writeln!(tx, "Lot wafer ID:      {:08X}\r", die.lot_wafer_id).ok();
    writeln!(tx, "Die position:      X {}, Y {}\r", die.x_position, die.y_position).ok();
    writeln!(tx, "Test result:       {:04X}\r", die.test_result).ok();

    // The ADC calibration (SLASEO7C Table 9-30, p. 72; how to use it: SLAU445I 1.13.3.2, p. 60 and
    // SLAU445I 1.13.3.3, p. 60)
    writeln!(tx, "ADC gain factor:   {:04X}\r", tlv::adc_gain_factor()).ok();
    writeln!(tx, "ADC offset:        {}\r", tlv::adc_offset()).ok();
    for (vref, name) in [
        (ReferenceVoltage::V1_5, "1.5"),
        (ReferenceVoltage::V2_0, "2.0"),
        (ReferenceVoltage::V2_5, "2.5"),
    ] {
        let temp = TempSensorCalibration::new(vref);
        writeln!(
            tx,
            "{} V reference: factor {:04X}, temperature sensor {} at 30 C and {} at {} C\r",
            name,
            tlv::reference_factor(vref),
            temp.count_30c(),
            temp.count_high(),
            temp.high_celsius(),
        )
        .ok();
    }

    // The DCO calibration (SLASEO7C Table 9-30, p. 72; SLAU445I 1.13.3.4, p. 60)
    writeln!(tx, "DCO tap, 16 MHz:   {:04X}\r", tlv::dco_tap_16mhz()).ok();
    writeln!(tx, "CRC: {}\r", if crc_ok { "matches" } else { "DOES NOT MATCH" }).ok();
    tx.flush().ok();

    led1.set_state(crc_ok.into()).ok();

    loop {
        msp430::asm::nop();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
