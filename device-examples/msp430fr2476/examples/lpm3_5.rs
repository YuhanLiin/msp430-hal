#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430fr247x::{P2, P3, P4, P5, P6};
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

// The RTC will wake the board every second. LED state is stored in and loaded from the backup memory.
// When programming with mspdebug you need to unplug and replug the board for the example to work, for some reason.
// Programming via Uniflash or Code Composer Studio works fine.
// (The RTC can wake the device from LPM3.5: SLAU445I 1.4.3.2, p. 41. The backup memory is retained in
// LPM3.5: SLASEO7C 9.10.10, p. 61. The LED is LED1 on P1.0, which is green: SLAU802 Figure 19, p. 25.)
#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366). A WDT in watchdog mode would keep
    // the device out of LPMx.5 (SLAU445I 1.4.3.1 step 7, p. 41).
    let wdt = Wdt::constrain(periph.wdt_a);
    // Pmm::new clears LOCKLPM5 here, before the pins are configured again. After a wake-up from LPM3.5,
    // SLAU445I 1.4.3.3 steps 1 to 4, p. 42 configures the pins and the RTC first and clears LOCKLPM5
    // after that (as xt1_lpm3_5.rs does with Pmm::new_locked); this example keeps the simpler order.
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);

    // The HAL uses some of the SYS registers internally, but we need a copy as well. We promise not to modify any control bits used by the HAL.
    let sys = unsafe{ msp430fr247x::Sys::steal() };

    // Floating input pins consume a *huge* amount of energy (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    // (SLAU445I 8.3.3, p. 317: "It is critical that no inputs are left floating", or LPMx.5 draws more.
    // Pulldowns, PxDIR = 0, PxREN = 1, PxOUT = 0: SLAU445I Table 8-1, p. 313.)
    let port1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut led1 = port1.pin0;

    init_unused_gpio(periph.p2, periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    // If this reset was a wake up from LPMx.5...
    // (SYSRSTIV = 08h, "LPMx.5 wakeup (BOR)": SLASEO7C Table 9-10, p. 52)
    if sys.sysrstiv().read().sysrstiv().is_lpm5wu() {
        // Toggle the LED.
        // I/O registers have their values reset coming out of LPMx.5,
        // so we have to store state in the backup memory.
        // (In LPMx.5 "The register content of all modules and the CPU is lost": SLAU445I 1.4.3, p. 40)
        let bak_mem = BackupMemory::as_u8s(periph.bkmem);

        let old_value = bak_mem[0] == 1;
        led1.set_state(old_value.into()).ok();

        let new_value = if old_value { 0 } else { 1 };
        bak_mem[0] = new_value;

        // Clear RTC interrupt flag
        // ("Reading RTCIV register clears the interrupt flag": SLAU445I 15.2.4, p. 418)
        periph.rtc.rtciv().read();

        // Enter LPM3.5 (without having to configure the RTC, we did that already).
        // (This relies on the RTC keeping its configuration through LPM3.5. SLAU445I 1.4.3.3 step 1,
        // p. 42 initializes the RTC registers again after a wake-up, before LOCKLPM5 is cleared.)
        // SVSHE = 0 turns the high-side SVS off in LPM3.5 (SLAU445I Table 2-2, p. 91; SLAU445I 1.4.3.1
        // step 9c, p. 41).
        unsafe { enter_lpm3_5_unchecked(wdt, SvsState::Disabled) };
    }
    // Otherwise this is a fresh start. Configure the RTC.
    else {
        // Configure RTC for 1 Hz interrupt
        // (VLOCLK_FREQ_HZ is the VLO's typical 10 kHz: SLASEO7C 8.12.3.5, p. 30; "10 kHz ±50%":
        // SLASEO7C Table 9-8, p. 50.)
        // (RTCSS = 11b selects VLOCLK, RTCPS = 000b divides by 1 and RTCIE enables the interrupt:
        // SLAU445I Table 15-2, p. 420. In LPM3.5 the RTC can run from XT1CLK or VLOCLK only: SLAU445I
        // 15.2.2, p. 417. A period lasts the modulo value + 1 ticks: SLAU445I 15.2.1, p. 417; SLAU445I
        // Figure 15-2, p. 418.)
        let mut rtc = Rtc::new(periph.rtc).use_vloclk();
        rtc.set_clk_div(RtcDiv::_1);
        rtc.start(VLOCLK_FREQ_HZ); // Count up to VLOCLK freq -> 1 Hz period
        rtc.enable_interrupts();
        // Interrupts were never enabled, so `enter_lpm3_5()` enters LPM3.5 with GIE clear, as
        // SLAU445I 1.4.3.1 step 8, p. 41 does. The RTC event still wakes the device (SLAU445I 1.4.3.2,
        // p. 41).
        // Leaving LPMx.5 requires a full system reset, so this function will never return.
        // ("Any exit from LPMx.5 causes a BOR": SLAU445I 1.4.3.2, p. 42)
        // SVSHE = 0 turns the high-side SVS off in LPM3.5 (SLAU445I Table 2-2, p. 91)
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
