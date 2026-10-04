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

// Echoes serial input on UART1 by roundtripping to UART0
// Only UART1 settings matter for the host
// UART0 is eUSCI_A0 in loopback mode, where its TXD "is internally fed back to the receiver"
// (SLAU445I 22.4.5, p. 596). UART1 is eUSCI_A1 on P2.6 (TXD) and P2.5 (RXD), J1 pins 4 and 3
// (SLAU802 Figure 10, p. 13). The backchannel UART of this LaunchPad is eUSCI_A0, not UART1
// (SLAU802 2.2.4, p. 9). To reach UART1 from the PC, pull the TXD and RXD jumpers off J101 and wire
// their eZ-FET side, the pins nearer the USB connector, to J1 pin 4 (TXD) and J1 pin 3 (RXD)
// (SLAU802 Table 2, p. 8; SLAU802 Figure 16, p. 22; SLAU802 Figure 1, p. 1).
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

    // UART0: P1.4 = UCA0TXD and P1.5 = UCA0RXD with P1SEL = 01 (SLASEO7C Table 9-23, p. 65)
    let (mut tx0, mut rx0) = setup_uart::<_, DefaultMapping>(
        periph.e_usci_a0,
        p1.pin4.to_alternate1().into(),
        p1.pin5.to_alternate1().into(),
        Parity::EvenParity,
        Loopback::Loopback,
        20000,
        &smclk,
    );

    // UART1: P2.6 = UCA1TXD and P2.5 = UCA1RXD with P2SEL = 01 (SLASEO7C Table 9-24, p. 66)
    let (mut tx1, mut rx1) = setup_uart(
        periph.e_usci_a1,
        p2.pin6.to_alternate1().into(),
        p2.pin5.to_alternate1().into(),
        Parity::NoParity,
        Loopback::NoLoop,
        19200,
        &smclk,
    );

    led.set_high().ok();

    loop {
        let ch = block!(rx1.read()).unwrap_or(b'!');
        block!(tx0.write(ch)).ok();
        let ch = block!(rx0.read()).unwrap_or(b'?');
        block!(tx1.write(ch)).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
