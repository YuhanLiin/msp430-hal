//! eUSCI_A0 on its default and its remapped pins: the example prints `HELLO DEFAULT` on the backchannel
//! UART, then moves eUSCI_A0 to P5.2 (TXD) and P5.1 (RXD), prints `HELLO REMAPPED` there, and sends back
//! every character it receives.
//!
//! The default pins are P1.4 and P1.5, the backchannel UART's; USCIA0RMP = 1 then selects the remapped
//! ones. Both run at 9600 baud, 8 data bits, no parity and one stop bit, clocked by ACLK from REFO. LED1
//! lights once the remapped UART is set up.
//! (The pins and USCIA0RMP: SLASEO7C Table 9-11, p. 54. The backchannel UART: SLAU802 2.2.4, p. 9. LED1
//! on P1.0 is green, P5.1 also drives the red part of LED2 through J8, and S3 is the reset button:
//! SLAU802 Figure 19, p. 25.)
//!
//! How to test (two jumper wires):
//! 1. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9). Expected: `HELLO DEFAULT` (press S3 to see it again if
//!    the terminal wasn't open yet), and LED1 lights.
//! 2. Now move the backchannel to the remapped pins. Pull the TXD and RXD jumpers off J101, and the J8
//!    jumper marked P5.1, so that LED2 doesn't load the RXD line. With the jumper wires, connect the eZ-FET
//!    side of TXD, the pin nearer the USB connector, to P5.2 (J4 pin 40), and the eZ-FET side of RXD to
//!    P5.1 (J4 pin 39). (J101: SLAU802 Table 2, p. 8; board layout: SLAU802 Figure 1, p. 1; header pins:
//!    SLAU802 Figure 10, p. 13.)
//! 3. Press S3. Expected: `HELLO REMAPPED`, and each character you type comes back (with the terminal's
//!    local echo off).
//! 4. Put the jumpers back for the other examples.
#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::{DefaultMapping, RemappedMapping}, pmm::Pmm, serial::*, watchdog::Wdt
};

use nb::block;
#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr247x::Peripherals::take() {
        let mut fram = Fram::new(periph.frctl);
        // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
        let _wdt = Wdt::constrain(periph.wdt_a);

        // MCLK from DCOCLKDIV, SMCLK = MCLK / 2 and ACLK from REFO (SELMS = 000b, SELA = 01b: SLAU445I
        // Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
        let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);

        let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

        let p1 = Batch::new(periph.p1).split(&pmm);
        let p5 = Batch::new(periph.p5).split(&pmm);

        // LED1 (SLAU802 Figure 19, p. 25)
        let mut led = p1.pin0.to_output();
        led.set_low().ok();

        let e_usci_a0 = periph.e_usci_a0;

        // FIRST: Default UART mapping (P1.4 TX / P1.5 RX)
        // (SLASEO7C Table 9-11, p. 54; UCA0TXD with P1SEL = 01: SLASEO7C Table 9-23, p. 65)
        // (8N1, LSB first: UCMSB, UC7BIT, UCSPB, UCPEN in SLAU445I Table 22-8, p. 593; ACLK is
        // UCSSEL = 01b: SLASEO7C Table 9-8, p. 50)
        let (e_usci_a0, _p1_4) = {
            let mut tx = SerialConfig::<_, _, DefaultMapping>::new(
                e_usci_a0,
                BitOrder::LsbFirst,
                BitCount::EightBits,
                StopBits::OneStopBit,
                Parity::NoParity,
                Loopback::NoLoop,
                9600,
            )
            .use_aclk(&aclk).tx_only(p1.pin4.to_alternate1());

            embedded_io::Write::write_all(&mut tx, b"HELLO DEFAULT\n").ok();
            // Send the last character, then take eUSCI_A0 back (P1.4 comes back as a GPIO input)
            embedded_io::Write::flush(&mut tx).ok();
            tx.free()
        };

        // SECOND: Remap UART to P5.2 TX / P5.1 RX
        // (USCIA0RMP: SLASEO7C Table 9-11, p. 54; UCA0TXD and UCA0RXD with P5SEL = 01:
        // SLASEO7C Table 9-27, p. 69. USCIA0RMP is bit 0 of SYSCFG3: SLAU445I Table 1-32, p. 83)
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

        let (mut tx, mut rx) =
            serial.split(p5.pin2.to_alternate1(), p5.pin1.to_alternate1());

        led.set_high().ok();

        embedded_io::Write::write_all(&mut tx, b"HELLO REMAPPED\n").ok();

        // Echo loop on remapped UART
        // (The receive errors are the UCPE, UCOE, UCFE and UCBRK flags: SLAU445I Table 22-1, p. 582)
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
