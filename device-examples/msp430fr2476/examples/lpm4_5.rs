//! LPM4.5 and a button wake-up: the board sleeps in LPM4.5 until S2 is pressed, and then LED1 flashes.
//!
//! LPM4.5 stops every clock, so only an edge on a wake-up pin, the RST pin or a power cycle ends it. The
//! wake-up is a reset: the program starts again from the top, sees in SYSRSTIV that it woke from LPMx.5, and
//! flashes LED1 instead of going back to sleep.
//! (What ends LPMx.5, and "Any exit from LPMx.5 causes a BOR": SLAU445I 1.4.3.2, p. 41 to p. 42. P2.3 has
//! "wake from LPMx.5": SLASEO7C Table 7-2, p. 16. S2 is on P2.3, and LED1 on P1.0 is green:
//! SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example. After flashing with mspdebug, unplug the board's USB cable, wait a second, and plug
//!    it back in: the example only works after that.
//! 2. Expected: LED1 stays off while the board sleeps.
//! 3. Press S2: LED1 flashes, and keeps flashing.
//! 4. Press S3 (reset): LED1 goes off, and the board sleeps until the next press of S2.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430::asm::nop;
use msp430_rt::entry;
use msp430fr247x::{P3, P4, P5, P6};
use msp430_hal::{gpio::Batch, lpm::{enter_lpm4_5, SvsState}, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366). A WDT in watchdog mode would keep
    // the device out of LPMx.5 (SLAU445I 1.4.3.1 step 7, p. 41).
    let wdt = Wdt::constrain(periph.wdt_a);
    // Pmm::new clears LOCKLPM5 here, before the pins are configured again. After a wake-up from LPM4.5,
    // SLAU445I 1.4.3.4 steps 1 and 2, p. 42 configures the port registers first and clears LOCKLPM5
    // after that; this example keeps the simpler order.
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // The HAL uses some of the SYS registers internally, but we need a copy as well. We promise not to modify any control bits used by the HAL.
    let sys = unsafe{ msp430fr247x::Sys::steal() };

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.3, p. 317: "It is critical that no inputs are left floating", or LPMx.5 draws more.
    // Pullup and pulldown settings: SLAU445I Table 8-1, p. 313.)
    let port1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut led1 = port1.pin0;

    let port2 = Batch::new(periph.p2)
        .pulldown_all()
        .config_pin3(|p| p.pullup())
        .split(&pmm);

    init_unused_gpio(periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    // If this reset was a wake up from LPMx.5...
    // (SYSRSTIV = 08h, "LPMx.5 wakeup (BOR)": SLASEO7C Table 9-10, p. 52)
    if sys.sysrstiv().read().sysrstiv().is_lpmx5_wake_up() {
        loop {
            for _ in 0..10_000 {
                nop();
            }
            led1.toggle().ok();
        }
    }
    // Otherwise it was a regular reset. Prepare to enter LPM4.5.
    else {
        // Configure P2.3 for interrupts
        // (S2 pulls P2.3 low, so a press is a falling edge: SLAU802 Figure 19, p. 25. Wake-up edge and
        // enable: SLAU445I 1.4.3.1 step 4, p. 41; PxIES = 1 for a high-to-low transition: SLAU445I
        // Table 8-16, p. 336)
        let mut button = port2.pin3;
        button.select_falling_edge_trigger().enable_interrupts();

        // And enter LPM4.5. Interrupts were never enabled, so GIE stays clear, as in
        // SLAU445I 1.4.3.1 step 8, p. 41; the P2.3 edge wakes the device anyway (SLAU445I 1.4.3.2, p. 41).
        // The RTC is stopped (RTCSS = 00b: SLAU445I Table 15-2, p. 420), so the device enters LPM4.5
        // rather than LPM3.5 (SLAU445I 1.4.3.1, p. 41), and SVSHE = 0 turns the high-side SVS off in
        // LPM4.5 (SLAU445I Table 2-2, p. 91).
        enter_lpm4_5(wdt, periph.rtc, SvsState::Disabled);
    }
}

/// Enable pulldowns on unused ports to massively reduce power usage (SLAU445I 8.3.2, p. 317).
fn init_unused_gpio(p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    Batch::new(p3).pulldown_all().split(pmm);
    Batch::new(p4).pulldown_all().split(pmm);
    Batch::new(p5).pulldown_all().split(pmm);
    Batch::new(p6).pulldown_all().split(pmm);
}

// Note: In this case we don't need an ISR when waking from LPMx.5, since power on disables interrupts.
// You *can* service the interrupt that causes the wakeup, but this isn't done here.
// (Any exit from LPMx.5 is a BOR: SLAU445I 1.4.3.2, p. 42, and a BOR resets the SR, GIE included:
// SLAU445I 1.2.1, p. 32.)

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
