#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use critical_section::with;
use msp430fr2355::interrupt;

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

// Red LED should blink about 3.3 seconds on, 3.3 seconds off: the WDT interval is 2^15 cycles of ACLK
// (WDTIS = 100b, SLAU445I Table 12-2, p. 366), which is VLOCLK, 10 kHz typical (SLASEC4D Table 5-8, p. 40)
// Both green and red LEDs should blink when P2.3 button (S2) is pressed
// (LED1, red, on P1.0; LED2, green, on P6.6; S2 on P2.3: SLAU680 Figure 18, p. 26)
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let mut wdt = Wdt::constrain(periph.wdt_a).to_interval();

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1) // 32 kHz MCLK (REFO, 32768 Hz: SLASEC4D Table 5-7, p. 40)
        .smclk_on(SmclkDiv::_2) // 16 kHz SMCLK
        .aclk_vloclk()
        .freeze(&mut Fram::new(periph.frctl));

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p6 = Batch::new(periph.p6)
        .config_pin6(|p| p.to_output())
        .split(&pmm);

    let red_led = p1.pin0.to_output();
    // Onboard button with interrupt disabled (S2, which connects P2.3 to GND and has no pull-up on the
    // board: SLAU680 Figure 18, p. 26)
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually. (On the LaunchPad P2.7 is XIN,
    // wired to the 32.768-kHz crystal Q1: SLAU680 2.5, p. 13; SLAU680 Figure 18, p. 26)
    let mut pin = p2.pin7.pulldown();
    let mut green_led = p6.pin6;
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
        green_led.toggle().ok();
        pin.set_ifg();
    }
}

// The port P2 vector at FFD2h, decoded with P2IV (SLASEC4D Table 6-2, p. 64)
#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut red_led) = *RED_LED.borrow_ref_mut(cs) else { return; };
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else { return; };

        if let GpioVector::Pin7Isr = p2iv.get_interrupt_vector() {
            red_led.toggle().ok();
        }
    });
}

// The watchdog timer interval mode vector at FFE6h, WDTIFG (SLASEC4D Table 6-2, p. 63)
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
