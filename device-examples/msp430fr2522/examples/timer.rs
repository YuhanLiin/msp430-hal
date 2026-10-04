#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::PinMap,
    pmm::Pmm,
    timer::{CapCmp, SubTimer, Timer, TimerConfig, TimerDiv, TimerExDiv, TimerParts3, TimerPeriph},
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

// 0.5 second on, 0.5 second off
// on P1.0, a GPIO output, P1SELx = 00 and P1DIR = 1 (SLASEE4C Table 6-15, p. 58)
#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut p1_0 = p1.pin0;

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let parts = TimerParts3::new(
        periph.ta0,
        // ACLK (REFO, 32768 Hz) / 32 = 1024 Hz
        // (REFO: SLASEE4C Table 5-7, p. 27. ID divides by 8 and TAIDEX by 4: SLAU445I 13.2.1.1, p. 370)
        TimerConfig::aclk(&aclk).clk_div(TimerDiv::_8, TimerExDiv::_4),
    );
    let mut timer = parts.timer;
    let mut subtimer = parts.subtimer2;

    set_time(&mut timer, &mut subtimer, 512);
    loop {
        block!(subtimer.wait()).unwrap();
        p1_0.set_high().unwrap();
        // first 0.5 s of timer countdown expires while subtimer expires, so this should only block
        // for 0.5 s
        block!(timer.wait()).unwrap();
        p1_0.set_low().unwrap();
    }
}

fn set_time<T: TimerPeriph<M> + CapCmp<C>, C, M: PinMap>(
    timer: &mut Timer<T, M>,
    subtimer: &mut SubTimer<T, C>,
    delay: u16,
) {
    timer.start(delay + delay);
    subtimer.set_count(delay);
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
