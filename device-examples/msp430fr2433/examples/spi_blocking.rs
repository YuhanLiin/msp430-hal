//! An SPI master with the blocking `SpiBus` interface of embedded-hal and a GPIO as the chip select:
//! once a second it sends five bytes, 12h 00h 00h 34h 56h, with the chip select low, and LED1 toggles.
//!
//! eUSCI_A1 is a 3-pin master in SPI mode 0 (the clock idles low, and data is captured on its rising
//! edges), MSB first, at 500 kHz from SMCLK. `write` sends 12h, `read` sends 00h twice to read two bytes,
//! and `transfer` sends 34h and 56h. P1.3, a GPIO, is the chip select.
//! (eUSCI_A1's SPI pins: SLASE59F Table 6-10, p. 49. SPI mode 0: SLAU445I Table 23-3, p. 613. LED1 on
//! P1.0 is red: SLAU739 Figure 18, p. 23.)
//!
//! How to test (the scope):
//! 1. Flash this example. Expected: LED1 toggles every second.
//! 2. Scope, ground on GND (J3 pin 22), 50 µs/div, trigger on CH3 falling: CH1 on SCLK, P2.4 (J1 pin 7),
//!    CH2 on MOSI, P2.6 (J2 pin 15), CH3 on the chip select, P1.3 (J1 pin 9). Expected once a second:
//!    the chip select low for 40 clock pulses, while MOSI sends 12h, 00h, 00h, 34h and 56h, MSB first.
//!    The scope's SPI decoder (Analysis > Decode) shows them as bytes.
//! (Header pins: SLAU739 Figure 18, p. 23.)
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}, spi::MODE_0};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, spi::SpiConfig, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Hold the watchdog (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2,
    // p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .config_pin3(|p| p.to_output())
        .split(&pmm);
    // P2.4 UCA1CLK, P2.5 UCA1SOMI, P2.6 UCA1SIMO: P2SELx = 01 (SLASE59F Table 6-10, p. 49; SLASE59F
    // Table 6-19, p. 58). eUSCI_A0's pins would include P1.4 and P1.5, the backchannel UART's (SLAU739
    // Figure 18, p. 23).
    let p2 = Batch::new(periph.p2)
        .config_pin4(|p| p.to_alternate1())
        .config_pin5(|p| p.to_alternate1())
        .config_pin6(|p| p.to_alternate1())
        .split(&pmm);
    // LaunchPad header pins: SCK J1 pin 7, MISO J2 pin 14, MOSI J2 pin 15, CS J1 pin 9 (SLAU739
    // Figure 18, p. 23)
    let sck    = p2.pin4;
    let miso   = p2.pin5;
    let mosi   = p2.pin6;
    let mut cs = p1.pin3;
    cs.set_high().ok();
    let mut red_led = p1.pin0; // Red LED1 (SLAU739 Figure 18, p. 23)
    red_led.set_low().ok();

    // MCLK = SMCLK = about 8 MHz: DCORSEL = 011b with the FLL locked to REFO (SLAU445I Table 3-5, p. 114;
    // SLAU445I 3.2.5, p. 104), DIVM and DIVS /1 (SLAU445I Table 3-9, p. 118); no FRAM wait state is needed
    // up to 8 MHz (SLASE59F 5.3, p. 16). ACLK = REFO: SELA = 01b (SLAU445I Table 3-8, p. 117).
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // SPI mode 0 is UCCKPH = 1, UCCKPL = 0, and `true` sends the MSB first, UCMSB = 1; master clocked from
    // SMCLK: UCMST = 1, UCSSELx = 10b (SLAU445I Table 23-3, p. 613). Without STE it runs as a 3-pin master,
    // UCMODEx = 00b (SLAU445I Table 23-3, p. 613; SLAU445I 23.3.3, p. 607).
    let mut spi = SpiConfig::new(periph.e_usci_a1, MODE_0, true)
        .to_master_using_smclk(&smclk, 16) // 8MHz / 16 = 500kHz (SLAU445I 23.3.6, Equation 15, p. 609)
        .single_master_bus(miso, mosi, sck);

    // The embedded-hal ecosystem includes multiple SPI traits:
    // embedded-hal contains the basic SpiBus trait, which models just the SPI bus (good for devices with no chip select pin).
    // embedded-hal also contains SpiDevice, which facilitates automatic chip select pin management and bus sharing. embedded-hal-bus provides common implementations of SpiDevice.
    // embedded-hal-nb contains a non-blocking interface through spi::FullDuplex.

    // For simplicity we'll use SpiBus here.
    use embedded_hal::spi::SpiBus;

    loop {
        // Perform the following transaction:
        // Send: 0x12, 0x00,    0x00,    0x34,    0x56,
        // Recv: N/A,  recv[0], recv[1], recv[2], N/A
        let mut recv = [0; 3];
        cs.set_low().ok();
            // These methods do return errors, but because we haven't used the non-blocking
            // API (from embedded-hal-nb) or interrupts the Rx buffer should never overrun because
            // the blocking interface automatically reads after every write.
            // (UCOE is set when a character reaches UCxRXBUF before the previous one was read: SLAU445I
            // 23.4.3, Table 23-5, p. 615. A master receives only while it sends, "because receive and
            // transmit operations operate concurrently": SLAU445I 23.3.3, p. 607.)
            spi.write(&[0x12]).unwrap();
            spi.read(&mut recv[0..2]).unwrap();
            spi.transfer(&mut recv[2..], &[0x34, 0x56]).unwrap();
            spi.flush().unwrap();
        cs.set_high().ok();

        red_led.toggle().ok();
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
