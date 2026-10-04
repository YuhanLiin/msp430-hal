//! LPM0 and a button interrupt: the CPU sleeps in LPM0, and each press of S2 wakes it to toggle LED1.
//!
//! S2 pulls P2.3 low, and the falling edge requests the port 2 interrupt. Its handler is declared with
//! `wake_cpu`, so the CPU stays awake when it returns: the main loop toggles LED1, waits a moment to debounce
//! the button, and enters LPM0 again.
//! (LPM0 turns off the CPU and MCLK: SLAU445I Table 1-2, p. 39. The SR is saved on the stack during an
//! interrupt, and a handler that changes it there returns to a different operating mode: SLAU445I 1.4.2,
//! p. 40. LED1 on P1.0 is green, and S2 is on P2.3: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example.
//! 2. Expected: LED1 is off, and each press of S2 toggles it. Between presses it holds still: the CPU sleeps.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]
#![feature(asm_experimental_arch)]

// NOTE: This example relies on the new wake-cpu feature recently added to the msp430-rt crate to return the CPU to active mode
// after the interrupt returns. This depends on Rust 1.88+. For a version compatible with the MSRV of this crate see lpm0_msrv.rs

use critical_section::with;
use msp430fr247x::{interrupt, P2, P3, P4, P5, P6};

use core::cell::RefCell;
use embedded_hal::digital::*;
use msp430::{asm, interrupt::{enable as enable_interrupts, Mutex}};
use msp430_rt::entry;
use msp430_hal::{
    gpio::{Batch, GpioVector, PxIV}, lpm::enter_lpm0, pmm::Pmm, watchdog::Wdt
};
use panic_msp430 as _;

static P2IV: Mutex<RefCell<Option< PxIV<P2> >>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.2, p. 317; pullup and pulldown settings: SLAU445I Table 8-1, p. 313)
    let p1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut green_led = p1.pin0;

    let p2 = Batch::new(periph.p2)
        .pulldown_all()
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let mut button = p2.pin3;
    let p2iv = p2.pxiv;

    init_unused_gpio(periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    with(|cs| {
        P2IV.borrow_ref_mut(cs).replace(p2iv);
    });

    // S2 pulls P2.3 low, so a press is a high-to-low transition (PxIES = 1: SLAU445I Table 8-16, p. 336;
    // PxIE: SLAU445I Table 8-17, p. 336)
    button.select_falling_edge_trigger().enable_interrupts();

    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { enable_interrupts() };

    loop {
        // Since no peripherals were configured to use SMCLK / ACLK we could just as well enter LPM3 / LPM4 here
        // (port interrupts wake the device from LPM4 too: SLASEO7C Table 9-1, p. 45)
        // LPM0 sets CPUOFF: "CPU, MCLK are disabled" (SLAU445I Table 1-2, p. 39)
        enter_lpm0();
        green_led.toggle().ok();

        for _ in 0..15_000 { // Debouncing
            asm::nop();
        }
    }
}

// Interrupt handlers with the `wake_cpu` argument will set the MSP430 back to Active Mode after the interrupt completes.
// (An interrupt returns to another operating mode if its handler changes the SR saved on the stack:
// SLAU445I 1.4, p. 36)
// The port 2 interrupt vector, P2IFG.0 to P2IFG.7 through P2IV (FFD4h: SLASEO7C Table 9-2, p. 47).
// Reading P2IV "automatically resets the highest pending interrupt flag" (SLAU445I 8.2.6, p. 315).
#[interrupt(wake_cpu)]
fn PORT2() {
    with(|cs| {
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {return};
        if let GpioVector::Pin3Isr = p2iv.get_interrupt_vector() {
            // Button pressed
        }
    });
}

/// Enable pulldowns on unused ports to massively reduce power usage (SLAU445I 8.3.2, p. 317).
fn init_unused_gpio(p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    Batch::new(p3).pulldown_all().split(pmm);
    Batch::new(p4).pulldown_all().split(pmm);
    Batch::new(p5).pulldown_all().split(pmm);
    Batch::new(p6).pulldown_all().split(pmm);
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
