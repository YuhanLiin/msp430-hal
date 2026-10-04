#![no_main]
#![no_std]

use defmt_serial::defmt_serial;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram, 
    gpio::*, 
    pmm::Pmm, 
    serial::*, 
    watchdog::Wdt
};
use msp430fr2355::EUsciA0;
use panic_msp430 as _;
use static_cell::StaticCell;

// Configure UART, then print "Hello!" over eUSCI_A0 using defmt once per second.
// eUSCI_A0's TXD is P1.7, pin 4 of the BoosterPack header (SLAU680 Figure 10, p. 15). The LaunchPad's
// own backchannel UART is eUSCI_A1 (SLAU680 2.2.4, p. 11), so this needs a separate adapter.

// Messages can be received using (a serial to USB adapter and) `defmt-print`, e.g.:
// stty -F /dev/ttyUSB0 9600 raw; cat /dev/ttyUSB0 | defmt-print -w -e ./target/msp430-none-elf/debug/examples/defmt
// Note: Using defmt also requires adding the defmt linker script to .cargo/config.toml

// Once configured, our UART peripheral will live here.
// This allows for printing from anywhere, including interrupts and panics.
static SERIAL: StaticCell<Tx<EUsciA0>> = StaticCell::new();

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let mut fram = Fram::new(periph.frctl);
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut led = p1.pin0.to_output();
    led.set_low().ok();

    let (_smclk, aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let tx = SerialConfig::new(
        periph.e_usci_a0,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        9600,
    )
    .use_aclk(&aclk)
    .tx_only(p1.pin7.to_alternate1()); // UCA0TXD, P1SELx = 01 (SLASEC4D Table 6-63, p. 96)

    // Tell defmt to use our serial peripheral
    defmt_serial(SERIAL.init(tx));

    led.set_high();
    loop {
        delay.delay_ms(1000);
        defmt::println!("Hello!");
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
