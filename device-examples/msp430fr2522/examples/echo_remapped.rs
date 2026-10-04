//! eUSCI_A0 on its default and its remapped pins: the example prints `HELLO DEFAULT` on P1.4, then moves
//! eUSCI_A0 to P2.0 (TXD) and P2.1 (RXD), prints `HELLO REMAPPED` there, and sends back every character
//! it receives.
//!
//! USCIARMP = 0 selects the default pins, P1.4 and P1.5, and USCIARMP = 1 the remapped ones. Both run at
//! 9600 baud, 8 data bits, no parity and one stop bit, clocked by ACLK from REFO. An LED on P1.0 lights
//! once the remapped UART is set up. There's no LaunchPad for the MSP430FR25x2, so the PC needs a
//! USB-to-UART adapter.
//! (The pins: SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58. P2.0 and P2.1 are also XOUT and
//! XIN: SLASEE4C Table 6-16, p. 60.)
//!
//! How to test (a 3.3-V USB-to-UART adapter, an LED and a resistor):
//! 1. Connect the LED with a series resistor (about 1 kΩ) from P1.0 to GND, the adapter's GND to GND, and
//!    its RX to P1.4. P2.0 and P2.1 must have no crystal on them.
//! 2. Flash this example, and open the adapter's COM port at 9600 baud. Expected: `HELLO DEFAULT`, and
//!    the LED lights.
//! 3. Move the adapter's RX to P2.0, connect its TX to P2.1, and flash the example again. Expected:
//!    `HELLO REMAPPED`, and each character you type comes back (with the terminal's local echo off).
#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::{DefaultMapping, RemappedMapping},
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
        let p2 = Batch::new(periph.p2).split(&pmm);

        // No board document covers an LED on P1.0: there is none for the MSP430FR25x2. P1.0 is a GPIO
        // output, P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58).
        let mut led = p1.pin0.to_output();
        led.set_low().ok();

        let mut e_usci_a0 = periph.e_usci_a0;

        // FIRST: Default UART mapping (P1.4 TX / P1.5 RX): USCIARMP = 0, UCA0TXD with P1SELx = 01
        // (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
        {
            let mut tx = SerialConfig::<_, _, DefaultMapping>::new(
                e_usci_a0,
                BitOrder::LsbFirst,
                BitCount::EightBits,
                StopBits::OneStopBit,
                Parity::NoParity,
                Loopback::NoLoop,
                9600,
            )
            .use_aclk(&aclk)
            .tx_only(p1.pin4.to_alternate1());

            embedded_io::Write::write_all(&mut tx, b"HELLO DEFAULT\n").ok();
        }

        unsafe {
            e_usci_a0 = msp430fr25x2::Peripherals::steal().e_usci_a0;
        }

        // SECOND: Remap UART to P2.0 TX / P2.1 RX: USCIARMP = 1, UCA0TXD and UCA0RXD with P2SELx = 01
        // (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
        let serial = SerialConfig::<_, _, RemappedMapping>::new(
            e_usci_a0,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_aclk(&aclk);

        let (mut tx, mut rx) = serial.split(p2.pin0.to_alternate1(), p2.pin1.to_alternate1());

        led.set_high().ok();

        embedded_io::Write::write_all(&mut tx, b"HELLO REMAPPED\n").ok();

        // Echo loop on remapped UART
        loop {
            let ch: u8 = match block!(rx.read()) {
                Ok(c) => c,
                Err(RecvError::Parity) => b'!',
                Err(RecvError::Overrun(_)) => b'}',
                Err(RecvError::Framing) => b'?',
                Err(RecvError::Break)   => b'#',
            };

            block!(tx.write(ch)).ok();
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
