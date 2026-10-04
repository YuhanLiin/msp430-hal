//! The UART address-bit multiprocessor format: one transmitter addresses several receivers on a shared
//! line, and each receiver only wakes up for the characters sent to its own address.
//!
//! eUSCI_A1 plays both parts, in loopback mode: it sends a message to address 12h (this receiver) and
//! one to address 34h (another receiver). The receiver sleeps in the dormant state, where it only takes
//! address characters. After its own address it leaves that state and takes the data that follows;
//! after another address it goes back to sleep, so it never sees the data for address 34h. The
//! backchannel UART prints what the receiver got.
//! (The address-bit format: SLAU445I 22.3.3.2, p. 579. UCDORM: SLAU445I Table 22-8, p. 594. Loopback,
//! UCLISTEN: SLAU445I 22.4.5, p. 596.)
//!
//! How to test:
//! 1. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9).
//! 2. Expected, once a second: `[address 12] Hello`, then `[address 34, not mine]`, and no `World`.
//! 3. Optional, with the scope, ground on GND (J3 pin 22): eUSCI_A1's TXD, P2.6 (J1 pin 4), still carries
//!    what it sends. Each character has 9 bits between start and stop bit: 8 data bits and the address
//!    bit, which is 1 for the addresses and 0 for the data. (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
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
use nb::block;
use panic_msp430 as _;

/// This receiver's address
const MY_ADDRESS: u8 = 0x12;
/// The other receiver's address
const OTHER_ADDRESS: u8 = 0x34;

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

    // eUSCI_A1 in the address-bit multiprocessor format (UCMODEx = 10b: SLAU445I Table 22-8, p. 593), in
    // loopback mode: its transmitter feeds its receiver. Its pins are P2.6 (TXD) and P2.5 (RXD), with
    // P2SEL = 01 (SLASEO7C Table 9-24, p. 66).
    let (mut tx, mut rx) = SerialConfig::new(
        periph.e_usci_a1,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::Loopback,
        9600,
    )
    .mode(UartMode::AddressBitMultiprocessor)
    .use_smclk(&smclk)
    .split(p2.pin6.to_alternate1(), p2.pin5.to_alternate1());

    // Wait for an address (UCDORM = 1: "Only characters that are preceded by an idle-line or with address
    // bit set" are received: SLAU445I Table 22-8, p. 594)
    rx.set_dormant(true);

    loop {
        writeln!(console, "\r").ok();
        send(&mut tx, &mut rx, &mut console, &mut delay, true, MY_ADDRESS);
        for &byte in b"Hello" {
            send(&mut tx, &mut rx, &mut console, &mut delay, false, byte);
        }
        send(&mut tx, &mut rx, &mut console, &mut delay, true, OTHER_ADDRESS);
        for &byte in b"World" {
            send(&mut tx, &mut rx, &mut console, &mut delay, false, byte);
        }
        delay.delay_ms(1000);
    }
}

/// Send an address or data character, then give the receiver its turn
fn send<USCI: SerialUsci>(
    tx: &mut Tx<USCI>,
    rx: &mut Rx<USCI>,
    console: &mut impl Write,
    delay: &mut impl DelayNs,
    address: bool,
    byte: u8,
) {
    if address {
        // UCTXADDR marks the next character as an address: its address bit is 1 (SLAU445I 22.3.3.2,
        // p. 579)
        block!(tx.send_address(byte)).ok();
    } else {
        tx.write_all(&[byte]).ok();
    }
    // A character of 11 bits takes 1.15 ms at 9600 baud. In the dormant state a data character never
    // arrives, so this waits a fixed time instead of blocking.
    delay.delay_ms(2);
    receive(rx, console);
}

/// What the receiver does with a character, if one has arrived
fn receive<USCI: SerialUsci>(rx: &mut Rx<USCI>, console: &mut impl Write) {
    // UCADDR shows whether the character had its address bit set (SLAU445I Table 22-12, p. 596)
    let Ok((byte, is_address)) = rx.read_with_address_flag() else { return };
    if !is_address {
        console.write_all(&[byte]).ok();
    } else if byte == MY_ADDRESS {
        // "user software can validate the address and must reset UCDORM to continue receiving data"
        // (SLAU445I 22.3.3.2, p. 579)
        rx.set_dormant(false);
        write!(console, "[address {:02X}] ", byte).ok();
    } else {
        rx.set_dormant(true);
        writeln!(console, "\r\n[address {:02X}, not mine]\r", byte).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
