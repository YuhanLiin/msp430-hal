//! Timer captures, polled: TB0 captures each falling edge on P1.6, and the backchannel UART prints the
//! time since the edge before, in VLO cycles.
//!
//! P1.6 is TB0.CCI1A, input A of TB0's CCR1, which captures TB0's count at each falling edge. TB0
//! counts ACLK from the VLO, typically 10 kHz, so a second is about 10000 counts, 0x2710; the count
//! wraps around after 65536 counts, about 6.5 s. Each capture also turns LED1 on. The LaunchPad's
//! buttons, S1 on P4.1 and S2 on P2.3, aren't capture inputs, so the edges come from the function
//! generator or a jumper wire.
//! (TB0.CCI1A on P1.6: SLASEC4D Table 6-16, p. 73. Captures: SLAU445I 14.2.4.1, p. 398. VLO: SLASEC4D
//! Table 5-8, p. 40. Capture inputs: SLASEC4D Tables 6-16 to 6-19, p. 73 to p. 75. S1, S2, and LED1 on
//! P1.0, red: SLAU680 Figure 18, p. 26.)
//!
//! How to test (function generator, or a jumper wire):
//! 1. Generator: square wave, 1 Hz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load High-Z. Check the
//!    levels on the scope before connecting: a negative or >3.6 V signal can damage the pin. Connect it
//!    to P1.6 (J1 pin 3), its ground to GND (J3 pin 22).
//! 2. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU680 2.2.4, p. 11).
//! 3. Expected: once a second, a line like `0x2710`: the VLO cycles since the falling edge before, in
//!    hex, which is the VLO's frequency in Hz. The VLO is only accurate to ±50 % (VLOCLK "10 kHz
//!    ±50%": SLASEC4D Table 6-9, p. 68), so anything from about `0x1388` to `0x3a98`. The first line
//!    counts from the start.
//! 4. Without the generator: put a jumper wire on P1.6 (J1 pin 3), and touch its free end to 3.3 V (J1
//!    pin 1), then to GND (J2 pin 20). Each touch of GND prints a line. P1.6 has no pull resistor, so it
//!    floats between touches, and the contact bounces: expect extra lines with small values, or `!`
//!    when a second edge came before the first was read.
//! (Header pins: SLAU680 Figure 10, p. 15.)
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
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p4 = Batch::new(periph.p4).split(&pmm);
    // P1.0 drives LED1, red (SLAU680 Figure 18, p. 26)
    let mut p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);

    let (smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut tx = SerialConfig::new(
        periph.e_usci_a1,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        9600,
    )
    .use_smclk(&smclk)
    .tx_only(p4.pin3.to_alternate1()); // UCA1TXD, P4SELx = 01 (SLASEC4D Table 6-66, p. 102)

    let captures = CaptureParts3::config(periph.tb0, TimerConfig::aclk(&aclk))
        .config_cap1_input_A(p1.pin6.to_alternate2()) // TB0.CCI1A, P1SELx = 10 (SLASEC4D Table 6-63, p. 96)
        .config_cap1_trigger(CapTrigger::FallingEdge)
        .commit();
    let mut capture = captures.cap1;

    let mut last_cap = 0;
    loop {
        match block!(capture.capture()) {
            Ok(cap) => {
                let diff = cap.wrapping_sub(last_cap);
                last_cap = cap;
                // LED1 on P1.0 (SLAU680 Figure 18, p. 26)
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
