//! Timer captures, polled: TA0 captures each falling edge on P1.5, and eUSCI_A0 prints the time since
//! the edge before, in ACLK cycles, on P1.4.
//!
//! P1.5 is TA0.CCI2A, input A of TA0's CCR2, which captures TA0's count at each falling edge. TA0
//! counts ACLK from REFO, 32768 Hz, so a second is 32768 counts, 0x8000; the count wraps around after
//! 65536 counts, 2 s. Each capture also sets P1.0 high, which lights an LED there.
//! (TA0.CCI2A on P1.5: SLASEE4C Table 6-15, p. 58; SLASEE4C Figure 6-2, p. 54. Captures: SLAU445I
//! 13.2.4.1, p. 374. REFO: SLASEE4C Table 5-7, p. 27. UCA0TXD on P1.4: SLASEE4C Table 6-11, p. 53. No
//! board document covers the parts to connect: there is none for the MSP430FR25x2.)
//!
//! How to test (a push button, an LED, two resistors, a USB-to-UART adapter; or the function generator):
//! 1. Connect a push button from P1.5 to GND, and a resistor of about 47 kΩ from P1.5 to 3.3 V: P1.5's
//!    internal pull resistor is off. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Connect a 3.3-V USB-to-UART adapter: its RX to P1.4 (UCA0TXD), GND to GND. Open its COM port at
//!    9600 baud.
//! 3. Flash this example, and press the button about once a second. Expected: a line per press, like
//!    `0x8000`: the ACLK cycles since the press before, in hex. The first press counts from the start,
//!    and presses more than 2 s apart wrap around. The LED is on after the first press.
//! 4. The button isn't debounced, so a press or a release can print an extra line with a small value,
//!    or `!` when a second edge came before the first was read.
//! 5. Instead of the button, the generator: square wave, 1 Hz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset),
//!    output load High-Z (check the levels on the scope first), to P1.5, its ground to GND: about `0x8000`,
//!    once a second.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use embedded_hal_nb::serial::Write;
use msp430_rt::entry;
use msp430_hal::{
    capture::{CapTrigger, CaptureParts3, OverCapture, TimerConfig},
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    prelude::*,
    serial::*,
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let mut p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);

    let (smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk() // ACLK from REFO, 32768 Hz (SLASEE4C Table 5-7, p. 27)
        .freeze(&mut fram);

    // TXD on P1.4: UCA0TXD with P1SELx = 01 in the default mapping, USCIARMP = 0
    // (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    let mut tx = SerialConfig::new(
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

    // P1.5 as TA0.CCI2A, capture input A of CCR2: P1SELx = 10 with P1DIR = 0 (SLASEE4C Table 6-15, p. 58;
    // SLASEE4C Figure 6-2, p. 54)
    let captures = CaptureParts3::config(periph.ta0, TimerConfig::aclk(&aclk))
        .config_cap2_input_A(p1.pin5.to_alternate2())
        .config_cap2_trigger(CapTrigger::FallingEdge)
        .commit();
    // CCR2 captures falling edges on its input A, P1.5
    let mut capture = captures.cap2;

    let mut last_cap = 0;
    loop {
        match block!(capture.capture()) {
            Ok(cap) => {
                let diff = cap.wrapping_sub(last_cap);
                last_cap = cap;
                p1.pin0.set_high().unwrap();
                print_num(&mut tx, diff);
            }
            Err(OverCapture(_)) => {
                p1.pin0.set_high().unwrap();
                write(&mut tx, '!');
                write(&mut tx, '\r');
                write(&mut tx, '\n');
            }
        }
    }
}

fn print_num<U: SerialUsci>(tx: &mut Tx<U>, num: u16) {
    write(tx, '0');
    write(tx, 'x');
    print_hex(tx, num >> 12);
    print_hex(tx, (num >> 8) & 0xF);
    print_hex(tx, (num >> 4) & 0xF);
    print_hex(tx, num & 0xF);
    write(tx, '\r');
    write(tx, '\n');
}

fn print_hex<U: SerialUsci>(tx: &mut Tx<U>, h: u16) {
    let c = match h {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        _ => '?',
    };
    write(tx, c);
}

fn write<U: SerialUsci>(tx: &mut Tx<U>, ch: char) {
    nb::block!(tx.write(ch as u8)).unwrap();
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
