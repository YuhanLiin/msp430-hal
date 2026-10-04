//! An echo on eUSCI_A0: the example prints `HELLO`, then sends back every character it receives.
//!
//! eUSCI_A0, the only UART of this device, runs at 9600 baud, 8 data bits, no parity and one stop bit,
//! clocked by ACLK from REFO. A character that arrives with an error comes back as `!` (parity), `}`
//! (overrun), `?` (framing) or `#` (break). An LED on P1.0 lights once the UART is set up. There's no
//! LaunchPad for the MSP430FR25x2, so the PC needs a USB-to-UART adapter.
//! (eUSCI_A0: SLASEE4C 6.10.7, p. 53. UCA0TXD is P1.4 and UCA0RXD P1.5: SLASEE4C Table 6-11, p. 53;
//! SLASEE4C Table 6-15, p. 58.)
//!
//! How to test (a 3.3-V USB-to-UART adapter, an LED and a resistor):
//! 1. Connect the LED with a series resistor (about 1 kΩ) from P1.0 to GND, and the adapter: its RX to
//!    P1.4, its TX to P1.5 and its GND to GND.
//! 2. Flash this example, and open the adapter's COM port at 9600 baud.
//! 3. Expected: the LED lights, and the terminal shows `HELLO`. Each character you type comes back (with
//!    the terminal's local echo off).
#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
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
#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr25x2::Peripherals::take() {
        let mut fram = Fram::new(periph.frctl);
        // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
        let _wdt = Wdt::constrain(periph.wdt_a);

        let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk() // ACLK from REFO, 32768 Hz (SLASEE4C Table 5-7, p. 27)
            .freeze(&mut fram);

        // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
        let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
        let p1 = Batch::new(periph.p1).split(&pmm);

        // No board document covers an LED on P1.0: there is none for the MSP430FR25x2. P1.0 is a GPIO
        // output, P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).
        let mut led = p1.pin0.to_output();

        led.set_low().ok();

        // TXD on P1.4 and RXD on P1.5: UCA0TXD and UCA0RXD with P1SELx = 01 in the default mapping,
        // USCIARMP = 0 (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
        let (mut tx, mut rx) = SerialConfig::<_, _, DefaultMapping>::new(
            periph.e_usci_a0,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            // Launchpad UART-to-USB converter doesn't handle parity, so we don't use it
            // (no LaunchPad or other board document covers the MSP430FR25x2)
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_aclk(&aclk)
        .split(p1.pin4.to_alternate1(), p1.pin5.to_alternate1());

        led.set_high().ok();
        // embedded_io contains methods for writing with buffers
        embedded_io::Write::write_all(&mut tx, b"HELLO\n").ok();
        loop {
            // embedded_hal_nb contains non-blocking methods for writing single bytes
            let ch: u8 = match block!(rx.read()) {
                Ok(c) => c,
                Err(RecvError::Parity) => b'!',
                Err(RecvError::Overrun(_)) => b'}',
                Err(RecvError::Framing) => b'?',
                Err(RecvError::Break)   => b'#',
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
