#![no_main]
#![no_std]

// This example uses the SpiBus embedded-hal interface, with a software controlled CS pin.
// The CS pin is a GPIO output, as the user's guide suggests for slave selects the eUSCI's STE can't make
// (SLAU445I 23.3.3.2, p. 608: "use general-purpose I/O pins instead to generate STE signals").

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

    // In single master mode SCK and MOSI are always outputs.
    // Multi-master mode allows another master to control whether this device's SCK
    // and MOSI pins are outputs or high impedance via the STE pin.
    // (SLAU445I 23.3.3.1, p. 608: with STE master-inactive, "UCxSIMO and UCxCLK are set to inputs"; that
    // mode has erratum SLAZ705H USCI50.) The bit clock is fBRCLK / UCBRx (SLAU445I 23.3.6, p. 609).
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
        // (Overrun, UCOE: "set when a character is transferred into UCxRXBUF before the previous character
        // was read", SLAU445I Table 23-5, p. 615)
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
