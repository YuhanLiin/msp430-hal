//! LPM0 and a button interrupt, with the work done in the interrupt: the CPU sleeps in LPM0, and each press
//! of a button on P2.3 toggles an LED on P1.0 in the port 2 interrupt, after which the CPU goes back to
//! sleep.
//!
//! The button pulls P2.3 low against its internal pullup, and the falling edge requests the port 2 interrupt.
//! Its handler toggles the LED and debounces the button. When it returns, the CPU is back in LPM0, so the
//! main program never gets past `enter_lpm0()`.
//! (LPM0 turns off the CPU and MCLK: SLAU445I Table 1-2, p. 39. On return from an interrupt "The original SR
//! is popped from the stack, restoring the previous operating mode": SLAU445I 1.4.2, p. 40. No board document
//! covers the LED or the button: there is none for the MSP430FR25x2. P1.0 and P2.3 are GPIO with PxSELx = 00:
//! SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60. The pullup: SLAU445I Table 8-1, p. 313.)
//!
//! How to test (an LED, a resistor and a push button):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND, and a push button from P2.3 to
//!    GND. P2.3 only exists on the 20-pin RHL package (SLASEE4C Table 4-2, p. 14).
//! 2. Flash this example.
//! 3. Expected: the LED is off, and each press of the button toggles it.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// NOTE: Historically there was no way to return the CPU to active mode after entering a low power mode,
// the MSP restores the CPU to active mode during an interrupt but turns it off afterwards.
// (SLAU445I 1.4.2, p. 40: entering the interrupt service routine resets CPUOFF, SCG1 and OSCOFF, and on
// return "The original SR is popped from the stack, restoring the previous operating mode")

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

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363). Pmm::new clears LOCKLPM5,
    // so the pins take on their configuration (SLAU445I 8.3.1, p. 316).
    let _wdt = Wdt::constrain(periph.wdt_a);
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.2, p. 317: "To prevent a floating input and to reduce power consumption";
    // SLASEE4C Table 4-4, p. 16)
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

    // P2IES = 1: P2IFG on a falling edge (SLAU445I 8.2.6.2, p. 316). The flag requests the interrupt with
    // P2IE and GIE set (SLAU445I 8.2.6, p. 315).
    button.select_falling_edge_trigger().enable_interrupts();

    unsafe { enable_interrupts() };

    // Since no peripherals were configured to use SMCLK / ACLK we could just as well enter LPM3 / LPM4 here
    // (SLASEE4C Table 6-1, p. 45: LPM3 stops SMCLK, LPM4 also ACLK, and I/O interrupts wake both).
    // LPM3 and LPM4 entry has errata: SLAZ705H CS13, p. 7 to p. 8, and PMM32, p. 8 to p. 10.
    enter_lpm0();

    loop {}
}

// The CPU will wake up to handle interrupts, but will be put back to sleep afterwards.
// (SLAU445I 1.4.2, p. 40)
// Port 2 interrupt, P2IV (SLASEE4C Table 6-2, p. 46: vector FFE4h)
#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {
            return;
        };
        let Some(ref mut red_led) = *RED_LED.borrow_ref_mut(cs) else {
            return;
        };
        // Reading P2IV clears the highest-priority pending flag (SLAU445I 8.2.6, p. 315)
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
