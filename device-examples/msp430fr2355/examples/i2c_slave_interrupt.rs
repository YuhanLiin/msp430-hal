#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// This I2C slave implements reading and writing from an 8-byte array (ARR_LEN).
// If a transaction begins with a write, the first byte is treated as the desired array index.
// A subsequent write provides data to store at the specified index. Additional writes will be stored at the following indices, the index autoincrementing after each.
// After any number of writes the master may perform a Repeated Start and switch to reading in order to retrieve the value at the specified index.
// Further reads will read the subsequent indices, autoincrementing.
// If a transaction does not begin with a write, then the index from the previous transaction is used.
// If no previous transaction has been performed the index defaults to 0.

// eUSCI B0 is configured as the master, B1 as the slave.
// Connect:
// P1.2 <--> P4.6.
// P1.3 <--> P4.7.
// (UCB0SDA to UCB1SDA and UCB0SCL to UCB1SCL: SLASEC4D Table 6-14, p. 72. On the BoosterPack header
// that is pin 10 to pin 15 and pin 9 to pin 14: SLAU680 Figure 10, p. 15.)

// This example is quite big (particularly debug builds on old compiler versions), so we use a couple of tricks here to shrink binary size:
// Anything that can panic (like RefCell) will pull in some string formatting, which bloats the binary! Instead use UnsafeCell (carefully).
// Likewise avoid panics from array bounds checks by using .get_unchecked(), etc.

use core::cell::UnsafeCell;

use critical_section::Mutex;
use msp430::interrupt::enable as enable_interrupts;
use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}, i2c::I2c};
use msp430_atomic::AtomicU8;
use msp430_rt::entry;
use msp430fr2355::{interrupt, EUsciB1};
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    i2c::{GlitchFilter, I2cConfig, I2cInterruptFlags as Flags, I2cSlave, I2cVector},
    pmm::Pmm,
    prelude::*,
    watchdog::Wdt,
};
use panic_msp430 as _;

static I2C_SLAVE: Mutex<UnsafeCell<Option< I2cSlave<EUsciB1> >>> = Mutex::new(UnsafeCell::new(None));

// Store the exposed 'registers' as Atomic values, so they can be easily read/written to between the interrupt and main fn
const ARR_LEN: usize = 8;
static ARR: [AtomicU8; ARR_LEN] = [const { AtomicU8::new(0) }; ARR_LEN];

// Red LED on P1.0 should blink rapidly. Green LED on pin P6.6 should blink once for every ten red blinks.
// (LED1, red, on P1.0 and LED2, green, on P6.6: SLAU680 Figure 18, p. 26)
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut red_led = p1.pin0.to_output();
    let mut green_led = Batch::new(periph.p6).split(&pmm).pin6.to_output();
    let p4 = Batch::new(periph.p4).split(&pmm);
    // UCB1SCL on P4.7 and UCB1SDA on P4.6, P4SELx = 01 (SLASEC4D Table 6-66, p. 102)
    let sl_scl = p4.pin7.to_alternate1();
    let sl_sda = p4.pin6.to_alternate1();

    // UCB0SCL on P1.3 and UCB0SDA on P1.2, P1SELx = 01 (SLASEC4D Table 6-63, p. 96). The internal
    // pull-ups are 20 to 50 kOhm (SLASEC4D Table 5-11, p. 43), and I2C needs pull-ups on SDA and SCL
    // (SLAU445I 24.3, p. 629).
    let scl = p1.pin3.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let sda = p1.pin2.pullup().to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut i2c_master = I2cConfig::new(periph.e_usci_b0, GlitchFilter::Max50ns)
        .as_single_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz (fBitClock = fBRCLK/UCBRx: SLAU445I 24.3.7, p. 642)
        .configure(scl, sda);

    const SLAVE_ADDR: u8 = 0x1A;
    let mut i2c_slave = I2cConfig::new(periph.e_usci_b1, GlitchFilter::Max50ns)
        .as_slave(SLAVE_ADDR)
        .configure(sl_scl, sl_sda);

    i2c_slave.set_interrupts(Flags::StopReceived | Flags::RxBufFull | Flags::TxBufEmpty);

    critical_section::with(|cs| {
        unsafe { *I2C_SLAVE.borrow(cs).get() = Some(i2c_slave) }
    });

    unsafe { enable_interrupts(); }

    let index = 0;
    let mut value = 0;
    loop {
        // Write a value between 0 and 9 to index 0.
        let _ = i2c_master.write(SLAVE_ADDR, &[index, value]);
        value = (value + 1) % 10;

        // Enable the green LED if the value at index 0 is 0.
        // (LED2, green, on P6.6 and LED1, red, on P1.0: SLAU680 Figure 18, p. 26)
        green_led.set_state((ARR[0].load() == 0).into()).ok();

        // Toggle the red LED after each
        red_led.toggle().ok();
        delay.delay_ms(100);
    }
}

// Static mut variables defined inside an interrupt handler are safe. See: https://docs.rust-embedded.org/book/start/interrupts.html
// The eUSCI_B1 receive or transmit vector at FFDEh (SLASEC4D Table 6-2, p. 64)
#[allow(static_mut_refs)]
#[interrupt]
fn EUSCI_B1() {
    static mut BYTE_COUNT: u8 = 0; // Bytes since the initial start condition
    static mut ARR_INDEX: usize = 0;

    critical_section::with(|cs| {
        let Some(i2c_slave) = unsafe { &mut *I2C_SLAVE.borrow(cs).get() }.as_mut() else { return; };
        match i2c_slave.interrupt_source() {
            I2cVector::RxBufFull => {
                // Safety: Rx interrupt triggered, so Rx buffer is ready.
                // ("After the first data byte is received, the receive interrupt flag UCRXIFG0 is set":
                // SLAU445I 24.3.5.1.2, p. 634)
                let val = unsafe { i2c_slave.read_rx_buf_unchecked() };
                // If this is the first byte treat the I2C byte as the array index
                if *BYTE_COUNT == 0 {
                    *ARR_INDEX = val as usize % ARR_LEN;
                } else {
                    // Otherwise treat the I2C byte as data to be stored
                    unsafe { ARR.get_unchecked(*ARR_INDEX) }.store(val);
                    *ARR_INDEX = (*ARR_INDEX + 1) % ARR_LEN; // Autoincrement index
                }
            }
            I2cVector::TxBufEmpty => {
                // Safety: ARR_INDEX is always less than ARR_LEN.
                let val = unsafe { ARR.get_unchecked(*ARR_INDEX) }.load();
                // Safety: Tx interrupt triggered, so Tx buffer is ready.
                // (When the master reads, "UCTR and UCTXIFG0 become set" and SCL is held low until data is
                // written to UCBxTXBUF: SLAU445I 24.3.5.1.1, p. 633)
                unsafe { i2c_slave.write_tx_buf_unchecked(val) };
                *ARR_INDEX = (*ARR_INDEX + 1) % ARR_LEN; // Autoincrement index
            }
            I2cVector::StopReceived => {
                *ARR_INDEX = ARR_INDEX.wrapping_sub(1).min(ARR_LEN - 1); // Undo last autoincrement
                *BYTE_COUNT = 0;
                return;
            }
            _ => (), // unreachable
        }
        *BYTE_COUNT += 1;
    })
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
