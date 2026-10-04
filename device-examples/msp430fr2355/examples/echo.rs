//! An echo on the backchannel UART: the example prints `HELLO`, then sends back every character it
//! receives.
//!
//! eUSCI_A1 runs at 9600 baud, 8 data bits, no parity and one stop bit, clocked by ACLK from REFO. A
//! character that arrives with an error comes back as `!` (parity), `}` (overrun), `?` (framing) or `#`
//! (break). LED1 lights once the UART is set up.
//! (The backchannel UART is eUSCI_A1: SLAU680 2.2.4, p. 11. Its pins are P4.3 (TXD) and P4.2 (RXD), LED1
//! on P1.0 is red, and S3 is the reset button: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example, with the TXD and RXD jumpers of J101 on, and open the COM port of "MSP
//!    Application UART1" at 9600 baud (SLAU680 2.2.4, p. 11).
//! 2. Expected: LED1 lights, and the terminal shows `HELLO`. If the terminal wasn't open yet, press S3 to
//!    start the example again.
//! 3. Type something: each character comes back, so the terminal shows what you type (with its local
//!    echo off).
#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};

use nb::block;
#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr2355::Peripherals::take() {
        let mut fram = Fram::new(periph.frctl);
        let _wdt = Wdt::constrain(periph.wdt_a);

        let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);

        let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
        let mut led = Batch::new(periph.p1).split(&pmm).pin0.to_output();
        let p4 = Batch::new(periph.p4).split(&pmm);
        led.set_low().ok();

        let (mut tx, mut rx) = SerialConfig::new(
            periph.e_usci_a1,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            // Launchpad UART-to-USB converter doesn't handle parity, so we don't use it
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_aclk(&aclk)
        // UCA1TXD on P4.3 and UCA1RXD on P4.2, P4SELx = 01 (SLASEC4D Table 6-66, p. 102), wired to the
        // eZ-FET as BCL_TXD and BCL_RXD (SLAU680 Figure 18, p. 26)
        .split(p4.pin3.to_alternate1(), p4.pin2.to_alternate1());

        led.set_high().ok();
        // embedded_io contains methods for writing with buffers
        embedded_io::Write::write_all(&mut tx, b"HELLO\n").ok();
        loop {
            // embedded_hal_nb contains non-blocking methods for writing single bytes
            let ch: u8 = match block!(rx.read()) {
                Ok(c) => c,
                Err(RecvError::Parity)      => b'!',
                Err(RecvError::Overrun(_))  => b'}',
                Err(RecvError::Framing)     => b'?',
                Err(RecvError::Break)       => b'#',
            };
            block!(tx.write(ch)).unwrap();
        }
    } else {
        loop {}
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
