#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pin_mapping::PinMap, pmm::Pmm, timer::{CapCmp, SubTimer, Timer, TimerConfig, TimerDiv, TimerExDiv, TimerParts3, TimerPeriph}, watchdog::Wdt
};
use nb::block;
use panic_msp430 as _;

// 0.5 second on, 0.5 second off
// (LED1 on P1.0: SLAU802 Figure 19, p. 25. TA0 counts ACLK from the VLO, typically 10 kHz:
// SLASEO7C 8.12.3.5, p. 30, divided by 2 and by 5: SLAU445I 13.2.1.1, p. 370.)
#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut p1_0 = p1.pin0;

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range (SELMS = 000b: SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I
    // Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the data sheet.
    let (_smclk, aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // TA0 counts ACLK (TASSEL = 01b: SLASEO7C Table 9-8, p. 50), divided by 2 with ID and by 5 with
    // TAIDEX (SLAU445I 13.2.1.1, p. 370)
    let parts = TimerParts3::new(
        periph.ta0,
        TimerConfig::aclk(&aclk).clk_div(TimerDiv::_2, TimerExDiv::_5),
    );
    let mut timer = parts.timer;
    let mut subtimer = parts.subtimer2;

    set_time(&mut timer, &mut subtimer, 500);
    loop {
        block!(subtimer.wait()).unwrap();
        p1_0.set_high().unwrap();
        // first 0.5 s of timer countdown expires while subtimer expires, so this should only block
        // for 0.5 s
        // (The sub-timer's CCIFG is set when TAxR counts to its TAxCCRn: SLAU445I 13.2.4.2, p. 376. The
        // timer's TAIFG is set "when the timer counts from TAxCCR0 to zero": SLAU445I 13.2.3.1, p. 371.)
        block!(timer.wait()).unwrap();
        p1_0.set_low().unwrap();
    }
}

/// Start the timer with a period of `2 * delay + 1` counts (up mode: SLAU445I 13.2.3.1, p. 371), and
/// set the sub-timer to fire after `delay` counts (compare mode: SLAU445I 13.2.4.2, p. 376)
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
