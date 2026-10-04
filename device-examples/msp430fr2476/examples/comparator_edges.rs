//! eCOMP edge interrupts: the comparator compares P2.2 with its 1.2 V reference, and its interrupt counts
//! the rising and falling edges of the output. Once a second the backchannel UART prints the counts,
//! which for a signal crossing 1.2 V once each way per period are its frequency.
//! (eCOMP0's inputs: COMP0.1 is P2.2 and the 1.2 V reference is channel 010b: SLASEO7C Table 9-21, p. 63.
//! Edge interrupts and CP0IV: SLAU445I 18.3, p. 507; SLAU445I Table 18-5, p. 512. Header pins: SLAU802
//! Figure 10, p. 13.)
//!
//! A high eCOMP0 output also switches all TB0 outputs to high impedance after reset (SLASEO7C Table 9-17,
//! p. 61), which doesn't matter here as TB0 isn't used; see `timer_b_high_impedance.rs`.
//!
//! How to test (function generator):
//! 1. Generator: square wave, 100 Hz, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load High-Z. Check the
//!    levels on the scope first.
//! 2. Connect it to P2.2 (J1 pin 5), its ground to GND (J3 pin 22).
//! 3. Flash this example, with the TXD jumper of J101 on, and open the COM port of "MSP Application
//!    UART1" at 9600 baud (SLAU802 2.2.4, p. 9).
//! 4. Expected: `100 rising, 100 falling edges per second`. Try a sine wave too, 1 Vpp around 1.2 V: the
//!    20 mV hysteresis keeps the noise near 1.2 V from adding edges. With the generator off, both counts
//!    are 0.
//! 5. Try other frequencies. Up to a few hundred hertz the counts match the frequency; higher, they come
//!    out too high, as the time the interrupts take makes the one-second wait longer: it counts CPU
//!    cycles (see the `delay` module documentation).
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
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range, so the interrupt keeps up with fast signals, and ACLK
    // from REFO (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9,
    // p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART: eUSCI_A0's TXD on P1.4, P1SEL = 01, 8N1 (SLAU802 2.2.4, p. 9; SLASEO7C
    // Table 9-23, p. 65; SLAU445I Table 22-8, p. 593)
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

    // V+ is COMP0.1 on P2.2, with P2SEL = 11 (SLASEO7C Table 9-24, p. 66), and V- the low-power 1.2 V
    // reference, so the output is high while P2.2 is above 1.2 V (SLAU445I 18.2.1, p. 505). High-speed
    // mode (CPMSEL = 0) with 20 mV of hysteresis (CPHSEL = 10b): SLAU445I Table 18-3, p. 510.
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

    // Configuring can set the edge flags, so clear them before enabling their interrupts (SLAU445I
    // Table 18-3, p. 510: "Changing CPFLT might set interrupt flag"). CPIE and CPIIE request the interrupt
    // for rising and falling edges (SLAU445I Table 18-3, p. 510).
    comparator.clear_edge_flags();
    comparator.enable_rising_interrupts();
    comparator.enable_falling_interrupts();
    with(|cs| COMPARATOR.borrow_ref_mut(cs).replace(comparator));

    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { enable_interrupts() };

    loop {
        // Count the edges of one second, leaving out the time the printing takes
        with(|cs| EDGES.borrow(cs).set((0, 0)));
        delay.delay_ms(1000);
        let (rising, falling) = with(|cs| EDGES.borrow(cs).get());
        writeln!(tx, "{} rising, {} falling edges per second\r", rising, falling).ok();
    }
}

// The eCOMP0 vector, CPIFG and CPIIFG through CP0IV (FFCAh: SLASEO7C Table 9-2, p. 47). Reading CP0IV
// returns the highest-priority pending flag and clears it (SLAU445I Table 18-5, p. 512).
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
