#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, Smclk, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::PinMap, pmm::Pmm, serial::*, watchdog::Wdt
};
use nb::block;
use panic_msp430 as _;

fn setup_uart<S, M>(
    usci: S,
    tx: S::TxPin,
    rx: S::RxPin,
    parity: Parity,
    loopback: Loopback,
    baudrate: u32,
    smclk: &Smclk,
) -> (Tx<S, M>, Rx<S, M>)
where
    S: SerialUsci<M>,
    M: PinMap,
{
    SerialConfig::new(
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
// (UART1, eUSCI_A1, is the backchannel UART to the host: SLAU680 2.2.4, p. 11. UART0 runs in loopback
// mode, in which "UCAxTXD is internally fed back to the receiver": SLAU445I 22.4.5, p. 596.)
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    let mut fram = Fram::new(periph.frctl);
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_4MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let mut led = p1.pin0.to_output();
    led.set_low().ok();

    // UCA0TXD on P1.7 and UCA0RXD on P1.6, P1SELx = 01 (SLASEC4D Table 6-63, p. 96)
    let (mut tx0, mut rx0) = setup_uart(
        periph.e_usci_a0,
        p1.pin7.to_alternate1().into(),
        p1.pin6.to_alternate1().into(),
        Parity::EvenParity,
        Loopback::Loopback,
        20000,
        &smclk,
    );

    // UCA1TXD on P4.3 and UCA1RXD on P4.2, P4SELx = 01 (SLASEC4D Table 6-66, p. 102), wired to the
    // eZ-FET backchannel (SLAU680 Figure 18, p. 26)
    let (mut tx1, mut rx1) = setup_uart(
        periph.e_usci_a1,
        p4.pin3.to_alternate1().into(),
        p4.pin2.to_alternate1().into(),
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
