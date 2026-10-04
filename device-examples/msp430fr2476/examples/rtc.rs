#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    rtc::{Rtc, RtcDiv},
    watchdog::Wdt,
};
use panic_msp430 as _;

// LED1 blinks 2 seconds on, 2 off
// Pressing P2.3 button toggles LED1
// (LED1 on P1.0 is green, the P2.3 button is S2: SLAU802 Figure 19, p. 25. 2000 ticks of the VLO
// divided by 10 take 2 s at the VLO's typical 10 kHz: SLASEO7C 8.12.3.5, p. 30; RTC predivider:
// SLAU445I 15.2.2, p. 417.)
#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    // S2 on P2.3 with the internal pullup (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I Table 8-1, p. 313)
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let mut led = p1.pin0;
    let mut button = p2.pin3;

    // MCLK = SMCLK from REFOCLK (SELMS = 001b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I
    // Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I Table 3-1, p. 98 lists
    // that for the enhanced clock system only, and the HAL follows the data sheet.
    let (_smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut Fram::new(periph.frctl));

    // RTCSS = 11b selects VLOCLK and RTCPS = 001b divides by 10 (SLAU445I Table 15-2, p. 420)
    let mut rtc = Rtc::new(periph.rtc).use_vloclk();
    rtc.set_clk_div(RtcDiv::_10);

    // PxIES = 1: P2IFG.3 is set on a high-to-low transition (SLAU445I Table 8-16, p. 336)
    button.select_falling_edge_trigger();
    led.set_high().ok();

    loop {
        // 2 seconds
        // (The RTC sets RTCIFG when it reaches RTCMOD and starts over from 0: SLAU445I 15.2.1, p. 417.
        // pause() stops it with RTCSS = 00b, "No clock (Stop)": SLAU445I Table 15-2, p. 420.)
        rtc.start(2000);
        while let Err(nb::Error::WouldBlock) = rtc.wait() {
            if button.wait_for_ifg().is_ok() {
                led.toggle().ok();
                rtc.pause();
            }
        }
        led.toggle().ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
