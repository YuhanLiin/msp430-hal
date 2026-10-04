#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430_hal::{
    ecomp::{ECompConfig, FilterStrength, Hysteresis, NegativeInput, OutputPolarity, PositiveInput, PowerMode},
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

// Configure one of the enhanced comparator (eCOMP) modules for use: If P2.2 is less than 1.2V then LED turns on
// (P2.2 is J1 pin 5: SLAU802 Figure 10, p. 13. The LED is LED1 on P1.0: SLAU802 Figure 19, p. 25.)

#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    // (WDTHOLD = 1 stops it: SLAU445I Table 12-2, p. 366; after a PUC it runs: SLAU445I 12.2.2, p. 363)
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Configure GPIO
    // (Pin settings take effect once LOCKLPM5 is cleared, which Pmm::new does: SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p2 = Batch::new(periph.p2).split(&pmm);
    let mut led = p1.pin0.to_output();

    // eCOMP configuration
    let (_dac_conf, comp_conf) = ECompConfig::begin(periph.e_comp0);

    // V+ is the low-power 1.2-V reference and V- is COMP0.1 on P2.2 (SLASEO7C Table 9-21, p. 63), with
    // P2SEL = 11 (SLASEO7C Table 9-24, p. 66). The output is high while V+ > V- (SLAU445I 18.2.1, p. 505).
    // CPPSEL and CPNSEL pick the inputs (SLAU445I Table 18-2, p. 509); CPINV = 0 (noninverted output),
    // CPMSEL = 1 (low-power mode), CPHSEL = 00b (no hysteresis) and CPFLT = 0 (no filter) are in
    // SLAU445I Table 18-3, p. 510.
    let mut comparator = comp_conf.configure(
            PositiveInput::_1V2,
            NegativeInput::COMPx_1(p2.pin2.to_alternate3()),
            OutputPolarity::Noninverted,
            PowerMode::LowPower,
            Hysteresis::Off,
            FilterStrength::Off,
        ).no_output_pin();

    // If P2.2 is less than 1.2V then LED turns on
    loop {
        led.set_state(comparator.value().into()).ok();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
