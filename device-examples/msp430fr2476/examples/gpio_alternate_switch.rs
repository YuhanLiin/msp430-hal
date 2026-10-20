//! Switches P1.4 directly between its alternate functions: 1 is the UART output UCA0TXD, 2 the timer output
//! TA1.2 and 3 the analog input A4. P1.4 is the LaunchPad's backchannel UART TX, so probe it on the TXD jumper
//! of J101, and open the COM port of "MSP Application UART1" at 9600 baud.
//!
//! Function test, after reset (LED1 off): P1.4 goes through its three alternate functions in turn.
//! - LED2 red, alternate function 1: the terminal shows "alternate function 1: UART" lines, and the scope the
//!   UART bytes.
//! - LED2 green, alternate function 2: the scope shows a 1 kHz PWM signal with a 25 % duty cycle. The terminal
//!   may show some garbage, since the eZ-FET reads the PWM as UART data.
//! - LED2 blue, alternate function 3: the MSP430 doesn't drive the pin, so there's no UART or PWM activity.
//!   With the TXD jumper removed, a 10 kOhm resistor to GND or 3V3 on the MSP430 side sets the level.
//!
//! Glitch test, hold S1 until LED1 goes on: P1.4 switches between alternate functions 1 and 2 every millisecond.
//! In both, the pin stays high (the UART is idle and TA1.2 outputs a constant high), while P1OUT.4 is low, so
//! any moment the pin spends as a GPIO shows up as a low pulse. Set the scope to trigger once on a falling edge.
//! - LED2 green, the direct switch `to_alternate2()`/`to_alternate1()`: no trigger, the pin stays high.
//! - LED2 red, through `to_gpio()`: a low pulse of about 5 to 10 microseconds at every switch.
//!
//! In the glitch test, S2 switches between the two. Hold S1 again to go back to the function test.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::*};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Alternate1, Batch, Output, Pin, Pin0, Pin1, Pin4, Pin7, P1, P4, P5},
    pmm::Pmm,
    watchdog::Wdt,
};
use msp430fr247x::{EUsciA0, Ta1};
use panic_msp430 as _;

/// P1.4 in alternate function 1, where each switching sequence starts and ends
type SwitchPin = Pin<P1, Pin4, Alternate1<Output>>;

/// The three colours of LED2
struct Led2 {
    red: Pin<P5, Pin1, Output>,
    green: Pin<P5, Pin0, Output>,
    blue: Pin<P4, Pin7, Output>,
}

impl Led2 {
    fn show(&mut self, red: bool, green: bool, blue: bool) {
        self.red.set_state(red.into()).ok();
        self.green.set_state(green.into()).ok();
        self.blue.set_state(blue.into()).ok();
    }
}

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.wdt_a);

    // MCLK and SMCLK at 1 MHz
    let mut fram = Fram::new(periph.frctl);
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).config_pin0(|p| p.to_output()).split(&pmm);
    let p2 = Batch::new(periph.p2).config_pin3(|p| p.pullup()).split(&pmm);
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .config_pin7(|p| p.to_output())
        .split(&pmm);
    let p5 = Batch::new(periph.p5)
        .config_pin0(|p| p.to_output())
        .config_pin1(|p| p.to_output())
        .split(&pmm);
    let mut led1 = p1.pin0;
    let mut led2 = Led2 { red: p5.pin1, green: p5.pin0, blue: p4.pin7 };
    let mut s1 = p4.pin0;
    let mut s2 = p2.pin3;

    // P1OUT.4 stays low, so the pin is only high while an alternate function drives it high
    let mut p1_4 = p1.pin4.to_output();
    p1_4.set_low().ok();
    let mut pin: SwitchPin = p1_4.to_alternate1();

    // The UART and the timer are set up with their registers, since their drivers would take the pin
    let uart = periph.e_usci_a0;
    let timer = periph.ta1;
    setup_uart(&uart);
    setup_timer(&timer);

    let mut glitch_test = false;
    let mut via_gpio = false;
    loop {
        if pressed(&mut s1) {
            glitch_test = !glitch_test;
            timer_output(&timer, glitch_test);
        }

        if glitch_test {
            if pressed(&mut s2) {
                via_gpio = !via_gpio;
            }
            led1.set_high().ok();
            led2.show(via_gpio, !via_gpio, false);
            for _ in 0..250 {
                pin = if via_gpio {
                    let alt2 = pin.to_gpio().to_alternate2();
                    delay.delay_ms(1);
                    alt2.to_gpio().to_alternate1()
                } else {
                    let alt2 = pin.to_alternate2();
                    delay.delay_ms(1);
                    alt2.to_alternate1()
                };
                delay.delay_ms(1);
            }
        } else {
            led1.set_low().ok();

            // Alternate function 1: UCA0TXD
            led2.show(true, false, false);
            for _ in 0..20 {
                uart_write(&uart, b"alternate function 1: UART\r\n");
            }
            uart_flush(&uart);

            // Alternate function 2: TA1.2
            let alt2 = pin.to_alternate2();
            led2.show(false, true, false);
            delay.delay_ms(1000);

            // Alternate function 3: A4
            let alt3 = alt2.to_alternate3();
            led2.show(false, false, true);
            delay.delay_ms(1000);

            pin = alt3.to_alternate1();
        }
    }
}

/// Whether the button was pressed, waiting until it is released
fn pressed<B: InputPin>(button: &mut B) -> bool {
    if button.is_low().unwrap_or(false) {
        while button.is_low().unwrap_or(false) {}
        true
    } else {
        false
    }
}

/// eUSCI_A0 as a UART from SMCLK (1 MHz) at 9600 baud, 8N1
fn setup_uart(uart: &EUsciA0) {
    unsafe {
        uart.uca0ctlw0().write(|w| w.bits(0x0081)); // UCSSEL = SMCLK, held in reset
        uart.uca0brw().write(|w| w.bits(6));
        uart.uca0mctlw().write(|w| w.bits(0x2081)); // UCBRS = 0x20, UCBRF = 8, UCOS16
        uart.uca0ctlw0().write(|w| w.bits(0x0080)); // out of reset
    }
}

fn uart_write(uart: &EUsciA0, bytes: &[u8]) {
    for &byte in bytes {
        while uart.uca0ifg().read().bits() & 0x02 == 0 {} // UCTXIFG
        uart.uca0txbuf().write(|w| unsafe { w.bits(byte as u16) });
    }
}

/// Wait until the last byte is sent
fn uart_flush(uart: &EUsciA0) {
    while uart.uca0statw().read().bits() & 0x01 != 0 {} // UCBUSY
}

/// Timer_A1 in up mode from SMCLK, with a period of 1000 cycles: 1 kHz
fn setup_timer(timer: &Ta1) {
    unsafe {
        timer.ta1ccr0().write(|w| w.bits(999));
        timer.ta1ccr2().write(|w| w.bits(250));
    }
    timer_output(timer, false);
    unsafe { timer.ta1ctl().write(|w| w.bits(0x0214)) }; // TASSEL = SMCLK, MC = up, TACLR
}

/// TA1.2 as a 25 % PWM (reset/set), or a constant high for the glitch test
fn timer_output(timer: &Ta1, constant_high: bool) {
    let cctl = if constant_high { 0x0004 } else { 0x00E0 }; // OUTMOD = 0 with OUT = 1, or OUTMOD = 7
    unsafe { timer.ta1cctl2().write(|w| w.bits(cctl)) };
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
