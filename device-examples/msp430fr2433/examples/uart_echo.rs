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

// Prints "HELLO" when started then echos on UART0
// Serial settings are listed in the code
#[entry]
fn main() -> ! {
    let Some(periph) = msp430fr2433::Peripherals::take() else { loop{} };
    let mut fram = Fram::new(periph.fram);
    let _wdt = Wdt::constrain(periph.watchdog_timer);

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    let mut led = port1.pin0.to_output();
    let tx_pin = port1.pin4;
    let rx_pin = port1.pin5;
    
    led.set_low().ok();

    let (mut tx, mut rx) = SerialConfig::new(
        periph.usci_a0_uart_mode,
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
        let ch: u8 = match block!(rx.read()) {
            Ok(c) => c,
            Err(RecvError::Parity)      => b'!',
            Err(RecvError::Overrun(_))  => b'}',
            Err(RecvError::Framing)     => b'?',
            Err(RecvError::Break)       => b'#',
        };
        block!(tx.write(ch));
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
