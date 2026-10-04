//! An SPI slave that answers in its receive interrupt, with a master on the same chip: once a second
//! eUSCI_B1, the master, sends 12, 14 and 255, and eUSCI_A0, the slave, answers each byte with that byte
//! plus one, during the next byte. LED1 lights while the answers are right.
//!
//! The master clocks four bytes (the fourth is 0) and reads four back; the last three must be 13, 15 and
//! 0. P2.7, a GPIO, drives the slave's STE, which is low during the transfer: only then does the slave
//! drive MISO. Both use SPI mode 0, MSB first, and the master runs at 10 kHz from SMCLK. The slave uses
//! eUSCI_A0's remapped pins, which leaves the backchannel UART's pins alone.
//! (The SPI pins: SLASEO7C Table 9-11, p. 54. STE in 4-pin slave mode: SLAU445I 23.3.4.1, p. 609. LED1
//! on P1.0 is green, and P5.1, P5.0 and P4.7 drive LED2 through J8: SLAU802 Figure 19, p. 25.)
//!
//! How to test (four jumper wires):
//! 1. Take the three jumpers off J8, so that LED2 doesn't load the slave's pins.
//! 2. Connect MOSI, P3.2 (J2 pin 15), to P5.2 (J4 pin 40); MISO, P3.6 (J2 pin 14), to P5.1 (J4 pin 39);
//!    SCLK, P3.5 (J1 pin 7), to P5.0 (J4 pin 38); and STE, P2.7 (J2 pin 12), to P4.7 (J4 pin 37).
//!    (Header pins: SLAU802 Figure 10, p. 13.)
//! 3. Flash this example.
//! 4. Expected: LED1 lights and stays on. Without the wires it stays off.
//! 5. Put the J8 jumpers back.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::RefCell;

use critical_section::Mutex;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{SpiBus, MODE_0};
use msp430_rt::entry;
use msp430_hal::pin_mapping::{DefaultMapping, RemappedMapping};
use msp430fr247x::{interrupt, EUsciA0};
use msp430_hal::spi::{Spi, SpiConfig, SpiErr, SpiSlave, StePolarity};
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, watchdog::Wdt
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
    let p2 = Batch::new(periph.p2).split(&pmm);
    let p3 = Batch::new(periph.p3).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    // Slave, eUSCI_A0 remapped: UCA0SIMO, UCA0SOMI, UCA0CLK and UCA0STE with PxSEL = 01
    // (SLASEO7C Table 9-27, p. 69; SLASEO7C Table 9-26, p. 68)
    let sl_mosi = p5.pin2.to_alternate1();
    let sl_miso = p5.pin1.to_alternate1();
    let sl_sclk = p5.pin0.to_alternate1();
    let sl_ste  = p4.pin7.to_alternate1();

    // Master, eUSCI_B1: UCB1SIMO, UCB1SOMI and UCB1CLK with P3SEL = 01 (SLASEO7C Table 9-25, p. 67).
    // P2.7 stays a GPIO that drives the slave's STE.
    let mosi = p3.pin2.to_alternate1();
    let miso = p3.pin6.to_alternate1();
    let sclk = p3.pin5.to_alternate1();
    let mut ste = p2.pin7.to_output();
    ste.set_high().ok();

    // LED1 on P1.0, which is green (SLAU802 Figure 19, p. 25)
    let mut led1 = p1.pin0.to_output();

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range (SELMS = 000b: SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I
    // Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the data sheet.
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // Configure a peripheral as an SPI slave.
    // It can be configured for either a shared or exclusive bus depending on whether
    // there are other slaves on the bus. On an exclusive bus MISO is always an output.
    // On a shared bus the STE pin is used to control whether this slave's MISO is an output or high impedance pin.
    // (While STE is inactive, "UCxSOMI is set to the input direction": SLAU445I 23.3.4.1, p. 609)
    // UCMST = 0 makes it a slave, and UCMODE = 10b is "4-pin SPI with UCxSTE active low"; MODE_0 and
    // MSB first are UCCKPH = 1, UCCKPL = 0, UCMSB = 1 (SLAU445I Table 23-3, p. 613).
    let mut spi_slave = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        .to_slave()
        .shared_bus(sl_miso, sl_mosi, sl_sclk, sl_ste, StePolarity::EnabledWhenLow);

    // Configure another as an SPI master to drive the bus.
    // (UCMST = 1, same clock phase and polarity: SLAU445I Table 23-12, p. 620. SMCLK is UCSSEL = 10b:
    // SLASEO7C Table 9-8, p. 50; fBitClock = fBRCLK / UCBRx: SLAU445I 23.3.6, p. 609)
    let mut spi: Spi<_, DefaultMapping> = SpiConfig::new(periph.e_usci_b1, MODE_0, true)
        .to_master_using_smclk(&smclk, 800) // 8MHz / 800 = 10kHz
        .single_master_bus(miso, mosi, sclk);

    // UCRXIE enables the receive interrupt (SLAU445I Table 23-8, p. 617)
    critical_section::with(|cs| {
        spi_slave.set_rx_interrupt();
        SPI_SLAVE.replace(cs, Some(spi_slave));
    });

    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { msp430::interrupt::enable() };

    loop {
        let mut recv_buf = [0; 4];
        let send_buf = [12, 14, 0xFF];

        ste.set_low().ok(); // Enable slave MISO

        // Can return Err, but both error types aren't relevant here.
        let _ = spi.transfer(&mut recv_buf, &send_buf);
        let _ = spi.flush();

        ste.set_high().ok(); // Make slave MISO high impedance

        // Green LED on if result matches expected (LED1: SLAU802 Figure 19, p. 25)
        led1.set_state( (recv_buf[1..] == [13, 15, 00]).into() ).ok();

        delay.delay_ms(1000);
    }
}

static SPI_SLAVE: Mutex<RefCell<Option<SpiSlave<EUsciA0, RemappedMapping>>>> = Mutex::new(RefCell::new(None));

// The eUSCI_A0 vector, UCRXIFG and UCTXIFG in SPI mode through UCA0IV (FFE0h: SLASEO7C Table 9-2,
// p. 46). A byte that arrives before the previous one was read sets UCOE (SLAU445I 23.4.3, p. 615),
// reported as `SpiErr::Overrun`.
#[interrupt]
fn EUSCI_A0() {
    critical_section::with(|cs| {
        let Some(ref mut spi_slave) = *SPI_SLAVE.borrow_ref_mut(cs) else {return};
        // If you have multiple interrupts enabled you can use .interrupt_source() to determine which one caused this interrupt
        let byte = match unsafe{spi_slave.read_unchecked()} { // Only Rx interrupts are enabled, so Rx buffer must be ready
            Ok(b) => b,
            Err(SpiErr::Overrun(b)) => b,
        };
        nb::block!( spi_slave.write(byte.wrapping_add(1)) ).unwrap(); // Infallible, safe to unwrap after blocking
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
