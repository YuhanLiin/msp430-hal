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
// (The RTC counts VLOCLK, 10 kHz typical (SLASEC4D Table 5-8, p. 40), divided by 10, up to 2000.)
// Pressing P2.3 button toggles red LED
// (LED1, red, on P1.0; button S2 on P2.3, which connects the pin to GND: SLAU680 Figure 18, p. 26)
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
