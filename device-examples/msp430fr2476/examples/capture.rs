//! Timer captures, polled: TA3 captures each press of button S1, and the backchannel UART prints the
//! time since the press before, in VLO cycles.
//!
//! S1 is on P4.0, which is TA3.CCI1A, input A of TA3's CCR1. S1 pulls P4.0 low when pressed, and a
//! 47-kΩ resistor on the board, R9, pulls it high again: CCR1 captures TA3's count at each falling edge.
//! TA3 counts ACLK from the VLO, typically 10 kHz, so a second is about 10000 counts, 0x2710; the count
//! wraps around after 65536 counts, about 6.5 s. Each capture also turns LED1 on.
//! (TA3.CCI1A is P4.0 in the default pin mapping: SLASEO7C Table 9-14, p. 58; SLASEO7C Table 9-16,
//! p. 60. Captures: SLAU445I 13.2.4.1, p. 374. VLO: SLASEO7C 8.12.3.5, p. 30. S1 on P4.0 with R9, and
//! LED1 on P1.0, green: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9).
//! 2. Press S1 about once a second. Expected: a line per press, like `0x2710`: the VLO cycles since the
//!    press before, in hex. The VLO is only accurate to ±50 % (VLOCLK "10 kHz ±50%": SLASEO7C
//!    Table 9-8, p. 50), so anything from about `0x1388` to `0x3a98`. The first press counts from the
//!    start.
//! 3. S1 isn't debounced, so a press or a release can print an extra line with a small value, or `!`
//!    when a second edge came before the first was read.
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
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    prelude::*,
    serial::*,
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // P1.0 drives LED1 (SLAU802 Figure 19, p. 25), switched on at the first capture
    let mut p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range, and ACLK from the VLO (SELMS, SELA: SLAU445I
    // Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118). SLASEO7C 9.10.2, p. 49 lists the VLO
    // as an ACLK source of this device; SLAU445I Table 3-1, p. 98 and the footnote of SLAU445I
    // Table 3-8, p. 117 give ACLK = VLO for the enhanced clock system only. The HAL follows the data
    // sheet.
    let (smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // eUSCI_A0 TXD is P1.4 with P1SEL = 01 (SLASEO7C Table 9-23, p. 65), in the default mapping
    // (USCIA0RMP = 0: SLASEO7C Table 9-11, p. 54; SLAU445I Table 1-32, p. 83)
    // (LSB first, 8 data bits, one stop bit, no parity: UCMSB, UC7BIT, UCSPB, UCPEN in SLAU445I
    // Table 22-8, p. 593; SMCLK is UCSSEL = 10b: SLASEO7C Table 9-8, p. 50)
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

    // TA3 counts ACLK, here from the VLO. Its CCR1 input A (CCI1A) is P4.0, S1, with P4SEL = 01 and
    // P4DIR = 0 (SLASEO7C Table 9-14, p. 58; SLASEO7C Table 9-26, p. 68), in the default TA3 mapping
    // (TA3RMP = 0: SLASEO7C Table 9-16, p. 60; SLAU445I Table 1-32, p. 83). ACLK is TASSEL = 01b
    // (SLASEO7C Table 9-8, p. 50). A capture on the falling edge (CM = 10b, CCIS = 00b: SLAU445I
    // Table 13-6, p. 386) copies TA3R into TA3CCR1 and sets CCIFG (SLAU445I 13.2.4.1, p. 374); a second
    // capture before the first is read sets COV (SLAU445I 13.2.4.1, p. 375), reported as `OverCapture`.
    let captures = CaptureParts3::<_, DefaultMapping>::config(periph.ta3, TimerConfig::aclk(&aclk))
        .config_cap1_input_A(p4.pin0.to_alternate1())
        .config_cap1_trigger(CapTrigger::FallingEdge)
        .commit();
    let mut capture = captures.cap1;

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
