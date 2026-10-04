//! UART breaks and the start-bit and transmit-complete interrupts, on eUSCI_A1 in loopback mode.
//!
//! Once a second eUSCI_A1 sends `Hi` and then a break: a character time with all bits low. Its receiver,
//! set up to report breaks, reads 'H', 'i' and then `RecvError::Break`. Meanwhile its interrupt counts
//! the start bits it receives and the characters it finishes sending, with `interrupt_source()`. The
//! backchannel UART prints all of it.
//! (Breaks: SLAU445I 22.3.3.2.1, p. 579. UCSTTIFG and UCTXCPTIFG: SLAU445I Table 22-6, p. 591. Loopback,
//! UCLISTEN: SLAU445I 22.4.5, p. 596.)
//!
//! How to test (optionally the scope):
//! 1. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9).
//! 2. Expected, once a second: `received H i <break>, start bits: 3, characters sent: 3`.
//! 3. With the scope on eUSCI_A1's TXD, P2.6 (J1 pin 4), ground on GND (J3 pin 22): two characters, then
//!    the line low for about 1 ms, the break. (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::{Cell, RefCell};
use critical_section::with;
use embedded_hal::delay::DelayNs;
use embedded_hal_nb::serial::Read;
use embedded_io::Write;
use msp430::interrupt::{enable as enable_interrupts, Mutex};
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
use msp430fr247x::{interrupt, EUsciA1};
use nb::block;
use panic_msp430 as _;

static RX: Mutex<RefCell<Option<Rx<EUsciA1>>>> = Mutex::new(RefCell::new(None));
/// Start bits received and characters sent since the last report
static COUNTS: Mutex<Cell<(u16, u16)>> = Mutex::new(Cell::new((0, 0)));

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

    // eUSCI_A1 in loopback mode, reporting breaks (UCBRKIE: SLAU445I Table 22-8, p. 593), and ignoring
    // pulses on RXD shorter than about 50 ns (UCGLITx = 01b: SLAU445I Table 22-9, p. 594). Its pins are P2.6
    // (TXD) and P2.5 (RXD), with P2SEL = 01 (SLASEO7C Table 9-24, p. 66).
    let (mut tx, mut rx) = SerialConfig::new(
        periph.e_usci_a1,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::Loopback,
        9600,
    )
    .break_interrupts()
    .deglitch(UartDeglitch::_50ns)
    .use_smclk(&smclk)
    .split(p2.pin6.to_alternate1(), p2.pin5.to_alternate1());

    // UCSTTIE and UCTXCPTIE (SLAU445I Table 22-17, p. 600). Set GIE, which masks every maskable interrupt
    // while clear (SLAU445I 1.3.3, p. 33).
    rx.enable_start_bit_interrupts();
    tx.enable_tx_complete_interrupts();
    with(|cs| RX.borrow_ref_mut(cs).replace(rx));
    unsafe { enable_interrupts() };

    loop {
        write!(console, "received").ok();
        for &byte in b"Hi" {
            tx.write_all(&[byte]).ok();
            report(&mut console, read());
        }
        // All bits low for a character time (UCTXBRK: SLAU445I 22.3.3.2.1, p. 579)
        block!(tx.send_break()).ok();
        report(&mut console, read());

        let (start_bits, sent) = with(|cs| COUNTS.borrow(cs).replace((0, 0)));
        writeln!(console, ", start bits: {}, characters sent: {}\r", start_bits, sent).ok();
        delay.delay_ms(1000);
    }
}

/// Wait for the next character, or error, from eUSCI_A1's receiver
fn read() -> Result<u8, RecvError> {
    loop {
        let result = with(|cs| RX.borrow_ref_mut(cs).as_mut().unwrap().read());
        match result {
            Err(nb::Error::WouldBlock) => continue,
            Err(nb::Error::Other(error)) => return Err(error),
            Ok(byte) => return Ok(byte),
        }
    }
}

fn report(console: &mut impl Write, result: Result<u8, RecvError>) {
    match result {
        Ok(byte) => write!(console, " {}", byte as char).ok(),
        Err(RecvError::Break) => write!(console, " <break>").ok(),
        Err(error) => write!(console, " <{:?}>", error).ok(),
    };
}

// The eUSCI_A1 vector (FFDEh: SLASEO7C Table 9-2, p. 46). `interrupt_source()` clears UCSTTIFG and
// UCTXCPTIFG, as a UCA1IV read does (SLAU445I 22.3.15.4, p. 591).
#[interrupt]
fn EUSCI_A1() {
    with(|cs| {
        if let Some(rx) = RX.borrow_ref_mut(cs).as_mut() {
            let counts = COUNTS.borrow(cs);
            loop {
                let (start_bits, sent) = counts.get();
                match rx.interrupt_source() {
                    UartVector::StartBit => counts.set((start_bits + 1, sent)),
                    UartVector::TxComplete => counts.set((start_bits, sent + 1)),
                    _ => break,
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
