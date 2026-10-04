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

// Connect push button input to P3.3. When button is pressed, putty should print the # of cycles
// since the last press. Sometimes we get 2 consecutive readings due to lack of debouncing.
// P3.3 is J4 pin 35 (SLAU802 Figure 10, p. 13), a timer capture header pin wired to nothing else on the
// LaunchPad (net P3.3_TC: SLAU802 Figure 18, p. 24). P1.1 isn't used, because the TMP235 temperature
// sensor drives it (SLAU802 2.2.5.1, p. 10). The text goes out on the backchannel UART, eUSCI_A0's TXD
// on P1.4 (SLAU802 2.2.4, p. 9; SLAU802 Figure 16, p. 22), with the TXD jumper of J101 on (SLAU802
// Table 2, p. 8).
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
    let p3 = Batch::new(periph.p3).split(&pmm);

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

    // TA2 counts ACLK, here from the VLO. Its CCR1 input A (CCI1A) is P3.3 with P3SEL = 01 and P3DIR = 0
    // (SLASEO7C Table 9-14, p. 58; SLASEO7C Table 9-25, p. 67), in the default TA2 mapping (TA2RMP = 0:
    // SLAU445I Table 1-32, p. 83). ACLK is TASSEL = 01b (SLASEO7C Table 9-8, p. 50). A capture on the
    // falling edge (CM = 10b, CCIS = 00b: SLAU445I Table 13-6, p. 386) copies TA2R into TA2CCR1 and sets
    // CCIFG (SLAU445I 13.2.4.1, p. 374); a second capture before the first is read sets COV (SLAU445I
    // 13.2.4.1, p. 375), reported as `OverCapture`.
    let captures = CaptureParts3::<_, DefaultMapping>::config(periph.ta2, TimerConfig::aclk(&aclk))
        .config_cap1_input_A(p3.pin3.to_alternate1())
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
