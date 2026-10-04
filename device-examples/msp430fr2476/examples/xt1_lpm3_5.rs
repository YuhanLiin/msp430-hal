//! The RTC clocked by XT1 through LPM3.5: the board sleeps in LPM3.5, and the RTC, counting the generator's
//! signal on XIN, wakes it every second. Each wake-up toggles LED1, whose state is kept in the backup memory.
//!
//! A wake-up from LPMx.5 is a reset, so the program starts again from the top. After a wake-up,
//! `Pmm::new_locked` keeps the pins, and XT1 with them, in their LPM3.5 state until XIN and XT1 have been
//! configured again, so XT1 never stops clocking the RTC. After a cold start, `freeze()` waits for XT1
//! without a timeout.
//! (In LPM3.5 the RTC runs from XT1CLK or VLOCLK: SLAU445I 15.2.2, p. 417. "Any exit from LPMx.5 causes a
//! BOR", and the I/Os stay locked until LOCKLPM5 is cleared: SLAU445I 1.4.3.2, p. 42. XIN and XT1 are
//! configured before that: SLAU445I 1.4.3.3 steps 3 and 4, p. 42. The backup memory is retained in LPM3.5:
//! SLASEO7C 9.10.10, p. 61. LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test (function generator, and optionally the scope):
//! 1. Generator: square wave, 32.768 kHz, duty cycle 50 %, 0 V to 3.3 V (3.3 Vpp, 1.65 V offset), output load
//!    High-Z. Check the levels, and the frequency's unit (kHz, not Hz), on the scope before connecting: a
//!    negative or >3.6 V signal can damage the pin.
//! 2. Connect it to XIN, P2.1 (J2 pin 18), its ground to GND (J2 pin 20), and switch the output on.
//! 3. Flash this example. After flashing with mspdebug, the board must be unplugged and plugged back in for
//!    the example to work (see `lpm3_5.rs`): switch the generator output off, unplug the board's USB cable,
//!    wait a second, plug it back in, and switch the output on again. Keep the output off while the board is
//!    unplugged: a pin may see at most VCC + 0.3 V (SLASEO7C 8.1, p. 20), and unplugged, VCC is 0 V.
//! 4. Expected: LED1 toggles every second, a period of 2 s; the CPU sleeps in between. The scope on LED1,
//!    P1.0 (J3 pin 27), ground clip on GND (J3 pin 22), shows it exactly.
//! 5. Set the generator to 16.384 kHz: LED1 toggles every 2 s, so the RTC runs from XT1 during LPM3.5.
//! 6. Set it back to 32.768 kHz, and switch the output off: LED1 stops toggling, because the RTC's XT1CLK
//!    input has no fail-safe. Switch it back on: LED1 carries on.
//! (Header pins: SLAU802 Figure 10, p. 13.)
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430fr247x::{P3, P4, P5, P6};
use msp430_hal::{
    bak_mem::BackupMemory,
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Xt1Config},
    fram::Fram,
    gpio::Batch,
    lpm::{enter_lpm3_5, enter_lpm3_5_unchecked, SvsState},
    pmm::Pmm,
    rtc::{Rtc, RtcDiv},
    watchdog::Wdt,
};
use panic_msp430 as _;

/// Frequency the function generator is set to
const XT1_FREQ_HZ: u32 = 32_768;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366). A WDT in watchdog mode would keep
    // the device out of LPMx.5 (SLAU445I 1.4.3.1 step 7, p. 41).
    let wdt = Wdt::constrain(periph.wdt_a);
    let mut fram = Fram::new(periph.frctl);

    // After a wake-up from LPM3.5 the pins stay locked until XIN and XT1 are reconfigured, so
    // XT1 keeps clocking the RTC. After a cold start XT1 can only start once the pins are
    // unlocked, so unlock straight away.
    // (SLAU445I 1.4.3.3, p. 42. After a BOR the pins stay high-impedance until LOCKLPM5 is cleared:
    // SLAU445I 8.3.1, p. 316. A wake-up from LPMx.5 shows as SYSRSTIV = 08h: SLASEO7C Table 9-10,
    // p. 52.)
    let woke_from_lpm3_5 = periph.sys.sysrstiv().read().sysrstiv().is_lpmx5_wake_up();
    let (mut pmm, _) = if woke_from_lpm3_5 {
        Pmm::new_locked(periph.pmm, periph.sys)
    } else {
        Pmm::new(periph.pmm, periph.sys)
    };

    // Configure the pins the same way every time. Floating input pins consume a *huge* amount
    // of energy (relatively speaking), so pull unused pins down.
    // (SLAU445I 1.4.3.3 step 2, p. 42; floating inputs: SLAU445I 8.3.3, p. 317; pulldowns, PxDIR = 0,
    // PxREN = 1, PxOUT = 0: SLAU445I Table 8-1, p. 313)
    let port1 = Batch::new(periph.p1)
        .pulldown_all()
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut led1 = port1.pin0;
    let port2 = Batch::new(periph.p2)
        .pulldown_all()
        .config_pin1(|p| p.floating())
        .split(&pmm);
    // P2.1 = XIN with P2SEL = 01 (SLASEO7C Table 9-24, p. 66)
    let xin = port2.pin1.to_alternate1();
    init_unused_gpio(periph.p3, periph.p4, periph.p5, periph.p6, &pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range (SELMS = 000b) and ACLK left on REFO (SELA = 01b)
    // (SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118); XT1 in bypass mode
    // (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120)
    let (_smclk, _aclk, xt1clk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .xt1clk_on(Xt1Config::bypass(XT1_FREQ_HZ, xin))
        .freeze(&mut fram);

    // XIN and XT1 are configured, so the pins can be released (already done after a cold start)
    // (SLAU445I 1.4.3.3 step 4, p. 42)
    pmm.unlock_lpm5();

    if woke_from_lpm3_5 {
        // Toggle the LED.
        // I/O registers have their values reset coming out of LPMx.5,
        // so we have to store state in the backup memory.
        // (In LPMx.5 "The register content of all modules and the CPU is lost": SLAU445I 1.4.3, p. 40)
        let bak_mem = BackupMemory::as_u8s(periph.bakmem);

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
        // Configure RTC for 1 Hz interrupt from XT1
        // (RTCSS = 10b selects XT1CLK: SLASEO7C Table 9-18, p. 61. RTCSS, RTCPS = 000b dividing by 1,
        // and RTCIE enabling the interrupt: SLAU445I Table 15-2, p. 420.)
        let mut rtc = Rtc::new(periph.rtc).use_xt1clk(&xt1clk);
        rtc.set_clk_div(RtcDiv::_1);
        // A period lasts `count + 1` ticks, so this gives a 1 Hz period
        // (The counter resets to 0 after reaching the modulo value: SLAU445I 15.2.1, p. 417;
        // SLAU445I Figure 15-2, p. 418)
        rtc.start((XT1_FREQ_HZ - 1) as u16);
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
fn init_unused_gpio(p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    Batch::new(p3).pulldown_all().split(pmm);
    Batch::new(p4).pulldown_all().split(pmm);
    Batch::new(p5).pulldown_all().split(pmm);
    Batch::new(p6).pulldown_all().split(pmm);
}

// Note: In this case we don't need an ISR when waking from LPMx.5, since power on disables interrupts
// and this program never enables them; the RTC interrupt flag is cleared before LPM3.5 is entered again.
// (A BOR resets the SR, GIE included: SLAU445I 1.2.1, p. 32. The wake-up interrupt is serviced once
// interrupts are enabled: SLAU445I 1.4.3.3 step 7, p. 42.)

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
