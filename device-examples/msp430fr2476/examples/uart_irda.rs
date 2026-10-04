//! IrDA encoding: eUSCI_A1 sends each 0 bit as a short pulse, 3/16 of a bit time, as an infrared
//! transceiver needs, and decodes such pulses when receiving. In loopback mode it receives what it sends,
//! and the backchannel UART prints what came back.
//! (IrDA encoding and decoding: SLAU445I 22.3.5, p. 581. Loopback, UCLISTEN: SLAU445I 22.4.5, p. 596.)
//!
//! How to test (scope):
//! 1. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9). Expected, five times a second: `received: IrDA`.
//! 2. Probe eUSCI_A1's TXD, P2.6 (J1 pin 4), ground clip on GND (J3 pin 22). Trigger on the rising edge,
//!    200 µs/div. Instead of a normal UART signal, which is high when idle, the line is low, with short
//!    high pulses of about 20 µs: one for each 0 bit, 104 µs apart at 9600 baud. The start bit is a 0, so
//!    each character begins with a pulse. (SLAU445I Figure 22-7, p. 581. Header pins: SLAU802 Figure 10,
//!    p. 13.)
#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal_nb::serial::Read;
use embedded_io::Write;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use panic_msp430 as _;

const MESSAGE: &[u8] = b"IrDA";

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART: eUSCI_A0's TXD on P1.4, P1SEL = 01, 8N1 (SLAU802 2.2.4, p. 9; SLASEO7C
    // Table 9-23, p. 65; SLAU445I Table 22-8, p. 593)
    let mut console = SerialConfig::<_, _, DefaultMapping>::new(
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

    // eUSCI_A1 with IrDA encoding: pulses of 3/16 of a bit time, counted in BITCLK16 (UCIRTXPLx = 5,
    // UCIRTXCLK = 1: SLAU445I 22.3.5.1, p. 581), in loopback mode. 1 MHz is more than 16 × 9600 baud, so
    // the baud-rate generator oversamples, which BITCLK16 needs (SLAU445I Table 22-16, p. 599). Its pins
    // are P2.6 (TXD) and P2.5 (RXD), with P2SEL = 01 (SLASEO7C Table 9-24, p. 66).
    let (mut tx, mut rx) = SerialConfig::new(
        periph.e_usci_a1,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::Loopback,
        9600,
    )
    .irda(IrdaConfig::standard())
    .use_smclk(&smclk)
    .split(p2.pin6.to_alternate1(), p2.pin5.to_alternate1());

    loop {
        write!(console, "received: ").ok();
        for &byte in MESSAGE {
            tx.write_all(&[byte]).ok();
            // Each character arrives back before the next is sent
            match nb::block!(rx.read()) {
                Ok(received) => console.write_all(&[received]).ok(),
                Err(error) => write!(console, "<{:?}>", error).ok(),
            };
        }
        writeln!(console, "\r").ok();
        delay.delay_ms(200);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
