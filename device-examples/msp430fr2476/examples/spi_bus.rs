//! An SPI master with the blocking `SpiBus` interface of embedded-hal and a GPIO as the chip select:
//! once a second it sends five bytes, 12h 00h 00h 34h 56h, with the chip select low.
//!
//! eUSCI_A0, on its remapped pins, is a 3-pin master in SPI mode 0 (the clock idles low, and data is
//! captured on its rising edges), MSB first, at 500 kHz from SMCLK. `write` sends 12h, `read` sends 00h
//! twice to read two bytes, and `transfer` sends 34h and 56h. With more than one slave, "the software
//! needs to use general-purpose I/O pins instead to generate STE signals", so P1.3 is the chip select.
//! (eUSCI_A0's remapped pins, USCIA0RMP = 1: SLASEO7C Table 9-11, p. 54. SPI mode 0: SLAU445I
//! Table 23-3, p. 613. GPIO chip selects: SLAU445I 23.3.3.2, p. 608. P5.0 and P5.1 also drive LED2
//! through J8: SLAU802 Figure 19, p. 25.)
//!
//! How to test (the scope):
//! 1. Take the J8 jumpers marked P5.0 and P5.1 off, so that LED2 doesn't load SCLK and MISO.
//! 2. Flash this example.
//! 3. Scope, ground on GND (J3 pin 22), 50 µs/div, trigger on CH3 falling: CH1 on SCLK, P5.0 (J4 pin 38),
//!    CH2 on MOSI, P5.2 (J4 pin 40), CH3 on the chip select, P1.3 (J1 pin 9). Expected once a second:
//!    the chip select low for 40 clock pulses, while MOSI sends 12h, 00h, 00h, 34h and 56h, MSB first.
//!    The scope's SPI decoder (Analysis > Decode) shows them as bytes.
//! 4. Put the J8 jumpers back.
//! (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::MODE_0};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::RemappedMapping, pmm::Pmm, spi::{Spi, SpiConfig}, watchdog::Wdt
};
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

    // In single master mode SCK and MOSI are always outputs.
    // Multi-master mode allows another master to control whether this device's SCK
    // and MOSI pins are outputs or high impedance via the STE pin.
    // (SLAU445I 23.3.3.1, p. 608. In that mode, only write the TX buffer while STE is active:
    // SLAZ726B USCI50, p. 8 to p. 9.)
    // MODE_0 captures data on the first clock edge with the clock idle low (UCCKPH = 1, UCCKPL = 0),
    // `true` sends the MSB first (UCMSB = 1), and UCMST = 1 makes it the master (SLAU445I Table 23-3,
    // p. 613). SMCLK is UCSSEL = 10b (SLASEO7C Table 9-8, p. 50), and fBitClock = fBRCLK / UCBRx
    // (SLAU445I 23.3.6, p. 609).
    let mut spi: Spi<_, RemappedMapping> = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        .to_master_using_smclk(&smclk, 16) // 8MHz / 16 = 500kHz
        .single_master_bus(miso, mosi, sck);

    loop {
        // Blocking interface available through embedded-hal trait
        use embedded_hal::spi::SpiBus;

        // Perform the following transaction:
        // Send: 0x12, 0x00,    0x00,    0x34,    0x56,
        // Recv: N/A,  recv[0], recv[1], recv[2], N/A
        let mut recv = [0; 3];
        cs.set_low().ok();

        // These methods do return errors, but because we haven't used the non-blocking
        // API (from embedded-hal-nb) or interrupts the Rx buffer should never overrun because
        // the blocking interface automatically reads after every write.
        // (UCOE is set when a character arrives before the previous one was read: SLAU445I 23.4.3,
        // p. 615)
        spi.write(&[0x12]).unwrap();
        spi.read(&mut recv[0..2]).unwrap();
        spi.transfer(&mut recv[2..], &[0x34, 0x56]).unwrap();

        spi.flush().unwrap();
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
