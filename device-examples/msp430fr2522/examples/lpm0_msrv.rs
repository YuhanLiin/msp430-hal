#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// NOTE: Historically there was no way to return the CPU to active mode after entering a low power mode,
// the MSP restores the CPU to active mode during an interrupt but turns it off afterwards.

// A feature was recently added to msp430-rt to allow the CPU to return to active mode, but it depends on Rust 1.88.
// For compatibility with the MSRV of this crate we showcase the old implementation here, with all work being done inside the interrupt.
// For the more flexible new version, see lpm0.rs

use critical_section::with;
use msp430fr25x2::{interrupt, P1, P2};

use core::cell::RefCell;
use embedded_hal::digital::*;
use msp430::{
    asm,
    interrupt::{enable as enable_interrupts, Mutex},
};
use msp430_rt::entry;
use msp430_hal::{
    gpio::{Batch, GpioVector, Output, Pin, Pin0, PxIV},
    lpm::enter_lpm0,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));
static RED_LED: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));

// P1.0 should toggle when P2.3 is pressed
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let _wdt = Wdt::constrain(periph.wdt_a);
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    let p1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let red_led = p1.pin0;

    let p2 = Batch::new(periph.p2)
        .pulldown_all()
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let mut button = p2.pin3;
    let p2iv = p2.pxiv;

    with(|cs| {
        P2IV.borrow_ref_mut(cs).replace(p2iv);
        RED_LED.borrow_ref_mut(cs).replace(red_led);
    });

    button.select_falling_edge_trigger().enable_interrupts();

    unsafe { enable_interrupts() };

    // Since no peripherals were configured to use SMCLK / ACLK we could just as well enter LPM3 / LPM4 here
    enter_lpm0();

    loop {}
}

// The CPU will wake up to handle interrupts, but will be put back to sleep afterwards.
#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {
            return;
        };
        let Some(ref mut red_led) = *RED_LED.borrow_ref_mut(cs) else {
            return;
        };
        if let GpioVector::Pin3Isr = p2iv.get_interrupt_vector() {
            red_led.toggle().ok();
            for _ in 0..15_000 {
                // Debouncing
                asm::nop();
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
