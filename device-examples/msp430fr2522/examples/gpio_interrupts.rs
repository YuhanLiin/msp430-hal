#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use critical_section::with;
use msp430fr25x2::interrupt;

use core::cell::RefCell;
use embedded_hal::digital::*;
use msp430::interrupt::{enable as enable_int, Mutex};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Batch, GpioVector, Output, Pin, Pin0, PxIV, P1, P2},
    pmm::Pmm,
    watchdog::{Wdt, WdtClkPeriods},
};
use nb::block;
use panic_msp430 as _;

static RED_LED: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));

// Red LED should blink 1 second on, 1 second off
// The red LED also toggles when the P2.3 button is pressed
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    let mut wdt = Wdt::constrain(periph.wdt_a).to_interval();

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1) // 32 kHz MCLK
        .smclk_on(SmclkDiv::_2) // 16 kHz SMCLK
        .aclk_refoclk()
        .freeze(&mut Fram::new(periph.frctl));

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);

    let red_led = p1.pin0.to_output();
    // Onboard button with interrupt disabled
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually.
    let mut pin = p2.pin6.pulldown();
    let p2iv = p2.pxiv;

    with(|cs| RED_LED.borrow_ref_mut(cs).replace(red_led));
    with(|cs| P2IV.borrow_ref_mut(cs).replace(p2iv));

    wdt.set_aclk(&aclk)
        .enable_interrupts()
        .set_interval_and_start(WdtClkPeriods::_32k);
    pin.select_rising_edge_trigger().enable_interrupts();
    button.select_falling_edge_trigger();

    unsafe { enable_int() };

    loop {
        block!(button.wait_for_ifg()).ok();
        pin.set_ifg();
    }
}

#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut red_led) = *RED_LED.borrow_ref_mut(cs) else {
            return;
        };
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {
            return;
        };

        if let GpioVector::Pin6Isr = p2iv.get_interrupt_vector() {
            red_led.toggle().ok();
        }
    });
}

#[interrupt]
fn WDT() {
    with(|cs| {
        RED_LED.borrow_ref_mut(cs).as_mut().map(|red_led| {
            red_led.toggle().ok();
        })
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
