//! An echo on the backchannel UART: the example prints `HELLO`, then sends back every character it
//! receives.
//!
//! eUSCI_A0 runs at 9600 baud, 8 data bits, no parity and one stop bit, clocked by SMCLK. A character
//! that arrives with an error comes back as `!` (parity), `}` (overrun), `?` (framing) or `#` (break).
//! LED1 lights once the UART is set up.
//! (The backchannel UART is eUSCI_A0: SLAU739 2.2.4, p. 9. Its pins are P1.4 (TXD) and P1.5 (RXD), LED1
//! on P1.0 is red, and S3 is the reset button: SLAU739 Figure 18, p. 23.)
//!
//! How to test:
//! 1. Flash this example, with the TXD and RXD jumpers of J101 on, and open the COM port of "MSP
//!    Application UART1" at 9600 baud (SLAU739 2.2.4, p. 9).
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
    let Some(periph) = msp430fr2433::Peripherals::take() else { loop{} };
    let mut fram = Fram::new(periph.frctl);
    // Hold the watchdog (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2,
    // p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // MCLK = about 1 MHz: DCORSEL = 000b with the FLL locked to REFO (SLAU445I Table 3-5, p. 114; SLAU445I
    // 3.2.5, p. 104), DIVM /1; SMCLK = MCLK / 2: DIVS (SLAU445I Table 3-9, p. 118). ACLK = REFO: SELA = 01b
    // (SLAU445I Table 3-8, p. 117).
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output(); // Red LED1 (SLAU739 Figure 18, p. 23)
    // P1.4 UCA0TXD and P1.5 UCA0RXD, P1SELx = 01 below (SLASE59F Table 6-17, p. 55): the backchannel
    // UART's TXD and RXD, through the J101 jumpers (SLAU739 Figure 18, p. 23; SLAU739 Table 2, p. 8)
    let tx_pin = port1.pin4;
    let rx_pin = port1.pin5;
    
    led.set_low().ok();

    // LSB first (UCMSB = 0), 8 data bits (UC7BIT = 0), one stop bit (UCSPB = 0), no parity (UCPEN = 0)
    // (SLAU445I Table 22-8, p. 593); no loopback (UCLISTEN = 0, SLAU445I Table 22-12, p. 596). The 9600-baud
    // divider for SMCLK (UCSSELx = 10b) follows SLAU445I 22.3.10, p. 586.
    let (mut tx, mut rx) = SerialConfig::new(
        periph.e_usci_a0,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        // Launchpad UART-to-USB converter doesn't handle parity, so we don't use it
        Parity::NoParity,
        Loopback::NoLoop,
        9600)
        .use_smclk(&smclk)
        .split(tx_pin.to_alternate1(), rx_pin.to_alternate1());

    // embedded_io contains methods for writing with buffers
    led.set_high().ok();
    embedded_io::Write::write_all(&mut tx, b"HELLO\n").ok();
    loop {
        // embedded_hal_nb contains non-blocking methods for writing single bytes
        // The receive errors are UCPE, UCOE, UCFE and UCBRK in UCAxSTATW (SLAU445I Table 22-12, p. 596).
        let ch: u8 = match block!(rx.read()) {
            Ok(c) => c,
            Err(RecvError::Parity)      => b'!',
            Err(RecvError::Overrun(_))  => b'}',
            Err(RecvError::Framing)     => b'?',
            Err(RecvError::Break)       => b'#',
        };
        block!(tx.write(ch)).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
