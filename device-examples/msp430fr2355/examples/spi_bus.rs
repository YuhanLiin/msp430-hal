#![no_main]
#![no_std]

// This example uses the SpiBus embedded-hal interface, with a software controlled CS pin.

use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::MODE_0};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, spi::SpiConfig, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    // UCA0SIMO on P1.7, UCA0SOMI on P1.6 and UCA0CLK on P1.5, P1SELx = 01 (SLASEC4D Table 6-14, p. 72;
    // SLASEC4D Table 6-63, p. 96). They are pins 4, 3 and 2 of the BoosterPack header, and P1.4, the
    // chip select, is pin 23 (SLAU680 Figure 10, p. 15).
    let mosi   = p1.pin7.to_alternate1();
    let miso   = p1.pin6.to_alternate1();
    let sck    = p1.pin5.to_alternate1();
    let mut cs = p1.pin4.to_output();
    cs.set_high().ok();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // In single master mode SCK and MOSI are always outputs.
    // Multi-master mode allows another master to control whether this device's SCK
    // and MOSI pins are outputs or high impedance via the STE pin.
    // (SLAU445I 23.3.3.1, p. 608: with UCxSTE master-inactive, "UCxSIMO and UCxCLK are set to inputs and
    // no longer drive the bus". See also SLAZ695J USCI50 for that 4-pin master mode.)
    let mut spi = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        // fBitClock = fBRCLK / UCBRx (SLAU445I 23.3.6, Equation 15, p. 609)
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
        // (UCOE is set "when a character is transferred into UCxRXBUF before the previous character was
        // read": SLAU445I 23.4.3, p. 615)
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
