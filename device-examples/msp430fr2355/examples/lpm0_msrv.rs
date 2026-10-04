#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// NOTE: Historically there was no way to return the CPU to active mode after entering a low power mode,
// the MSP restores the CPU to active mode during an interrupt but turns it off afterwards.
// (SLASEC4D 6.2, p. 61: an interrupt event can wake the device from LPM0, LPM3 or LPM4, "service the
// request, and return to the low-power mode on return from the interrupt program")

// A feature was recently added to msp430-rt to allow the CPU to return to active mode, but it depends on Rust 1.88.
// For compatibility with the MSRV of this crate we showcase the old implementation here, with all work being done inside the interrupt.
// For the more flexible new version, see lpm0.rs

use critical_section::with;
use msp430fr2355::{interrupt, P1, P2, P3, P4, P5, P6};

use core::cell::RefCell;
use embedded_hal::digital::*;
use msp430::{asm, interrupt::{enable as enable_interrupts, Mutex}};
use msp430_rt::entry;
use msp430_hal::{
    gpio::{Batch, GpioVector, Output, Pin, Pin0, PxIV}, lpm::enter_lpm0, pmm::Pmm, watchdog::Wdt
};
use panic_msp430 as _;

static P2IV: Mutex<RefCell<Option< PxIV<P2> >>> = Mutex::new(RefCell::new(None));
static RED_LED: Mutex<RefCell<Option< Pin<P1, Pin0, Output> >>> = Mutex::new(RefCell::new(None));

// P1.0 should toggle when P2.3 is pressed
// (LED1, red, on P1.0; button S2 on P2.3, which connects the pin to GND: SLAU680 Figure 18, p. 26)
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let _wdt = Wdt::constrain(periph.wdt_a);
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.2, p. 317: "To prevent a floating input and to reduce power consumption, unused I/O
    // pins should be configured as I/O function, output direction", or with the pullup or pulldown on.)
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

    init_unused_gpio(periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    with(|cs| {
        P2IV.borrow_ref_mut(cs).replace(p2iv);
        RED_LED.borrow_ref_mut(cs).replace(red_led);
    });

    button.select_falling_edge_trigger().enable_interrupts();

    unsafe { enable_interrupts() };

    // Since no peripherals were configured to use SMCLK / ACLK we could just as well enter LPM3 / LPM4 here
    // (LPM3 keeps only ACLK, LPM4 no clock, and an I/O interrupt wakes both: SLASEC4D Table 6-1, p. 61.
    // Errata on entering LPM3 or LPM4: SLAZ695J CS13 and PMM32.)
    enter_lpm0();

    loop {}
}

// The CPU will wake up to handle interrupts, but will be put back to sleep afterwards.
// (RETI pops the original SR, "restoring the previous operating mode": SLAU445I 1.4.2, p. 40. This is
// the port P2 vector at FFD2h: SLASEC4D Table 6-2, p. 64.)
#[interrupt]
fn PORT2() {
    with(|cs| {
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {return};
        let Some(ref mut red_led) = *RED_LED.borrow_ref_mut(cs) else {return};
        if let GpioVector::Pin3Isr = p2iv.get_interrupt_vector() {
            red_led.toggle().ok();
            for _ in 0..15_000 { // Debouncing
                asm::nop();
            }
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
