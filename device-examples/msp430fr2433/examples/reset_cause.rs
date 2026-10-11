//! Why did the device reset? `Pmm::take_reset_cause` reads the reasons, highest priority first, and the 
//! red and green LEDs shows the first one:
//!
//! | LED                    | Reset                                                         |
//! |------------------------|---------------------------------------------------------------|
//! | solid red              | power-up: plug in the USB cable (brownout reset)              |
//! | solid green            | the reset button S3                                           |
//! | blinking red           | button S1, which calls `Pmm::software_bor()`                  |
//! | blinking green         | button S2, which calls `Pmm::software_por()`                  |
//! | blinking red and green | any other reason                                              |
//! | solid red and green    | no reason: the debugger started the program after flashing it |
//!
//! The buttons reset the device when they are released.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, 
    fram::Fram, 
    gpio::{Batch, Input, Output, Pin, Pin0, Pin1, Pin3, Pin7, Pullup}, 
    pmm::{Pmm, ResetCause}, 
    watchdog::Wdt,
};
use msp430fr2433::{P1, P2};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();
    // Stop the watchdog
    Wdt::constrain(periph.watchdog_timer);

    // The pins stay locked until the ports are configured below, as the data sheet asks
    let (mut pmm, _) = Pmm::new_locked(periph.pmm, periph.sys);

    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .config_pin1(|p| p.to_output())
        .split(&pmm);
    // S1 and S2 are inputs with their internal pullups
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .config_pin7(|p| p.pullup())
        .split(&pmm);
    let mut sw1 = p2.pin3;
    let mut sw2 = p2.pin7;
    let mut red = p1.pin0;
    let mut green = p1.pin1;

    // The ports are configured: release them
    pmm.unlock_lpm5();

    // Configure clocks for delay timing
    let mut fram = Fram::new(periph.fram);
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // Store the first reset reason, then discard the rest, clearing them for the next reset
    let first = pmm.take_reset_cause();
    while pmm.take_reset_cause().is_some() {}

    let colour = match first {
        Some(ResetCause::Brownout)    => LedColours::SolidRed,
        Some(ResetCause::ResetPin)    => LedColours::SolidGreen,
        Some(ResetCause::SoftwareBor) => LedColours::BlinkingRed,
        Some(ResetCause::SoftwarePor) => LedColours::BlinkingGreen,
        Some(_)                       => LedColours::BlinkingRedGreen,
        None                          => LedColours::SolidRedGreen,
    };

    loop {
        service_reset_buttons(&mut sw1, &mut sw2);
        control_leds(colour, &mut red, &mut green);
        delay.delay_ms(50);
    }
}

fn service_reset_buttons(sw1: &mut Pin<P2, Pin3, Input<Pullup>>, sw2: &mut Pin<P2, Pin7, Input<Pullup>>) {
    if sw1.is_low().unwrap() {
        while sw1.is_low().unwrap() {}
        Pmm::trigger_bor_reset();
    }
    if sw2.is_low().unwrap() {
        while sw2.is_low().unwrap() {}
        Pmm::trigger_por_reset();
    }
}

fn control_leds(colour: LedColours, red: &mut Pin<P1, Pin0, Output>, green: &mut Pin<P1, Pin1, Output>) {
    match colour {
        LedColours::SolidRed => {
            red.set_high().ok();
            green.set_low().ok();
        },
        LedColours::SolidGreen => {
            red.set_low().ok();
            green.set_high().ok();
        },
        LedColours::SolidRedGreen => {
            red.set_high().ok();
            green.set_high().ok();
        },
        LedColours::BlinkingRed => {
            red.toggle().ok();
            green.set_low().ok();
        },
        LedColours::BlinkingGreen => {
            red.set_low().ok();
            green.toggle().ok();
        },
        LedColours::BlinkingRedGreen => {
            red.toggle().ok();
            green.toggle().ok();
        },
    }
}

#[derive(Copy, Clone)]
enum LedColours {
    SolidRed,
    SolidGreen,
    SolidRedGreen,
    BlinkingRed,
    BlinkingGreen,
    BlinkingRedGreen,
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
