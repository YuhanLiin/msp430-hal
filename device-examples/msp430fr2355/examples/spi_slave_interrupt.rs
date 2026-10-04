#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// This example demonstrates an SPI slave using interrupts.
// Another eUSCI peripheral is configured as an SPI master to drive the bus.
// P6.6 (green LED) should turn on and stay on
// P1.0 (red LED) should blink with each sent SPI transaction
// (LED2, green, on P6.6 and LED1, red, on P1.0: SLAU680 Figure 18, p. 26)

// Connect:
// P1.7 <--> P4.6,
// P1.6 <--> P4.7,
// P1.5 <--> P4.5,
// P1.4 <--> P4.4
// (UCA0SIMO, UCA0SOMI, UCA0CLK and UCA0STE to UCB1SIMO, UCB1SOMI, UCB1CLK and P4.4, which drives STE as a
// GPIO: SLASEC4D Table 6-14, p. 72. On the BoosterPack header that is pin 4 to pin 15, pin 3 to pin 14,
// pin 2 to pin 7 and pin 23 to pin 13: SLAU680 Figure 10, p. 15.)

use core::cell::RefCell;

use critical_section::Mutex;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use embedded_hal::spi::{SpiBus, MODE_0};
use msp430_rt::entry;
use msp430fr2355::{interrupt, EUsciA0};
use msp430_hal::spi::{SpiConfig, SpiErr, SpiSlave, StePolarity};
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, watchdog::Wdt
};

use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    // UCA0SIMO, UCA0SOMI, UCA0CLK and UCA0STE on P1.7 to P1.4, P1SELx = 01 (SLASEC4D Table 6-63, p. 96)
    let sl_mosi = p1.pin7.to_alternate1();
    let sl_miso = p1.pin6.to_alternate1();
    let sl_sclk = p1.pin5.to_alternate1();
    let sl_ste  = p1.pin4.to_alternate1();

    let p4 = Batch::new(periph.p4).split(&pmm);
    // UCB1SIMO on P4.6, UCB1SOMI on P4.7 and UCB1CLK on P4.5, P4SELx = 01 (SLASEC4D Table 6-66, p. 102)
    let mosi = p4.pin6.to_alternate1();
    let miso = p4.pin7.to_alternate1();
    let sclk = p4.pin5.to_alternate1();
    let mut ste = p4.pin4.to_output();
    ste.set_high().ok();

    // LED1, red, on P1.0 and LED2, green, on P6.6 (SLAU680 Figure 18, p. 26)
    let mut red_led = p1.pin0.to_output();
    let mut green_led = Batch::new(periph.p6).split(&pmm).pin6.to_output();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // Configure another peripheral as an SPI master to drive the bus. It is configured before the slave:
    // MODE_0 is UCCKPH = 1, "Data is captured on the first UCLK edge" (SLAU445I 23.4.1, p. 613), and an
    // SPI slave with UCCKPH = 1 goes wrong if its clock pin isn't at the idle level when it leaves reset
    // (SLAZ695J USCI47). The erratum's workaround: "The SPI master must set the clock pin at the
    // appropriate idle level (low for UCCKPL = 0, high for UCCKPL = 1) before SPI slave is reset (UCSWRST
    // bit is cleared)". With the master running, SCLK idles low (UCCKPL = 0 in MODE_0).
    let mut spi = SpiConfig::new(periph.e_usci_b1, MODE_0, true)
        // fBitClock = fBRCLK / UCBRx (SLAU445I 23.3.6, Equation 15, p. 609)
        .to_master_using_smclk(&smclk, 800) // 8MHz / 800 = 10kHz
        .single_master_bus(miso, mosi, sclk);

    // Configure a peripheral as an SPI slave.
    // It can be configured for either a shared or exclusive bus depending on whether
    // there are other slaves on the bus. On an exclusive bus MISO is always an output.
    // On a shared bus the STE pin is used to control whether this slave's MISO is an output or high impedance pin.
    // (SLAU445I 23.3.4.1, p. 609: with UCxSTE slave-inactive "UCxSOMI is set to the input direction", and
    // "The UCxSTE input signal is not used in 3-pin slave mode".)
    let mut spi_slave = SpiConfig::new(periph.e_usci_a0, MODE_0, true)
        .to_slave()
        .shared_bus(sl_miso, sl_mosi, sl_sclk, sl_ste, StePolarity::EnabledWhenLow);

    critical_section::with(|cs| {
        spi_slave.set_rx_interrupt();
        SPI_SLAVE.replace(cs, Some(spi_slave));
    });

    unsafe { msp430::interrupt::enable() };

    loop {
        let mut recv_buf = [0; 4];
        let send_buf = [12, 14, 0xFF];

        ste.set_low().ok(); // Enable slave MISO (STE active low, UCMODEx = 10b: SLAU445I Table 23-1, p. 606)

        // Can return Err, but both error types aren't relevant here.
        let _ = spi.transfer(&mut recv_buf, &send_buf);
        let _ = spi.flush();

        ste.set_high().ok(); // Make slave MISO high impedance

        // Green LED on if result matches expected
        // (LED2, green, on P6.6 and LED1, red, on P1.0: SLAU680 Figure 18, p. 26)
        green_led.set_state( (recv_buf[1..] == [13, 15, 00]).into() ).ok();
        red_led.toggle().ok();

        delay.delay_ms(1000);
    }
}

static SPI_SLAVE: Mutex<RefCell<Option< SpiSlave<EUsciA0> >>> = Mutex::new(RefCell::new(None));

// The eUSCI_A0 receive or transmit vector at FFE4h (SLASEC4D Table 6-2, p. 63)
#[interrupt]
fn EUSCI_A0() {
    critical_section::with(|cs| {
        let Some(ref mut spi_slave) = *SPI_SLAVE.borrow_ref_mut(cs) else {return};
        // If you have multiple interrupts enabled you can use .interrupt_source() to determine which one caused this interrupt
        // (UCRXIFG is set when data moves from the RX shift register to UCxRXBUF: SLAU445I 23.3.4, p. 608)
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
