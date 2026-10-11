//! eCOMP edge interrupts: the comparator compares P2.2 with its 1.2 V reference, and its interrupt counts
//! the rising and falling edges of the output. Once a second the backchannel UART prints the counts,
//! which for a signal crossing 1.2 V once each way per period are its frequency.
//!
//! How to test (function generator):
//! 1. Generator: square wave, 100 Hz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load High-Z. Check the
//!    levels on the scope first.
//! 2. Connect it to P2.2 (J1 pin 5), its ground to GND (J3 pin 22).
//! 3. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud.
//! 4. Expected: `100 rising, 100 falling edges per second`. Try other frequencies up to about 20 kHz, and
//!    a sine wave, 1 Vpp around 1.2 V: the 20 mV hysteresis keeps the noise near 1.2 V from adding edges.
//!    With the generator off, both counts are 0.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::{Cell, RefCell};
use critical_section::with;
use embedded_hal::delay::DelayNs;
use embedded_io::Write;
use msp430::interrupt::{enable as enable_interrupts, Mutex};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    ecomp::{Comparator, ComparatorVector, ECompConfig, FilterStrength, Hysteresis, NegativeInput, OutputPolarity, PositiveInput, PowerMode},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use msp430fr247x::{interrupt, EComp0};
use panic_msp430 as _;

static COMPARATOR: Mutex<RefCell<Option<Comparator<EComp0>>>> = Mutex::new(RefCell::new(None));
/// Rising and falling edges since the last report
static EDGES: Mutex<Cell<(u16, u16)>> = Mutex::new(Cell::new((0, 0)));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);

    // 8 MHz MCLK and SMCLK, so the interrupt keeps up with fast signals
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART, TXD on P1.4
    let mut tx = SerialConfig::<_, _, DefaultMapping>::new(
        periph.e_usci_a0,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        9600,
    )
    .use_smclk(&smclk)
    .tx_only(p1.pin4.to_alternate1());

    // The output is high while P2.2 is above the 1.2 V reference, with 20 mV of hysteresis against noise
    let (_dac_config, comparator_config) = ECompConfig::begin(periph.e_comp0);
    let mut comparator = comparator_config
        .configure(
            PositiveInput::COMPx_1(p2.pin2.to_alternate3()),
            NegativeInput::_1V2,
            OutputPolarity::Noninverted,
            PowerMode::HighSpeed,
            Hysteresis::_20mV,
            FilterStrength::Off,
        )
        .no_output_pin();

    // Configuring can set the edge flags, so clear them before enabling their interrupts
    comparator.clear_edge_flags();
    comparator.enable_rising_interrupts();
    comparator.enable_falling_interrupts();
    with(|cs| COMPARATOR.borrow_ref_mut(cs).replace(comparator));

    unsafe { enable_interrupts() };

    loop {
        delay.delay_ms(1000);
        let (rising, falling) = with(|cs| EDGES.borrow(cs).replace((0, 0)));
        writeln!(tx, "{} rising, {} falling edges per second\r", rising, falling).ok();
    }
}

#[interrupt]
fn ECOMP0() {
    with(|cs| {
        if let Some(comparator) = COMPARATOR.borrow_ref_mut(cs).as_mut() {
            let edges = EDGES.borrow(cs);
            loop {
                let (rising, falling) = edges.get();
                match comparator.interrupt_source() {
                    ComparatorVector::RisingEdge => edges.set((rising.wrapping_add(1), falling)),
                    ComparatorVector::FallingEdge => edges.set((rising, falling.wrapping_add(1))),
                    ComparatorVector::None => break,
                }
            }
        }
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
