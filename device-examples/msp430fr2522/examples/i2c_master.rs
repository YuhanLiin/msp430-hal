//! An I2C master with the blocking `I2c` interface of embedded-hal: once a second it runs five
//! transactions with a device at address 0x68, and an LED on P2.0 lights while they all succeed.
//!
//! eUSCI_B0 is the only master on the bus, at 100 kHz from SMCLK, with the internal pull-ups of SDA and
//! SCL on. The transactions: a write of no bytes, which checks that the device answers; a write of A0h
//! 03h; a read of one byte; a write and a read with a repeated start in between; and that again with
//! `transaction`. An LED on P1.0 toggles every second. There's no LaunchPad for the MSP430FR25x2.
//! (eUSCI_B0's I2C pins, with USCIBRMP = 0: SLASEE4C Table 6-11, p. 53. SDA and SCL need pull-ups, but
//! "must not be pulled up above the device VCC level": SLAU445I 24.3, p. 629. The internal ones are
//! 20 kΩ to 50 kΩ: SLASEE4C Table 5-10, p. 29. P2.0 is also XOUT: SLASEE4C Table 6-16, p. 60.)
//!
//! How to test (an I2C device at address 0x68, two LEDs and resistors, and optionally the scope):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND, and another from P2.0 to GND.
//!    P2.0 must have no crystal on it.
//! 2. Connect the device: SDA to P1.2, SCL to P1.3, its supply to 3.3 V and its ground to GND. The
//!    internal pull-ups may be too weak: if the device's board has none, add resistors from SDA and SCL to
//!    3.3 V.
//! 3. Flash this example.
//! 4. Expected: the LED on P1.0 toggles every second, and the one on P2.0 lights. Without the device it
//!    stays off, as nothing acknowledges the address.
//! 5. Scope, ground on GND, 500 µs/div, trigger on CH1 falling: CH1 on SDA, CH2 on SCL. Five short
//!    transactions each second, each starting with the address. The scope's I2C decoder
//!    (Analysis > Decode) shows them as bytes.
#![no_main]
#![no_std]

use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, StatefulOutputPin},
    i2c::{I2c, Operation},
};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    i2c::{GlitchFilter, I2cConfig, I2cSingleMaster},
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    prelude::*,
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    // No board document covers the LEDs on P1.0 and P2.0: there is none for the MSP430FR25x2. Both pins
    // are GPIO outputs, PxSELx = 00 and PxDIR = 1 (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60).
    let mut red_led = p1.pin0.to_output();
    let mut green_led = Batch::new(periph.p2).split(&pmm).pin0.to_output();

    // UCB0SCL on P1.3 and UCB0SDA on P1.2: P1SELx = 01 in the default mapping, USCIBRMP = 0
    // (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58). The internal pullups are 20 kΩ to 50 kΩ
    // (SLASEE4C Table 5-10, p. 29: RPull).
    let scl = p1.pin3.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let sda = p1.pin2.pullup().to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // fBitClock = fBRCLK / UCBRx (SLAU445I 24.3.7, p. 642); 100 kHz is the I2C standard mode
    // (SLAU445I 24.2, p. 627: "standard mode up to 100 kbps")
    let mut i2c: I2cSingleMaster<_, DefaultMapping> =
        I2cConfig::new(periph.e_usci_b0, GlitchFilter::Max50ns)
            .as_single_master()
            .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
            .configure(scl, sda);

    const SLAVE_ADDR: u8 = 0x68;

    loop {
        // Below are examples of the various I2C methods provided for writing to / reading from the bus.
        // 7- and 10-bit addressing modes are controlled by passing the address as either a u8 or a u16.
        // (SLAU445I 24.2, p. 627: "7-bit and 10-bit device addressing modes")

        let mut is_ok = true;
        let send_buf = [(1 << 7) + (0b01 << 5), 0b11];

        // Check if anything with this address is present on the bus by
        // sending a zero-byte write and listening for an ACK.
        // (SLAU445I 24.3.5.2.1, p. 637: the STOP is generated "even if no data was transmitted to the slave")
        if let Ok(is_present) = i2c.is_slave_present(SLAVE_ADDR) {
            if !is_present {
                is_ok = false;
            }
        }

        // Blocking write. Write two bytes (length of buffer) to SLAVE_ADDR.
        // If a NACK is recieved the transmission is aborted.
        // (UCNACKIFG, after which "The master must react with either a STOP condition or a repeated START
        // condition": SLAU445I 24.3.5.2.1, p. 637)
        let wr_res = i2c.write(SLAVE_ADDR, &send_buf);
        if wr_res.is_err() {
            is_ok = false;
        }

        // Blocking read. Read one byte from SLAVE_ADDR.
        // Each byte recieved is automatically ACKed, except for the last one which is NACKed.
        // (SLAU445I 24.3.5.2.2, p. 639: "The next byte received from the slave is followed by a NACK and
        // a STOP condition")
        let mut recv = [0];
        let rd_res = i2c.read(SLAVE_ADDR, &mut recv);
        if rd_res.is_err() {
            is_ok = false;
        }

        // Do a write then a read within one transaction.
        // Commonly used to read a specific register from the slave.
        // There is no 'stop' between the write and read, only a repeated start.
        // (SLAU445I 24.3.5.2.1, p. 637: "Setting UCTXSTT generates a repeated START condition")
        let wr_rd_res = i2c.write_read(SLAVE_ADDR, &send_buf, &mut recv);
        if wr_rd_res.is_err() {
            is_ok = false;
        }

        // Do any arbitrary transaction. One initial start, a repeated
        // start between operations of dissimilar types, and a stop at the end.
        // This particular example is equivalent to the write_read call above.
        let tr_res = i2c.transaction(
            SLAVE_ADDR,
            &mut [Operation::Write(&send_buf), Operation::Read(&mut recv)],
        );
        if tr_res.is_err() {
            is_ok = false;
        }

        green_led.set_state(is_ok.into()).ok();
        red_led.toggle().ok();
        delay.delay_ms(1000);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
