//! An SPI master with the non-blocking `FullDuplex` interface of embedded-hal-nb and a GPIO as the chip
//! select: once a second it sends two bytes, AAh and FFh, with the chip select low.
//!
//! eUSCI_A0, on its remapped pins, is a 3-pin master in SPI mode 0 (the clock idles low, and data is
//! captured on its rising edges), MSB first, at 500 kHz from SMCLK. Each `write` of a byte is followed by
//! a `read` of the byte that came in meanwhile. With more than one slave, "the software needs to use
//! general-purpose I/O pins instead to generate STE signals", so P1.3 is the chip select. There's no
//! LaunchPad for the MSP430FR25x2.
//! (eUSCI_A0's remapped pins, USCIARMP = 1: SLASEE4C Table 6-11, p. 53. SPI mode 0: SLAU445I Table 23-3,
//! p. 613. GPIO chip selects: SLAU445I 23.3.3.2, p. 608. P2.0 and P2.1 are also XOUT and XIN: SLASEE4C
//! Table 6-16, p. 60.)
//!
//! How to test (the scope):
//! 1. P2.0 and P2.1 must have no crystal on them. Flash this example.
//! 2. Scope, ground on GND, 20 µs/div, trigger on CH3 falling: CH1 on SCLK, P1.6, CH2 on MOSI, P2.0, CH3
//!    on the chip select, P1.3. Expected once a second: the chip select low for 16 clock pulses, while
//!    MOSI sends AAh and FFh (10101010 and 11111111), MSB first. The scope's SPI decoder
//!    (Analysis > Decode) shows them as bytes.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::MODE_0};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::RemappedMapping,
    pmm::Pmm,
    spi::{Spi, SpiConfig},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    // eUSCI_A0 in its remapped mapping, USCIARMP = 1: UCA0SIMO on P2.0 and UCA0SOMI on P2.1 with
    // P2SELx = 01, UCA0CLK on P1.6 with P1SELx = 01 (SLASEE4C Table 6-11, p. 53;
    // SLASEE4C Table 6-16, p. 60; SLASEE4C Table 6-15, p. 58)
    let mosi = p2.pin0.to_alternate1();
    let miso = p2.pin1.to_alternate1();
    let sck = p1.pin6.to_alternate1();
    // CS on P1.3 as a GPIO output, P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58)
    let mut cs = p1.pin3.to_output();
    cs.set_high().ok();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The bit clock is fBRCLK / UCBRx (SLAU445I 23.3.6, p. 609)
    let mut spi: Spi<_, RemappedMapping> = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        .to_master_using_smclk(&smclk, 16) // 8MHz / 16 = 500kHz
        .single_master_bus(miso, mosi, sck);

    loop {
        // Non-blocking interface available through embedded-hal-nb
        use embedded_hal_nb::spi::FullDuplex;

        cs.set_low().ok();

        // Blocking send. Sends out data on the MOSI line
        // Sending is infallible, besides `nb::WouldBlock` when the bus is busy.
        block!(spi.write(0b10101010)).unwrap();

        // Writing on MOSI also shifts in data on MISO - read from the hardware buffer with `.read()`.
        // (SLAU445I 23.3.3, p. 607: "receive and transmit operations operate concurrently")
        // Every successful `.write()` call should be followed by a `.read()`.
        // You should handle errors here rather than unwrapping
        let _ = block!(spi.read()).unwrap();

        // This concludes the first byte of an SPI transaction.

        // Multi-byte transactions are performed by calling the methods repeatedly:
        block!(spi.write(0xFF)).unwrap();
        let _ = block!(spi.read()).unwrap();

        cs.set_high().ok();

        delay.delay_ms(1000);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
