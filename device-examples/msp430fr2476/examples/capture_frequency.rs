//! A frequency counter with two TB0 capture inputs: CCR0 on P6.2, read in CCR0's own interrupt with
//! `Capture::interrupt_capture()`, and CCR6 on P4.4, polled. Twice a second the backchannel UART prints
//! the frequency each one measures.
//!
//! TB0 counts SMCLK at about 8 MHz in continuous mode, and each input captures the count at every rising
//! edge, so a signal's period is the difference between two captures. The clocks come from REFO, which is
//! accurate to ±3.5 % (SLASEO7C 8.12.3.4, p. 30), so the result is too.
//! (TB0.CCI0A is P6.2 and TB0.CCI6A is P4.4: SLASEO7C Table 9-15, p. 59. Captures: SLAU445I 14.2.4.1,
//! p. 398. CCR0's interrupt clears its own flag: SLAU445I 14.2.6.1, p. 405.)
//!
//! How to test (function generator):
//! 1. Generator: square wave, 1 kHz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load High-Z. Check the
//!    levels on the scope before connecting: a negative or >3.6 V signal can damage the pins.
//! 2. Connect it to P6.2 (J4 pin 33) and P4.4 (J3 pin 25), with the BNC T-piece and two leads, and its
//!    ground to GND (J3 pin 22). (Header pins: SLAU802 Figure 10, p. 13.)
//! 3. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9).
//! 4. Expected: `CCR0 (P6.2): 1000.0 Hz, CCR6 (P4.4): 1000.0 Hz`, give or take the clock's tolerance.
//!    Try 200 Hz to 20 kHz: below 123 Hz a period no longer fits in the 16-bit counter, and far above
//!    20 kHz the interrupt can't keep up with the edges.
//! 5. Disconnect one input: its value shows `no signal`.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::{Cell, RefCell};
use critical_section::with;
use embedded_hal::delay::DelayNs;
use embedded_io::Write;
use msp430::interrupt::{enable as enable_interrupts, Mutex};
use msp430_rt::entry;
use msp430_hal::{
    capture::{CapTrigger, Capture, CaptureParts7, CapturePin, TimerConfig, CCR0, CCR6},
    clock::{Clock, ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use msp430fr247x::{interrupt, Tb0};
use panic_msp430 as _;

static CAPTURE0: Mutex<RefCell<Option<Capture<Tb0, CCR0>>>> = Mutex::new(RefCell::new(None));
/// The last capture of CCR0, and the period between the last two (0 until there are two)
static LAST0: Mutex<Cell<Option<u16>>> = Mutex::new(Cell::new(None));
static PERIOD0: Mutex<Cell<u16>> = Mutex::new(Cell::new(0));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let p6 = Batch::new(periph.p6).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);
    let smclk_hz = smclk.freq();

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

    // TB0 counts SMCLK (TBSSEL = 10b: SLAU445I Table 14-6, p. 409) in continuous mode. Input A of CCR0 is
    // P6.2 (P6SEL = 01: SLASEO7C Table 9-28, p. 70) and of CCR6 P4.4 (P4SEL = 10: SLASEO7C Table 9-26,
    // p. 68). Both capture on rising edges (CM = 01b: SLAU445I Table 14-8, p. 411).
    let captures = CaptureParts7::config(periph.tb0, TimerConfig::smclk(&smclk))
        .config_cap0_input_A(p6.pin2.to_alternate1())
        .config_cap0_trigger(CapTrigger::RisingEdge)
        .config_cap6_input_A(p4.pin4.to_alternate2())
        .config_cap6_trigger(CapTrigger::RisingEdge)
        .commit();
    let mut capture0 = captures.cap0;
    let mut capture6 = captures.cap6;

    // CCIE requests CCR0's interrupt for each capture (SLAU445I Table 14-8, p. 412). Set GIE, which masks
    // every maskable interrupt while clear (SLAU445I 1.3.3, p. 33).
    capture0.enable_interrupts();
    with(|cs| CAPTURE0.borrow_ref_mut(cs).replace(capture0));
    unsafe { enable_interrupts() };

    loop {
        delay.delay_ms(500);

        let period0 = with(|cs| PERIOD0.borrow(cs).replace(0));
        let period6 = poll_period(&mut capture6);

        write!(tx, "CCR0 (P6.2): ").ok();
        print_frequency(&mut tx, smclk_hz, period0);
        write!(tx, ", CCR6 (P4.4): ").ok();
        print_frequency(&mut tx, smclk_hz, period6);
        writeln!(tx, "\r").ok();
    }
}

/// The period between two captures of CCR6, in SMCLK cycles, or 0 if one doesn't come within 20 000 tries
fn poll_period(capture: &mut Capture<Tb0, CCR6>) -> u16 {
    // Throw away a capture from before
    let _ = capture.capture();
    let mut captures = [0u16; 2];
    for value in captures.iter_mut() {
        let mut tries = 0u16;
        *value = loop {
            match capture.capture() {
                Ok(count) => break count,
                // A second capture before the first was read: start again from this one
                Err(nb::Error::Other(over)) => break over.0,
                Err(nb::Error::WouldBlock) => {
                    tries += 1;
                    if tries == 20_000 {
                        return 0;
                    }
                }
            }
        };
    }
    captures[1].wrapping_sub(captures[0])
}

/// Print `smclk_hz / period` with one decimal, or `no signal` for a period of 0
fn print_frequency(tx: &mut impl Write, smclk_hz: u32, period: u16) {
    if period == 0 {
        write!(tx, "no signal").ok();
    } else {
        let decihertz = smclk_hz * 10 / period as u32;
        write!(tx, "{}.{} Hz", decihertz / 10, decihertz % 10).ok();
    }
}

// The TB0 CCR0 vector, CCIFG0 (FFE8h: SLASEO7C Table 9-2, p. 46). Servicing it clears CCIFG0 (SLAU445I
// 14.2.6.1, p. 405), so the capture is read with `interrupt_capture()`.
#[interrupt]
fn TIMER0_B0() {
    with(|cs| {
        if let Some(capture) = CAPTURE0.borrow_ref_mut(cs).as_mut() {
            let (count, valid) = match capture.interrupt_capture() {
                Ok(count) => (count, true),
                // An edge was missed, so this period would be wrong
                Err(over) => (over.0, false),
            };
            if let (Some(last), true) = (LAST0.borrow(cs).get(), valid) {
                PERIOD0.borrow(cs).set(count.wrapping_sub(last));
            }
            LAST0.borrow(cs).set(Some(count));
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
