//! The RTC counter as a timer: LED1 blinks, about 2 s on and 2 s off, timed by the RTC from the VLO.
//! Pressing S2 toggles LED1 and stops the RTC, which ends the blinking.
//!
//! The RTC counts VLOCLK divided by 10, about 1 kHz, from 0 to 2000 and then starts over: about 2 s at the
//! VLO's typical 10 kHz, but the VLO is only accurate to ±50 %, so anywhere from 1.3 s to 4 s. S2 pulls
//! P2.3 low, which sets P2IFG.3. The loop then toggles LED1 and stops the RTC (`pause()`, RTCSS = 00b),
//! and from then on it waits for an overflow that doesn't come, so only S2 toggles LED1.
//! (VLO: 10 kHz typical, SLASEC4D Table 5-8, p. 40; ±50 %, SLASEC4D Table 6-9, p. 68. RTCSS and RTCPS:
//! SLAU445I Table 15-2, p. 420. RTC predivider: SLAU445I 15.2.2, p. 417. LED1 on P1.0 is red, S2 is
//! P2.3, and S3 is the reset button: SLAU680 Figure 18, p. 26.)
//!
//! How to test:
//! 1. Flash this example. Expected: LED1 is on for about 2 s, off for about 2 s, and so on.
//! 2. Press S2: LED1 toggles and then stays as it is. Each further press of S2 toggles it.
//! 3. Press the reset button S3 to start the blinking again.
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

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let mut led = p1.pin0;
    let mut button = p2.pin3;

    let (_smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_refoclk(MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut Fram::new(periph.frctl));

    let mut rtc = Rtc::new(periph.rtc).use_vloclk();
    rtc.set_clk_div(RtcDiv::_10);

    button.select_falling_edge_trigger();
    led.set_high().ok();

    loop {
        // 2 seconds
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
