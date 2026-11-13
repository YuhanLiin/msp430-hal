#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// This example demonstrates an SPI slave using interrupts.
// Another eUSCI peripheral is configured as an SPI master to drive the bus.
// P6.6 (green LED) should turn on and stay on
// P1.0 (red LED) should blink with each sent SPI transaction

// Connect:
// P1.7 <--> P4.6,
// P1.6 <--> P4.7,
// P1.5 <--> P4.5,
// P1.4 <--> P4.4

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
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let p3 = Batch::new(periph.p3).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    let sl_mosi = p5.pin2.to_alternate1();
    let sl_miso = p5.pin1.to_alternate1();
    let sl_sclk = p5.pin0.to_alternate1();
    let sl_ste  = p4.pin7.to_alternate1();

    let mosi = p3.pin2.to_alternate1();
    let miso = p3.pin6.to_alternate1();
    let sclk = p3.pin5.to_alternate1();
    let mut ste = p2.pin7.to_output();
    ste.set_high().ok();

    // LED1 on P1.0 is green, whatever the variable name says
    let mut led1 = p1.pin0.to_output();
    // let mut green_led = Batch::new(periph.p6).split(&pmm).pin6.to_output();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // Configure a peripheral as an SPI slave.
    // It can be configured for either a shared or exclusive bus depending on whether
    // there are other slaves on the bus. On an exclusive bus MISO is always an output.
    // On a shared bus the STE pin is used to control whether this slave's MISO is an output or high impedance pin.
    let mut spi_slave = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        .to_slave()
        .shared_bus(sl_miso, sl_mosi, sl_sclk, sl_ste, StePolarity::EnabledWhenLow);

    // Configure another as an SPI master to drive the bus.
    let mut spi: Spi<_, DefaultMapping> = SpiConfig::new(periph.e_usci_b1, MODE_0, true)
        .to_master_using_smclk(&smclk, 800) // 8MHz / 80 = 100kHz
        .single_master_bus(miso, mosi, sclk);

    critical_section::with(|cs| {
        spi_slave.set_rx_interrupt();
        SPI_SLAVE.replace(cs, Some(spi_slave));
    });

    unsafe { msp430::interrupt::enable() };

    loop {
        let mut recv_buf = [0; 4];
        let send_buf = [12, 14, 0xFF];

        ste.set_low().ok(); // Enable slave MISO

        // Can return Err, but both error types aren't relevant here.
        let _ = spi.transfer(&mut recv_buf, &send_buf);
        let _ = spi.flush();

        ste.set_high().ok(); // Make slave MISO high impedance

        // Green LED on if result matches expected (LED1)
        led1.set_state( (recv_buf[1..] == [13, 15, 00]).into() ).ok();
        // led1.toggle().ok();

        delay.delay_ms(1000);
    }
}

static SPI_SLAVE: Mutex<RefCell<Option<SpiSlave<EUsciA0, RemappedMapping>>>> = Mutex::new(RefCell::new(None));

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
