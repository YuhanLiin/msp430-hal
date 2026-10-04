//! Automatic baud-rate detection, as in LIN: a receiver whose clock is off measures the sender's baud rate
//! from a break and synch field, and then receives the data correctly.
//!
//! eUSCI_A0, on its remapped TXD pin P5.2, sends at 9600 baud: a break, the synch field 55h, then `OK`.
//! eUSCI_A1 receives on P2.5, set up for 11 000 baud, about 15 % too fast. It measures the synch field,
//! takes on the sender's baud rate, and receives `OK`. LED2 shows the result: green for `OK`, red for
//! anything else. This repeats once a second.
//! (Automatic baud-rate detection: SLAU445I 22.3.4, p. 580. Sending the break and synch field: SLAU445I
//! 22.3.4.1, p. 581. UCA0TXD is P5.2 with USCIA0RMP = 1: SLASEO7C Table 9-11, p. 54. UCA1RXD is P2.5:
//! SLASEO7C Table 9-24, p. 66. LED2 is red on P5.1 and green on P5.0: SLAU802 Figure 19, p. 25.)
//!
//! The receiver detects a break as 11 to 21 of its own bit times ("A break is detected when 11 or more
//! continuous zeros (spaces) are received. If the length of the break exceeds 21 bit times, the break
//! timeout error flag UCBTOE is set": SLAU445I 22.3.4, p. 580). The 13-bit break at 9600 baud is 15 bit
//! times at 11 000 baud, so the receiver's baud rate may be off by about -15 % to +60 %.
//!
//! How to test (a jumper wire, and optionally the scope):
//! 1. Connect P5.2 (J4 pin 40) to P2.5 (J1 pin 3) with a jumper wire. (Header pins: SLAU802 Figure 10,
//!    p. 13.)
//! 2. Flash this example: LED2 blinks green once a second.
//! 3. Set `AUTO_BAUD = false` and flash again: eUSCI_A1 is now a plain UART at 11 000 baud, and LED2
//!    blinks red, as it can't read 9600 baud.
//! 4. With the scope on P5.2, ground on GND (J3 pin 22), 1 ms/div: the line low for 1.4 ms (the break),
//!    a short high (the delimiter), then the synch field 55h, whose bits alternate, and `OK`.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use embedded_hal_nb::serial::Read;
use embedded_io::Write;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::RemappedMapping,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

/// Measure the baud rate from the break and synch field, or receive as a plain UART
const AUTO_BAUD: bool = true;
/// The baud rate the sender uses
const SENDER_BAUD: u32 = 9600;
/// The baud rate the receiver starts with
const RECEIVER_BAUD: u32 = 11_000;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    let mut red = p5.pin1.to_output_low();
    let mut green = p5.pin0.to_output_low();

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range, so the CPU keeps up with the receiver, and ACLK from
    // REFO (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The sender: eUSCI_A0 on its remapped TXD pin, P5.2, with P5SEL = 01 (SLASEO7C Table 9-27, p. 69). In
    // automatic baud-rate mode (UCMODEx = 11b), a break comes with a delimiter and the synch field (SLAU445I
    // 22.3.4.1, p. 581); 8 data bits, LSB first, no parity and one stop bit, as LIN needs (SLAU445I 22.3.4,
    // p. 580).
    let mut sender = SerialConfig::<_, _, RemappedMapping>::new(
        periph.e_usci_a0,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        SENDER_BAUD,
    )
    .mode(UartMode::AutoBaud { delimiter: BreakDelimiter::_1Bit })
    .use_smclk(&smclk)
    .tx_only(p5.pin2.to_alternate1());

    // The receiver: eUSCI_A1's RXD, P2.5, with P2SEL = 01 (SLASEO7C Table 9-24, p. 66). UCBRKIE makes the
    // break and synch field readable, as `RecvError::Break` (SLAU445I 22.3.4, p. 580: "If the UCBRKIE bit
    // is set, reception of the break/synch sets the UCRXIFG").
    let mode = if AUTO_BAUD { UartMode::AutoBaud { delimiter: BreakDelimiter::_1Bit } } else { UartMode::Uart };
    let mut receiver = SerialConfig::new(
        periph.e_usci_a1,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        RECEIVER_BAUD,
    )
    .mode(mode)
    .break_interrupts()
    .use_smclk(&smclk)
    .rx_only(p2.pin5.to_alternate1());

    loop {
        // While dormant, the receiver waits for a break and synch field (SLAU445I 22.3.4, p. 580: "If UCDORM
        // remains set, only the character after the next reception of a break/synch field is received")
        if AUTO_BAUD {
            receiver.set_dormant(true);
        }
        block!(sender.send_break()).ok();

        // The break, then the characters
        let ok = wait_for_break(&mut receiver, &mut delay) && {
            // "user software must reset UCDORM to continue receiving data" (SLAU445I 22.3.4, p. 580)
            receiver.set_dormant(false);
            sender.write_all(b"OK").ok();
            let mut received = [0u8; 2];
            let mut all = true;
            for byte in received.iter_mut() {
                match read_within(&mut receiver, &mut delay) {
                    Some(Ok(value)) => *byte = value,
                    _ => all = false,
                }
            }
            // Characters a plain UART at the wrong baud rate reads from the synch field may be left over
            while receiver.read().is_ok() {}
            all && &received == b"OK"
        };
        // UCBTOE and UCSTOE, the break and synch timeout errors (SLAU445I Table 22-15, p. 598)
        let (break_too_long, synch_too_long) = receiver.auto_baud_errors();
        let ok = ok && !break_too_long && !synch_too_long;

        let led: &mut dyn OutputPin<Error = _> = if ok { &mut green } else { &mut red };
        led.set_high().ok();
        delay.delay_ms(100);
        led.set_low().ok();
        delay.delay_ms(900);
    }
}

/// Whether the receiver reports a break within 10 ms
fn wait_for_break<USCI: SerialUsci>(receiver: &mut Rx<USCI>, delay: &mut impl DelayNs) -> bool {
    loop {
        match read_within(receiver, delay) {
            Some(Err(RecvError::Break)) => return true,
            None => return false,
            // A plain UART at the wrong baud rate may read other things first
            Some(_) => continue,
        }
    }
}

/// The next character or error from the receiver, or `None` if nothing comes within 10 ms
fn read_within<USCI: SerialUsci>(receiver: &mut Rx<USCI>, delay: &mut impl DelayNs) -> Option<Result<u8, RecvError>> {
    for _ in 0..1000 {
        match receiver.read() {
            Ok(byte) => return Some(Ok(byte)),
            Err(nb::Error::Other(error)) => return Some(Err(error)),
            Err(nb::Error::WouldBlock) => delay.delay_us(10),
        }
    }
    None
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
