//! Timer_B counting modes: up mode, up/down mode, pausing and resuming, and the counter length.
//!
//! TB0 counts SMCLK at 1 MHz with a target count of 499, and P1.0 (LED1) toggles at the end of each
//! timer period, so the scope sees a square wave of half the timer's frequency:
//! - Up mode counts 0 to 499: a period of 500 µs, so P1.0 toggles at 1 kHz, a 1 ms square wave.
//! - Up/down mode counts 0 to 499 and back to 0: a period of 998 µs, so P1.0 is a 501 Hz square wave.
//!
//! Button S1 switches between the two modes. Button S2 pauses the timer, and P1.0 stops; pressing S2
//! again resumes it in the mode it was in, at the same frequency.
//! (Up mode: SLAU445I 14.2.3.1, p. 394: "The number of timer counts in the period is TBxCL0 + 1".
//! Up/down mode: SLAU445I 14.2.3.4, p. 396. Stop mode, MC = 00b: SLAU445I Table 14-1, p. 394. LED1 is
//! P1.0, S1 is P4.0 and S2 is P2.3: SLAU802 Figure 19, p. 25.)
//!
//! `COUNTER_LENGTH` sets how many bits TB0 counts with. With `CounterLength::_8Bit` the timer counts no
//! higher than 255 (CNTL = 11b: SLAU445I Table 14-6, p. 409), below the target of 499, so it counts 0 to
//! 255 over and over, as in continuous mode: a period of 256 µs in both modes, so P1.0 is a 1953 Hz square
//! wave. (Up/down mode: SLAU445I 14.2.3.4, p. 396, note "TBxCL0 > TBxR(max)": "the counter operates as if
//! it were configured for continuous mode". Measured on an MSP430FR2476: up mode does the same.)
//!
//! How to test (scope):
//! 1. Probe P1.0 (J3 pin 27), ground clip on GND (J3 pin 22). Flash this example: a 1 kHz square wave.
//!    (Header pins: SLAU802 Figure 10, p. 13.)
//! 2. Press S1: 501 Hz (up/down mode). Press S1 again: 1 kHz.
//! 3. Press S2: the square wave stops. Press S2 again: it continues at the same frequency.
//! 4. Set `COUNTER_LENGTH` to `CounterLength::_8Bit` and flash again: 1953 Hz in both modes.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    timer::{CounterLength, TimerConfig, TimerParts7},
    watchdog::Wdt,
};
use panic_msp430 as _;

/// How many bits TB0 counts with
const COUNTER_LENGTH: CounterLength = CounterLength::_16Bit;
/// The count up mode counts to, and up/down mode counts up to
const COUNT: u16 = 499;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    // S1 pulls P4.0 low and S2 P2.3, each with the internal pullup on (PxDIR = 0, PxREN = 1, PxOUT = 1:
    // SLAU445I Table 8-1, p. 313)
    let p2 = Batch::new(periph.p2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .split(&pmm);
    let mut led1 = p1.pin0.to_output_low();
    let mut s1 = p4.pin0;
    let mut s2 = p2.pin3;

    // MCLK = DCOCLKDIV in the 8 MHz range, so the CPU keeps up with the timer, SMCLK = MCLK / 8 = 1 MHz,
    // and ACLK from REFO (SELMS = 000b, SELA = 01b: SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I
    // Table 3-9, p. 118)
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_8)
        .aclk_refoclk()
        .freeze(&mut fram);

    // TB0 counts SMCLK (TBSSEL = 10b: SLAU445I Table 14-6, p. 409)
    let config = TimerConfig::smclk(&smclk).counter_length(COUNTER_LENGTH);
    let mut timer = TimerParts7::new(periph.tb0, config).timer;

    let mut up_down = false;
    let mut paused = false;
    timer.start(COUNT);

    loop {
        // TBIFG is set at the end of each period: when the count goes from TBxCL0 to 0 in up mode, and when
        // it gets back down to 0 in up/down mode (SLAU445I 14.2.3.1, p. 394; SLAU445I 14.2.3.4, p. 396)
        if timer.wait().is_ok() {
            led1.toggle().ok();
        }

        if s1.is_low().unwrap() {
            // Starting clears the timer (TBCLR: SLAU445I 14.2.2, p. 393)
            up_down = !up_down;
            if up_down {
                timer.start_up_down(COUNT);
            } else {
                timer.start(COUNT);
            }
            paused = false;
            wait_for_release(&mut s1, &mut delay);
        }

        if s2.is_low().unwrap() {
            paused = !paused;
            if paused {
                timer.pause();
            } else {
                // Continues from the current count, in the same mode and direction (SLAU445I 14.2.3.4,
                // p. 396)
                timer.resume();
            }
            wait_for_release(&mut s2, &mut delay);
        }
    }
}

/// Wait until the button is released and has stopped bouncing
fn wait_for_release(button: &mut impl InputPin, delay: &mut impl DelayNs) {
    while button.is_low().unwrap() {}
    delay.delay_ms(20);
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
