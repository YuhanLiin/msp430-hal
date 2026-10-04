#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read, Write};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::DefaultMapping, pmm::Pmm, serial::*, watchdog::Wdt
};

use nb::block;
#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

// Prints "HELLO" when started then echos on the backchannel UART: eUSCI_A0 on P1.4 (TXD) and P1.5
// (RXD) (SLAU802 2.2.4, p. 9; SLAU802 Figure 16, p. 22), the "MSP Application UART1" COM port on the
// PC (SLAU802 2.2.4, p. 10). The RXD and TXD jumpers of J101 must be on (SLAU802 Table 2, p. 8).
// Serial settings are listed in the code
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

        // LED1 (SLAU802 Figure 19, p. 25)
        let mut led = p1.pin0.to_output();

        led.set_low().ok();

        // P1.4 = UCA0TXD and P1.5 = UCA0RXD with P1SEL = 01 (SLASEO7C Table 9-23, p. 65), the default
        // eUSCI_A0 mapping (SLASEO7C Table 9-11, p. 54)
        // (8N1, LSB first: UCMSB, UC7BIT, UCSPB, UCPEN in SLAU445I Table 22-8, p. 593; ACLK is
        // UCSSEL = 01b: SLASEO7C Table 9-8, p. 50)
        let (mut tx, mut rx) = SerialConfig::<_, _, DefaultMapping>::new(
            periph.e_usci_a0,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            // Launchpad UART-to-USB converter doesn't handle parity, so we don't use it
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
            // (The receive errors are the UCPE, UCOE, UCFE and UCBRK flags: SLAU445I Table 22-1, p. 582)
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
