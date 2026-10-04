//! LPM3.5 and an RTC wake-up: the board sleeps in LPM3.5, and the RTC wakes it about once a second. Each
//! wake-up toggles LED1, whose state is kept in the backup memory.
//!
//! The RTC counts VLOCLK, which keeps running in LPM3.5. A wake-up from LPMx.5 is a reset, so the program
//! starts again from the top: if SYSRSTIV says it woke from LPMx.5, it toggles LED1 and goes back to sleep,
//! and otherwise it sets up the RTC first. The VLO's frequency isn't exact, so neither is the second.
//! (The RTC can wake the device from LPM3.5, and "Any exit from LPMx.5 causes a BOR": SLAU445I 1.4.3.2, p. 41
//! to p. 42. In LPM3.5 the RTC can only count XT1CLK or VLOCLK: SLAU445I 15.2.2, p. 417. The VLO runs at
//! 10 kHz ±50 %: SLASEC4D Table 6-9, p. 68. The backup memory keeps its 32 bytes during LPM3.5:
//! SLASEC4D 6.10.10, p. 76. LED1 on P1.0 is red: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example. After flashing with mspdebug, unplug the board's USB cable, wait a second, and plug
//!    it back in: the example only works after that. (Uniflash and Code Composer Studio need no replug.)
//! 2. Expected: LED1 toggles about once a second: on for about a second, off for about a second.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430fr2355::{P2, P3, P4, P5, P6};
use msp430_hal::{
    bak_mem::BackupMemory,
    clock::VLOCLK_FREQ_HZ,
    gpio::Batch,
    lpm::{enter_lpm3_5, enter_lpm3_5_unchecked, SvsState},
    pmm::Pmm,
    rtc::{Rtc, RtcDiv},
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = Wdt::constrain(periph.wdt_a);
    // Pmm::new clears LOCKLPM5 here. After a wake-up from LPM3.5, SLAU445I 1.4.3.3, p. 42 initializes the
    // RTC registers and the port registers "exactly the same way" as before LPM3.5 first and only then
    // clears LOCKLPM5 (step 4), which Pmm::new_locked allows; this example does it the other way round.
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // The HAL uses some of the SYS registers internally, but we need a copy as well. We promise not to modify any control bits used by the HAL.
    let sys = unsafe{ msp430fr2355::Sys::steal() };

    // Floating input pins consume a *huge* amount of energy (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.2, p. 317: "To prevent a floating input and to reduce power consumption, unused I/O
    // pins should be configured as I/O function, output direction", or with the pullup or pulldown on.)
    let port1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = port1.pin0;

    init_unused_gpio(periph.p2, periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    // If this reset was a wake up from LPMx.5...
    // (SYSRSTIV can be used to decode the reset condition: SLAU445I 1.4.3.2, p. 42)
    if sys.sysrstiv().read().sysrstiv().is_lpmx5_wake_up() {
        // Toggle the LED.
        // I/O registers have their values reset coming out of LPMx.5,
        // so we have to store state in the backup memory.
        // (In LPMx.5 "The register content of all modules and the CPU is lost": SLAU445I 1.4.3, p. 40.)
        let bak_mem = BackupMemory::as_u8s(periph.bakmem);

        let old_value = bak_mem[0] == 1;
        red_led.set_state(old_value.into()).ok();

        let new_value = if old_value { 0 } else { 1 };
        bak_mem[0] = new_value;

        // Clear RTC interrupt flag
        // ("Reading RTCIV register clears the interrupt flag": SLAU445I 15.2.4, p. 418)
        periph.rtc.rtciv().read();

        // Enter LPM3.5 (without having to configure the RTC, we did that already).
        // (SLAU445I 1.4.3.3, p. 42, step 1, re-initializes "the registers of the modules connected to the
        // RTC LDO" after each wake-up from LPM3.5; this example relies on the RTC settings from the first
        // run instead.)
        unsafe { enter_lpm3_5_unchecked(wdt, SvsState::Disabled) };
    }
    // Otherwise this is a fresh start. Configure the RTC.
    else {
        // Configure RTC for 1 Hz interrupt
        // (VLOCLK is 10 kHz typical: SLASEC4D Table 5-8, p. 40. It can stay on in LPM3.5: SLASEC4D
        // Table 6-1, p. 61.)
        let mut rtc = Rtc::new(periph.rtc).use_vloclk();
        rtc.set_clk_div(RtcDiv::_1);
        rtc.start(VLOCLK_FREQ_HZ); // Count up to VLOCLK freq -> 1 Hz period
        rtc.enable_interrupts();
        // Interrupts were never enabled, so `enter_lpm3_5()` enters LPM3.5 with GIE clear, as
        // SLAU445I 1.4.3.1 step 8, p. 41 does. The RTC event still wakes the device (SLAU445I 1.4.3.2,
        // p. 41).
        // Leaving LPMx.5 requires a full system reset, so this function will never return.
        // ("Any exit from LPMx.5 causes a BOR": SLAU445I 1.4.3.2, p. 42)
        enter_lpm3_5(wdt, rtc, SvsState::Disabled);
    }
}

/// Enable pulldowns on unused ports to massively reduce power usage (SLAU445I 8.3.2, p. 317).
fn init_unused_gpio(p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    Batch::new(p2).pulldown_all().split(pmm);
    Batch::new(p3).pulldown_all().split(pmm);
    Batch::new(p4).pulldown_all().split(pmm);
    Batch::new(p5).pulldown_all().split(pmm);
    Batch::new(p6).pulldown_all().split(pmm);
}

// Note: In this case we don't need an ISR when waking from LPMx.5, since power on disables interrupts
// and this program never enables them; the RTC interrupt flag is cleared before LPM3.5 is entered again.
// You *can* service the interrupt that causes the wakeup, but this isn't done here.
// (A BOR resets the SR, GIE included: SLAU445I 1.2.1, p. 32. The wake-up interrupt is serviced once
// interrupts are enabled: SLAU445I 1.4.3.3 step 7, p. 42.)

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
