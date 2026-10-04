//! defmt logging over the backchannel UART: once a second the example logs `Hello!` with
//! `defmt::println!`, and defmt-print on the PC shows it.
//!
//! defmt-serial sends defmt's frames on eUSCI_A1's TXD, P4.3, at 9600 baud, 8 data bits, no parity and
//! one stop bit, clocked by ACLK from REFO. The frames are binary, so a terminal shows them as garbage:
//! defmt-print decodes them with this example's ELF file, which holds the strings. defmt also needs its
//! linker script, defmt.x, which this crate's .cargo/config.toml adds. LED1 lights once the UART is set
//! up.
//! (The backchannel UART is eUSCI_A1: SLAU680 2.2.4, p. 11. UCA1TXD is P4.3: SLASEC4D Table 6-14, p. 72.
//! LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test (defmt-print on the PC):
//! 1. Flash this example, with the TXD jumper of J101 on. Expected: LED1 lights.
//! 2. Feed the bytes of the COM port of "MSP Application UART1" to defmt-print, at 9600 baud (SLAU680
//!    2.2.4, p. 11). In a shell, from this crate's folder, with `<port>` the COM port's device file (in
//!    Git Bash, COMn is `/dev/ttyS<n-1>`):
//!    `stty -F <port> 9600 raw`
//!    `cat <port> | defmt-print -w -e ./target/msp430-none-elf/debug/examples/defmt`
//! 3. Expected: `Hello!` once a second.
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
use msp430fr2355::EUsciA1;
use panic_msp430 as _;
use static_cell::StaticCell;

// Once configured, our UART peripheral will live here.
// This allows for printing from anywhere, including interrupts and panics.
static SERIAL: StaticCell<Tx<EUsciA1>> = StaticCell::new();

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let mut fram = Fram::new(periph.frctl);
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);
    let mut led = p1.pin0.to_output();
    led.set_low().ok();

    let (_smclk, aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let tx = SerialConfig::new(
        periph.e_usci_a1,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        9600,
    )
    .use_aclk(&aclk)
    .tx_only(p4.pin3.to_alternate1()); // UCA1TXD, P4SELx = 01 (SLASEC4D Table 6-66, p. 102)

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
