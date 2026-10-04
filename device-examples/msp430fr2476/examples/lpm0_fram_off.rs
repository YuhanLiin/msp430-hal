//! The supply current in LPM0, with the FRAM powered down (`lpm::enter_lpm0_fram_off_with_interrupts`) and
//! with it on (`lpm::enter_lpm0_with_interrupts`). The CPU sleeps in LPM0 while the DCO keeps running at
//! 8 MHz for SMCLK, and each press of button S2 switches between the two, then sleeps again. A multimeter
//! in series with the supply shows the difference.
//! (FRPWR = 0 disables the FRAM's supply, and "For LPM0, the FRAM power state during LPM0 is saved from the
//! previous state in active mode": SLAU445I 6.8, p. 303. LPM0 keeps SMCLK: SLAU445I Table 1-2, p. 39. LPM0
//! at 8 MHz, 25 °C and 3 V: 325 µA typical, with the FRAM on: SLASEO7C 8.6, p. 21; SLASEO7C Table 9-1,
//! p. 45. S2 is P2.3: SLAU802 Figure 19, p. 25.)
//!
//! After each press, LED1 (green) blinks if the FRAM now stays on, and the red part of LED2 if it's now
//! powered down.
//!
//! How to test (multimeter, following SLAU802 2.4, p. 11):
//! 1. Flash this example. Disconnect the function generator from the board.
//! 2. Remove the J9 jumper, which powers the TMP235 temperature sensor (SLAU802 2.2.5.1, p. 10).
//! 3. On J101, remove the TXD, RXD, SBW RST, SBW TST and 3V3 jumpers. Keep GND. The board is now off.
//! 4. Set the multimeter to DC current, with its leads in its current jacks (see its manual), and hold
//!    the leads on the two 3V3 pins of J101. The board starts, with the FRAM on in LPM0.
//! 5. Read the current: about 0.3 mA. Press S2: the red LED blinks, and the current drops, as the FRAM is
//!    now off while the CPU sleeps. Press S2 again: green blinks, and it rises again.
//!
//! The current flickers while the LED blinks, so read it a few seconds after. Put the jumpers back
//! afterwards: without them the board can't be flashed.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]
#![feature(asm_experimental_arch)]

use core::cell::RefCell;
use critical_section::with;
use embedded_hal::{delay::DelayNs, digital::*};
use msp430::interrupt::Mutex;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Batch, GpioVector, PxIV},
    lpm::{enter_lpm0_fram_off_with_interrupts, enter_lpm0_with_interrupts},
    pmm::Pmm,
    watchdog::Wdt,
};
use msp430fr247x::{interrupt, P2};
use panic_msp430 as _;

static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    // MCLK = SMCLK = DCOCLKDIV at 8 MHz and ACLK from REFO (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8,
    // p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118). SMCLK keeps the DCO running in LPM0.
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // "Make sure there are no floating inputs/outputs" (SLAU802 2.4, p. 11): S1 (P4.0) and S2 (P2.3) get
    // their pullup, as external resistors pull them up (R9, R10: SLAU802 Figure 19, p. 25) and a pulldown
    // would draw current through those, and `pulldown_unused` gives every other pin its pulldown (SLAU445I
    // 8.3.2, p. 317; pullup and pulldown: PxDIR = 0, PxREN = 1, PxOUT = 1 or 0: SLAU445I Table 8-1, p. 313)
    let p1 = Batch::new(periph.p1).pulldown_unused().split(&pmm);
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .pulldown_unused()
        .split(&pmm);
    let _p3 = Batch::new(periph.p3).pulldown_unused().split(&pmm);
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .pulldown_unused()
        .split(&pmm);
    let p5 = Batch::new(periph.p5).pulldown_unused().split(&pmm);
    let _p6 = Batch::new(periph.p6).pulldown_unused().split(&pmm);

    // LED1 is P1.0 and the red part of LED2 is P5.1 (SLAU802 Figure 19, p. 25)
    let mut led1 = p1.pin0.to_output_low();
    let mut led2_red = p5.pin1.to_output_low();
    let _s1 = p4.pin0;
    let mut s2 = p2.pin3;
    with(|cs| P2IV.borrow_ref_mut(cs).replace(p2.pxiv));

    // S2 pulls P2.3 low, so a press is a high-to-low transition (PxIES = 1: SLAU445I Table 8-16, p. 336;
    // PxIE: SLAU445I Table 8-17, p. 336)
    s2.select_falling_edge_trigger().enable_interrupts();

    let mut fram_off = false;
    loop {
        // The PORT2 handler wakes the CPU, and the loop continues. It runs from FRAM, which powers the FRAM
        // up again (SLAU445I 6.8, p. 303).
        if fram_off {
            enter_lpm0_fram_off_with_interrupts();
        } else {
            enter_lpm0_with_interrupts();
        }

        fram_off = !fram_off;
        let led: &mut dyn OutputPin<Error = _> = if fram_off { &mut led2_red } else { &mut led1 };
        led.set_high().ok();
        delay.delay_ms(100);
        led.set_low().ok();

        // Wait for the button's release and its bouncing to end, then forget the edges it caused
        while s2.is_low().unwrap() {}
        delay.delay_ms(100);
        s2.clear_ifg();
    }
}

// The port 2 interrupt vector, P2IFG.0 to P2IFG.7 through P2IV (FFD4h: SLASEO7C Table 9-2, p. 47).
// Reading P2IV "automatically resets the highest pending interrupt flag" (SLAU445I 8.2.6, p. 315).
// `wake_cpu` returns the CPU to active mode afterwards (an interrupt returns to another operating mode
// if its handler changes the SR saved on the stack: SLAU445I 1.4, p. 36).
#[interrupt(wake_cpu)]
fn PORT2() {
    with(|cs| {
        if let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) {
            let _: GpioVector = p2iv.get_interrupt_vector();
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
