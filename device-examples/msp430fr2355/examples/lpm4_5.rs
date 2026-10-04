#![no_main]
#![no_std]

// This examples enters LPM4.5, then when a button on P2.3 is pressed the system wakes and flashes the red LED.
// (Button S2 on P2.3 and LED1, red, on P1.0: SLAU680 Figure 18, p. 26. P1 to P4 have "LPM3.5, LPM4 and
// LPM4.5 wake-up input capability": SLASEC4D 6.10.3, p. 69.)

use embedded_hal::digital::*;
use msp430::asm::nop;
use msp430_rt::entry;
use msp430fr2355::{P3, P4, P5, P6};
use msp430_hal::{gpio::Batch, lpm::{enter_lpm4_5, SvsState}, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = Wdt::constrain(periph.wdt_a);
    // Pmm::new clears LOCKLPM5 here. After a wake-up from LPM4.5, SLAU445I 1.4.3.4, p. 42 initializes the
    // port registers "exactly the same way" as before LPM4.5 first and only then clears LOCKLPM5 (step 2),
    // which Pmm::new_locked allows; this example does it the other way round.
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // The HAL uses some of the SYS registers internally, but we need a copy as well. We promise not to modify any control bits used by the HAL.
    let sys = unsafe{ msp430fr2355::Sys::steal() };

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.2, p. 317: "To prevent a floating input and to reduce power consumption, unused I/O
    // pins should be configured as I/O function, output direction", or with the pullup or pulldown on.)
    let port1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = port1.pin0;

    // S2 connects P2.3 to GND and the board has no pull-up for it (SLAU680 Figure 18, p. 26)
    let port2 = Batch::new(periph.p2)
        .pulldown_all()
        .config_pin3(|p| p.pullup())
        .split(&pmm);

    init_unused_gpio(periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    // If this reset was a wake up from LPMx.5...
    // (SYSRSTIV can be used to decode the reset condition: SLAU445I 1.4.3.2, p. 42)
    if sys.sysrstiv().read().sysrstiv().is_lpm5wu() {
        loop {
            for _ in 0..10_000 {
                nop();
            }
            red_led.toggle().ok();
        }
    }
    // Otherwise it was a regular reset. Prepare to enter LPM4.5.
    else {
        // Configure P2.3 for interrupts
        // (S2 pulls P2.3 to GND, hence the pull-up and the falling edge: SLAU680 Figure 18, p. 26)
        let mut button = port2.pin3;
        button.select_falling_edge_trigger().enable_interrupts();

        // And enter LPM4.5. Global interrupts are enabled before LPM4.5 is entered.
        // ("TI also recommends setting GIE = 1 before entry into LPMx.5": SLAU445I 8.3.3, p. 318)
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
// (The exit from LPMx.5 is a BOR, SLAU445I 1.4.3.2, p. 42, and after a BOR the "Status register (SR) is
// reset", which clears GIE: SLAU445I 1.2.1, p. 32.)
// You *can* service the interrupt that causes the wakeup, but this isn't done here.

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
