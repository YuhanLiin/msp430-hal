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

// Red LED blinks 2 seconds on, 2 off
// Pressing P2.3 button toggles red LED
// No board document covers the LED (on P1.0 here) or the button: there is none for the MSP430FR25x2.
// P2.3 only exists on the 20-pin RHL package (SLASEE4C Table 4-2, p. 14). Both pins are GPIO, PxSELx = 00
// (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60): P1.0 an output, P2.3 an input with its
// pullup (SLAU445I Table 8-1, p. 313).
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
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
        .mclk_refoclk(MclkDiv::_1) // MCLK from REFO, 32768 Hz (SLASEE4C Table 5-7, p. 27)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut Fram::new(periph.frctl));

    // VLO / 10: 1 kHz typical, so the RTC counts milliseconds. RTCSS = 11 is VLOCLK
    // (SLASEE4C Table 6-12, p. 55), RTCPS = 001b divides by 10 (SLAU445I 15.3.1, p. 420). The VLO is
    // 10 kHz typical (SLASEE4C Table 5-8, p. 28), within ±50 % (SLASEE4C Table 6-8, p. 49).
    let mut rtc = Rtc::new(periph.rtc).use_vloclk();
    rtc.set_clk_div(RtcDiv::_10);

    // P2IES = 1: P2IFG is set on a falling edge (SLAU445I 8.2.6.2, p. 316); the loop polls that flag
    button.select_falling_edge_trigger();
    led.set_high().ok();

    loop {
        // 2 seconds (2000 periods of the 1 kHz typical clock above)
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
