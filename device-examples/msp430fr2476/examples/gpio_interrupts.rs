#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use critical_section::with;
use msp430fr247x::interrupt;

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

static LED1: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));

// LED1 (P1.0), which is green, should blink, toggling about every 3.3 seconds
// LED1 and the blue part of LED2 (P4.7) should both toggle when button S2 (P2.3) is pressed
#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let mut wdt = Wdt::constrain(periph.wdt_a).to_interval();

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1) // 32 kHz MCLK
        .smclk_on(SmclkDiv::_2) // 16 kHz SMCLK
        .aclk_vloclk()
        .freeze(&mut Fram::new(periph.frctl));

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p4 = Batch::new(periph.p4).split(&pmm);

    let led1 = p1.pin0.to_output();
    // Onboard button with interrupt disabled
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually.
    let mut pin = p2.pin7.pulldown();
    let mut led2_blue = p4.pin7.to_output();
    let p2iv = p2.pxiv;

    with(|cs| LED1.borrow_ref_mut(cs).replace(led1));
    with(|cs| P2IV.borrow_ref_mut(cs).replace(p2iv));

    wdt.set_aclk(&aclk)
        .enable_interrupts()
        .set_interval_and_start(WdtClkPeriods::_32k);
    pin.select_rising_edge_trigger().enable_interrupts();
    button.select_falling_edge_trigger();

    unsafe { enable_int() };

    // P2IFG.3 is set on the selected edge even with its interrupt disabled, so it can be polled
    loop {
        block!(button.wait_for_ifg()).ok();
        led2_blue.toggle().ok();
        pin.set_ifg();
    }
}

#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut led1) = *LED1.borrow_ref_mut(cs) else { return; };
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else { return; };

        if let GpioVector::Pin7Isr = p2iv.get_interrupt_vector() {
            led1.toggle().ok();
        }
    });
}

#[interrupt]
fn WDT() {
    with(|cs| {
        LED1.borrow_ref_mut(cs).as_mut().map(|led1| {
            led1.toggle().ok();
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
