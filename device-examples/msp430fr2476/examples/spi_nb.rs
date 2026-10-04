//! An SPI master with the non-blocking `FullDuplex` interface of embedded-hal-nb and a GPIO as the chip
//! select: once a second it sends two bytes, AAh and FFh, with the chip select low.
//!
//! eUSCI_A0, on its remapped pins, is a 3-pin master in SPI mode 0 (the clock idles low, and data is
//! captured on its rising edges), MSB first, at 500 kHz from SMCLK. Each `write` of a byte is followed by
//! a `read` of the byte that came in meanwhile. With more than one slave, "the software needs to use
//! general-purpose I/O pins instead to generate STE signals", so P1.3 is the chip select.
//! (eUSCI_A0's remapped pins, USCIA0RMP = 1: SLASEO7C Table 9-11, p. 54. SPI mode 0: SLAU445I
//! Table 23-3, p. 613. GPIO chip selects: SLAU445I 23.3.3.2, p. 608. P5.0 and P5.1 also drive LED2
//! through J8: SLAU802 Figure 19, p. 25.)
//!
//! How to test (the scope):
//! 1. Take the J8 jumpers marked P5.0 and P5.1 off, so that LED2 doesn't load SCLK and MISO.
//! 2. Flash this example.
//! 3. Scope, ground on GND (J3 pin 22), 20 µs/div, trigger on CH3 falling: CH1 on SCLK, P5.0 (J4 pin 38),
//!    CH2 on MOSI, P5.2 (J4 pin 40), CH3 on the chip select, P1.3 (J1 pin 9). Expected once a second:
//!    the chip select low for 16 clock pulses, while MOSI sends AAh and FFh (10101010 and 11111111),
//!    MSB first. The scope's SPI decoder (Analysis > Decode) shows them as bytes.
//! 4. Put the J8 jumpers back.
//! (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::MODE_0};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::RemappedMapping, pmm::Pmm, spi::{Spi, SpiConfig}, watchdog::Wdt
};
use nb::block;
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    // USCIA0RMP is bit 0 of SYSCFG3 (SLAU445I Table 1-32, p. 83).
    // eUSCI_A0 remapped (USCIA0RMP): P5.2 = UCA0SIMO, P5.1 = UCA0SOMI and P5.0 = UCA0CLK with
    // P5SEL = 01 (SLASEO7C Table 9-11, p. 54; SLASEO7C Table 9-27, p. 69), on J4 pins 40, 39 and 38;
    // CS is P1.3, J1 pin 9 (SLAU802 Figure 10, p. 13). P5.0 and P5.1 also drive the green and red
    // parts of LED2 through J8 (SLAU802 Figure 19, p. 25).
    let mosi   = p5.pin2.to_alternate1();
    let miso   = p5.pin1.to_alternate1();
    let sck    = p5.pin0.to_alternate1();
    let mut cs = p1.pin3.to_output();
    cs.set_high().ok();

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range (SELMS = 000b: SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I
    // Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the data sheet.
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // MODE_0 captures data on the first clock edge with the clock idle low (UCCKPH = 1, UCCKPL = 0),
    // `true` sends the MSB first (UCMSB = 1), and UCMST = 1 makes it the master (SLAU445I Table 23-3,
    // p. 613). SMCLK is UCSSEL = 10b (SLASEO7C Table 9-8, p. 50), and fBitClock = fBRCLK / UCBRx
    // (SLAU445I 23.3.6, p. 609).
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
        // (Receive and transmit run concurrently: SLAU445I 23.3.3, p. 607)
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
