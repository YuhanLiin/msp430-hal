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

// Prints "HELLO" when started then echos on eUSCI_A0
// Serial settings are listed in the code
#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr25x2::Peripherals::take() {
        let mut fram = Fram::new(periph.frctl);
        let _wdt = Wdt::constrain(periph.wdt_a);

        let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);

        let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

        let p1 = Batch::new(periph.p1).split(&pmm);
        let p2 = Batch::new(periph.p2).split(&pmm);

        let mut led = p1.pin0.to_output();
        led.set_low().ok();

        let mut e_usci_a0 = periph.e_usci_a0;

        // FIRST: Default UART mapping (P1.4 TX / P1.5 RX)
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

        // SECOND: Remap UART to P2.0 TX / P2.1 RX
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
