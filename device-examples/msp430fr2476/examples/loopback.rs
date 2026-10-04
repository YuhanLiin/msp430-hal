//! An echo through a second UART: each character from the PC goes out on eUSCI_A1, which runs in
//! loopback mode, and what eUSCI_A1 receives goes back to the PC on the backchannel UART.
//!
//! The backchannel UART, eUSCI_A0, runs at 19200 baud, 8 data bits, no parity and two stop bits. eUSCI_A1
//! runs at 20000 baud with even parity, and its loopback mode feeds its TXD back to its receiver, so it
//! needs no wires. SMCLK clocks both, at about 2 MHz. A character that arrives with an error comes back
//! as `!` (from the PC) or `?` (in the loopback). LED1 lights once both UARTs are set up.
//! (The backchannel UART is eUSCI_A0: SLAU802 2.2.4, p. 9. Loopback, where "UCAxTXD is internally fed
//! back to the receiver": SLAU445I 22.4.5, p. 596. LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example, with the TXD and RXD jumpers of J101 on, and open the COM port of "MSP
//!    Application UART1" at 19200 baud, two stop bits (SLAU802 2.2.4, p. 9).
//! 2. Expected: LED1 lights, and each character you type comes back (with the terminal's local echo off).
#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, Smclk, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::*, pmm::Pmm, serial::*, watchdog::Wdt
};
use nb::block;
use panic_msp430 as _;

/// LSB first, 8 data bits and two stop bits (UCMSB, UC7BIT, UCSPB), the given parity (UCPEN, UCPAR)
/// and loopback (UCLISTEN), clocked by SMCLK (UCSSEL = 10b) (SLAU445I Table 22-8, p. 593; SLAU445I
/// 22.4.5, p. 596; SLASEO7C Table 9-8, p. 50)
fn setup_uart<USCI, M>(
    usci: USCI,
    tx: USCI::TxPin,
    rx: USCI::RxPin,
    parity: Parity,
    loopback: Loopback,
    baudrate: u32,
    smclk: &Smclk,
) -> (Tx<USCI, M>, Rx<USCI, M>)
where
    USCI: SerialUsci<M>,
    M: PinMap,
{
    SerialConfig::<USCI, NoClockSet, M>::new(
        usci,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::TwoStopBits,
        parity,
        loopback,
        baudrate,
    )
    .use_smclk(smclk)
    .split(tx, rx)
}

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let mut fram = Fram::new(periph.frctl);
    // MCLK from DCOCLKDIV in the 4 MHz range, SMCLK = MCLK / 2, ACLK from REFO (SELMS = 000b, SELA =
    // 01b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_4MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    // LED1 (SLAU802 Figure 19, p. 25)
    let mut led = p1.pin0.to_output();
    led.set_low().ok();

    // UART0, the backchannel UART: P1.4 = UCA0TXD and P1.5 = UCA0RXD with P1SEL = 01 (SLASEO7C
    // Table 9-23, p. 65)
    let (mut tx0, mut rx0) = setup_uart::<_, DefaultMapping>(
        periph.e_usci_a0,
        p1.pin4.to_alternate1().into(),
        p1.pin5.to_alternate1().into(),
        Parity::NoParity,
        Loopback::NoLoop,
        19200,
        &smclk,
    );

    // UART1, in loopback mode: P2.6 = UCA1TXD and P2.5 = UCA1RXD with P2SEL = 01 (SLASEO7C Table 9-24,
    // p. 66)
    let (mut tx1, mut rx1) = setup_uart(
        periph.e_usci_a1,
        p2.pin6.to_alternate1().into(),
        p2.pin5.to_alternate1().into(),
        Parity::EvenParity,
        Loopback::Loopback,
        20000,
        &smclk,
    );

    led.set_high().ok();

    loop {
        let ch = block!(rx0.read()).unwrap_or(b'!');
        block!(tx1.write(ch)).ok();
        let ch = block!(rx1.read()).unwrap_or(b'?');
        block!(tx0.write(ch)).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
