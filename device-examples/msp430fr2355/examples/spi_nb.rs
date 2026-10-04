#![no_main]
#![no_std]

// This example uses the non-blocking interface from embedded-hal-nb, with a software controlled CS pin.

use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::MODE_0};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, spi::SpiConfig, watchdog::Wdt
};
use nb::block;
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

    let mut spi = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        // fBitClock = fBRCLK / UCBRx (SLAU445I 23.3.6, Equation 15, p. 609)
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
        // ("Data on UCxSOMI is shifted into the receive shift register on the opposite clock edge":
        // SLAU445I 23.3.3, p. 607)
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
