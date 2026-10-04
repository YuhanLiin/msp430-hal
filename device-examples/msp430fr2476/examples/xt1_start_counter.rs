//! XT1 start fault counter (ENSTFCNT1) in bypass mode.
//!
//! With the start counter enabled, `freeze()` should only return once XT1 has run cleanly for
//! 1024 cycles (device data sheets, and measured on the FR2476). LED1 is switched on right
//! after `freeze()` returns.
//! (SLASEO7C 8.12.3.1 note 9, p. 27: "start-up counter of 1024 clock cycles"; measured on an
//! MSP430FR2476. SLAU445I 3.2.13, p. 110 gives 8192 for bypass mode, which the measurement
//! contradicts. Start counter enable, ENSTFCNT1: SLAU445I Table 3-11, p. 121. LED1 on P1.0 is green:
//! SLAU802 Figure 19, p. 25.)
//!
//! At 32.768 kHz those 1024 cycles take only 31 ms, too short to see, so this example runs the
//! generator at 4.096 kHz, where they take 250 ms. Don't go below about 4 kHz: XT1 may count as
//! faulty under 3.5 kHz (SLASEO7C 8.12.3.1, p. 27: fFault,LFXT is at most 3500 Hz).
//!
//! Wiring: function generator -> P2.1/XIN (J2 pin 18), ground -> J2 pin 20. Square wave,
//! 4.096 kHz, 0 V to 3.3 V, 50 % duty, output load High-Z (see `xt1_bypass_aclk.rs`).
//!
//! Scope: the generator signal on CH3 through a BNC T-piece, CH1 on P1.0/LED1 (J3 pin 27).
//! (Header pins: SLAU802 Figure 10, p. 13.)
//! Single-shot trigger on the CH1 rising edge at 100 ms/div, with the trigger point near the
//! right of the screen.
//!
//! What to try:
//! 1. Switch the generator output off and reset the board. LED1 should stay off: `freeze()` is
//!    waiting for XT1. If LED1 turns on with no signal at all, the start-up wait does not
//!    block in bypass mode.
//! 2. Switch the generator on. LED1 should turn on ~250 ms after the first XIN edge.
//! 3. Repeat at 8.192 kHz: the delay should halve to ~125 ms.
//! 4. Set `START_COUNTER` to false and repeat: LED1 should turn on almost immediately after
//!    the first edge.
#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Xt1Config},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

/// Frequency the function generator is set to
const XT1_FREQ_HZ: u32 = 4_096;
/// Whether `freeze()` waits for the 1024-cycle start counter
const START_COUNTER: bool = true;

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
    let p2 = Batch::new(periph.p2).split(&pmm);
    let mut led = p1.pin0;
    led.set_low().ok();

    // ACLK on P2.2 with P2SEL = 10 and P2DIR = 1, XIN on P2.1 with P2SEL = 01 (SLASEO7C Table 9-24, p. 66)
    let _aclk_out = p2.pin2.to_output().to_alternate2();
    let xin = p2.pin1.to_alternate1();

    // XT1 in bypass mode (XT1BYPASS = 1: SLAU445I Table 3-10, p. 120), optionally with the start
    // counter (ENSTFCNT1 = 1: SLAU445I Table 3-11, p. 121)
    let xt1 = Xt1Config::bypass(XT1_FREQ_HZ, xin);
    let xt1 = if START_COUNTER { xt1.enable_start_counter() } else { xt1 };

    // MCLK = SMCLK = DCOCLKDIV (SELMS = 000b) and ACLK from XT1CLK (SELA = 00b) (SLAU445I Table 3-8,
    // p. 117)
    let (_smclk, _aclk, _xt1clk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .xt1clk_on(xt1)
        .aclk_xt1clk()
        .freeze(&mut fram);

    led.set_high().ok();

    loop {
        msp430::asm::nop();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
