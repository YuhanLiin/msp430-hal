//! Infrared remote control: the infrared modulator sends an NEC remote control frame (address 0x00,
//! command 0xA5) on P1.4 every 108 ms, as a remote control does to drive its IR LED.
//!
//! TA0's CCR2 output is the 38 kHz carrier, and TA1's CCR2 output, the envelope, is held low. In ASK
//! mode the modulator then outputs the carrier while the data bit is 1, and stays low while it's 0.
//! P1.4 is eUSCI_A0's TXD pin, which the modulator takes over. An NEC frame is a 9 ms burst, a 4.5 ms
//! space, then 32 bits, least significant bit first: the address, its inverse, the command and its
//! inverse. Each bit is a 562 µs burst followed by a 562 µs space for 0 or a 1687 µs space for 1, and a
//! last 562 µs burst ends the frame.
//! (Carrier and coding inputs: SLASEO7C Table 9-12, p. 55 and SLASEO7C Table 9-13, p. 56. The ASK
//! logic is drawn for the MSP430FR2433 in SLAU445I Figure 1-8, p. 50. The modulator drives "the
//! eUSCI_A pin of UCA0TXD/UCA0SIMO": SLASEO7C 9.10.8, p. 60. SLASEO7C Figure 9-2, p. 57 labels that
//! pin P2.0, but measured on an MSP430FR2476 the output is on P1.4, with eUSCI_A0 in its default
//! mapping.)
//!
//! How to test (the scope):
//! 1. P1.4 isn't on the BoosterPack headers: it goes to the debug probe's backchannel UART through the
//!    TXD jumper of J101. Pull that jumper off, connect the probe tip to the TXD pin on the MSP430 side,
//!    away from the USB connector, and the ground clip to GND (J3 pin 22). (J101: SLAU802 Table 2, p. 8;
//!    its target-side TXD pin is P1.4_UART_TX: SLAU802 Figure 16, p. 22; board layout: SLAU802 Figure 1,
//!    p. 1; header pins: SLAU802 Figure 10, p. 13.)
//! 2. Flash this example.
//! 3. Scope: 1 V/div, 10 ms/div, trigger on a rising edge at 1.5 V in normal mode. Expected: a frame
//!    every 108 ms, as blocks: the long first burst, the space, then the 32 bits. At 20 µs/div a burst
//!    shows the carrier, a square wave with a period of 26 µs.
//! 4. Put the TXD jumper back for the examples that use the backchannel UART.
#![no_main]
#![no_std]

use embedded_hal::delay::DelayNs;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    ir::{IrMode, IrModulator, SoftwareData},
    pmm::Pmm,
    pwm::{PwmParts3, TimerConfig},
    watchdog::Wdt,
};
use panic_msp430 as _;

/// 38 kHz carrier period, in cycles of the 8 MHz SMCLK
const CARRIER_PERIOD: u16 = 210;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range, ACLK from REFO (SELMS = 000b, SELA = 01b: SLAU445I
    // Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118). The timers count SMCLK (TASSEL = 10b:
    // SLASEO7C Table 9-8, p. 50).
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The carrier, high for half of each period
    // (TA0's CCR2 output is the "IR carrier input": SLASEO7C Table 9-12, p. 55)
    let carrier = PwmParts3::new(periph.ta0, TimerConfig::smclk(&smclk), CARRIER_PERIOD - 1)
        .pwm2
        .into_ir_input(CARRIER_PERIOD / 2);
    // The envelope stays low, so the data bit alone switches the carrier on and off
    // (TA1's CCR2 output is the "IR coding input": SLASEO7C Table 9-13, p. 56)
    let envelope = PwmParts3::new(periph.ta1, TimerConfig::smclk(&smclk), CARRIER_PERIOD - 1)
        .pwm2
        .into_ir_input(0);
    // P1.4 = UCA0TXD with P1SEL = 01 (SLASEO7C Table 9-23, p. 65). The data bit is IRDATA, set by
    // software (SLASEO7C 9.10.8, p. 60 to p. 61). In SYSCFG1: IREN = 1, IRMSEL = 0 for ASK, IRPSEL = 0
    // for normal polarity, IRDSSEL = 1 for data "From IRDATA bit" (SLAU445I Table 1-30, p. 81).
    let mut ir = IrModulator::with_software_data(&carrier, &envelope, IrMode::Ask, false, p1.pin4.to_alternate1());

    loop {
        send_nec(&mut ir, &mut delay, 0x00, 0xA5);
        // Frames start every 108 ms; this one took about 68 ms
        delay.delay_ms(40);
    }
}

/// Send one NEC frame
fn send_nec(ir: &mut IrModulator<SoftwareData>, delay: &mut impl DelayNs, address: u8, command: u8) {
    burst(ir, delay, 9000, 4500);
    for byte in [address, !address, command, !command] {
        for bit in 0..8 {
            let space_us = if byte >> bit & 1 == 1 { 1687 } else { 562 };
            burst(ir, delay, 562, space_us);
        }
    }
    burst(ir, delay, 562, 0);
}

/// Send the carrier for `on_us`, then nothing for `off_us`
fn burst(ir: &mut IrModulator<SoftwareData>, delay: &mut impl DelayNs, on_us: u32, off_us: u32) {
    ir.set_data(true);
    delay.delay_us(on_us);
    ir.set_data(false);
    delay.delay_us(off_us);
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
