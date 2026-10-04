//! Interrupt vectors in RAM: the program changes which function handles an interrupt while it runs.
//!
//! The watchdog, as an interval timer, requests its interrupt every 0.25 s. Its vector in the RAM table
//! points at a handler that toggles LED1 (green), or at one that toggles the red part of LED2. Each press of
//! button S1 points it at the other handler, so the other LED blinks.
//! (Interrupt vectors in RAM: SLAU445I 1.3.6.1, p. 36. The watchdog's interval mode: SLAU445I 12.2.3,
//! p. 363. LED1 is P1.0, the red part of LED2 is P5.1, and S1 is P4.0: SLAU802 Figure 19, p. 25.)
//!
//! Before building this example, change `RAM LENGTH` in `memory.x` from 0x2000 to 0x1F80. The RAM table
//! takes the top 128 bytes of RAM, 3F80h to 3FFFh, and the stack starts at the end of RAM, so RAM must
//! leave them out, or the table and the stack overwrite each other. The other examples work either way.
//! Cargo doesn't notice changes to `memory.x`, so clean the examples after the change, or they keep the old
//! RAM length: `cargo clean -p msp430fr247x-hal-examples --target msp430-none-elf`.
//!
//! How to test:
//! 1. Change `memory.x` and clean the examples as above, then flash this example. LED1 (green) blinks twice
//!    a second.
//! 2. Press S1: LED1 stops, and LED2 blinks red instead.
//! 3. Press S1 again: LED1 blinks again.
//!
//! The RAM table stays in use until the next brownout reset (BOR), even when another program is flashed,
//! and that program's interrupts would then go to this example's handlers. So after flashing the next
//! example, press the reset button S3 once, which is a BOR, or unplug the USB cable and plug it back in.
//! (SYSRIVECT is reset by a BOR only: SLAU445I Table 1-13, p. 66. The RST pin causes a BOR: SLAU445I 1.2,
//! p. 30. S3 is the reset button: SLAU802 Figure 19, p. 25.)
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::RefCell;
use critical_section::with;
use embedded_hal::{delay::DelayNs, digital::*};
use msp430::interrupt::{enable as enable_interrupts, Mutex};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Batch, Output, Pin, Pin0, Pin1, P1, P5},
    pmm::Pmm,
    sys::SysParts,
    watchdog::{Wdt, WdtClkPeriods},
};
use msp430fr247x::Interrupt;
use panic_msp430 as _;

static LED1: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static LED2_RED: Mutex<RefCell<Option<Pin<P5, Pin1, Output>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog, then use it as an interval timer (WDTHOLD, WDTTMSEL: SLAU445I Table 12-2,
    // p. 366)
    let mut wdt = Wdt::constrain(periph.wdt_a).to_interval();

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // S1 pulls P4.0 low, with the internal pullup on (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I
    // Table 8-1, p. 313)
    let p1 = Batch::new(periph.p1).split(&pmm);
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .split(&pmm);
    let p5 = Batch::new(periph.p5).split(&pmm);
    let mut s1 = p4.pin0;
    with(|cs| {
        LED1.borrow_ref_mut(cs).replace(p1.pin0.to_output_low());
        LED2_RED.borrow_ref_mut(cs).replace(p5.pin1.to_output_low());
    });

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO, 32.768 kHz (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; REFO: SLASEO7C 8.12.3.4, p. 30)
    let (_smclk, aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // Copy the interrupt vectors from FRAM to the top of RAM, and take them from there (SYSRIVECT = 1:
    // SLAU445I Table 1-13, p. 66). Then point the watchdog's vector at the first handler.
    let mut sys = SysParts::new(periph.sfr);
    unsafe {
        sys.interrupt_vectors.use_ram();
        sys.interrupt_vectors.set_handler(Interrupt::WDT, blink_led1);
    }

    // The watchdog counts ACLK (WDTSSEL = 01b) and requests its interrupt every 8192 cycles, 0.25 s
    // (WDTIS = 101b: SLAU445I Table 12-2, p. 366; WDTIE: SLAU445I Table 1-9, p. 62)
    wdt.set_aclk(&aclk)
        .enable_interrupts()
        .set_interval_and_start(WdtClkPeriods::_8192);

    // Set GIE, which masks every maskable interrupt while clear (SLAU445I 1.3.3, p. 33)
    unsafe { enable_interrupts() };

    let mut led1_blinks = true;
    loop {
        // Wait for a press and release of S1, and for the bouncing to stop
        while s1.is_high().unwrap() {}
        while s1.is_low().unwrap() {}
        delay.delay_ms(20);

        // One word is written, so the interrupt never sees half a vector
        led1_blinks = !led1_blinks;
        let handler: extern "msp430-interrupt" fn() = if led1_blinks { blink_led1 } else { blink_led2_red };
        unsafe { sys.interrupt_vectors.set_handler(Interrupt::WDT, handler) };

        // Turn both LEDs off, so only the blinking one lights
        with(|cs| {
            LED1.borrow_ref_mut(cs).as_mut().map(|led| led.set_low().ok());
            LED2_RED.borrow_ref_mut(cs).as_mut().map(|led| led.set_low().ok());
        });
    }
}

// The handlers for the RAM table: functions with the interrupt calling convention. (`#[interrupt]` puts a
// handler into the FRAM table instead.) In interval mode "WDTIFG is reset automatically by servicing the
// interrupt" (SLAU445I Table 1-10, p. 63).
extern "msp430-interrupt" fn blink_led1() {
    with(|cs| {
        LED1.borrow_ref_mut(cs).as_mut().map(|led| led.toggle().ok());
    });
}

extern "msp430-interrupt" fn blink_led2_red() {
    with(|cs| {
        LED2_RED.borrow_ref_mut(cs).as_mut().map(|led| led.toggle().ok());
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
