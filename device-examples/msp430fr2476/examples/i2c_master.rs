//! An I2C master with the blocking `I2c` interface of embedded-hal: once a second it runs five
//! transactions with a device at address 0x68, and LED2 lights green while they all succeed.
//!
//! eUSCI_B1 is the only master on the bus, at 100 kHz from SMCLK, with the internal pull-ups of SDA and
//! SCL on. The transactions: a write of no bytes, which checks that the device answers; a write of A0h
//! 03h; a read of one byte; a write and a read with a repeated start in between; and that again with
//! `transaction`. LED1 toggles every second.
//! (eUSCI_B1's I2C pins: SLASEO7C Table 9-11, p. 54. SDA and SCL need pull-ups, but "must not be pulled
//! up above the device VCC level": SLAU445I 24.3, p. 629. The internal ones are 20 kΩ to 50 kΩ: SLASEO7C
//! 8.12.4.1, p. 31. The LaunchPad has none on these pins: SLAU802 Figure 18, p. 24. LED1 on P1.0 is
//! green, and LED2's green part is on P5.0: SLAU802 Figure 19, p. 25.)
//!
//! How to test (an I2C device at address 0x68, and optionally the scope):
//! 1. Connect the device: SDA to P3.2 (J2 pin 15), SCL to P3.6 (J2 pin 14), its supply to 3.3 V (J1
//!    pin 1) and its ground to GND (J3 pin 22). The internal pull-ups may be too weak: if the device's
//!    board has none, add resistors from SDA and SCL to 3.3 V. (Header pins: SLAU802 Figure 10, p. 13.)
//! 2. Flash this example.
//! 3. Expected: LED1 toggles every second, and LED2 lights green. Without the device LED2 stays off, as
//!    nothing acknowledges the address.
//! 4. Scope, ground on GND (J3 pin 22), 500 µs/div, trigger on CH1 falling: CH1 on SDA, CH2 on SCL. Five
//!    short transactions each second, each starting with the address. The scope's I2C decoder
//!    (Analysis > Decode) shows them as bytes.
#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}, i2c::{I2c, Operation}};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, i2c::{GlitchFilter, I2cConfig, I2cSingleMaster}, pin_mapping::DefaultMapping, pmm::Pmm, prelude::*, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // LED1 on P1.0, green, toggles every second; P5.0 is the green part of LED2, on while every
    // transaction succeeds (SLAU802 Figure 19, p. 25)
    let mut led1 = Batch::new(periph.p1).split(&pmm).pin0.to_output();
    let mut green_led = Batch::new(periph.p5).split(&pmm).pin0.to_output();
    let p3 = Batch::new(periph.p3).split(&pmm);

    // eUSCI_B1: P3.6 = UCB1SCL and P3.2 = UCB1SDA with P3SEL = 01 (SLASEO7C Table 9-25, p. 67;
    // SLASEO7C Table 9-11, p. 54), on J2 pins 14 and 15 (SLAU802 Figure 10, p. 13). The internal
    // pullups are 20 kΩ to 50 kΩ (SLASEO7C 8.12.4.1, p. 31).
    let scl = p3.pin6.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let sda = p3.pin2.pullup().to_alternate1();

    // MCLK = SMCLK = DCOCLKDIV in the 8 MHz range (SELMS = 000b: SLAU445I Table 3-8, p. 117; DIVM,
    // DIVS: SLAU445I Table 3-9, p. 118). ACLK from the VLO: SLASEO7C 9.10.2, p. 49; SLAU445I
    // Table 3-1, p. 98 lists that for the enhanced clock system only, and the HAL follows the data sheet.
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // UCGLITx = 00b filters pulses of up to 50 ns (SLAU445I 24.3.6, p. 642). SMCLK is UCSSEL = 10b
    // (SLASEO7C Table 9-8, p. 50), and fBitClock = fBRCLK/UCBRx (SLAU445I 24.3.7, p. 642).
    let mut i2c: I2cSingleMaster<_, DefaultMapping> = I2cConfig::new(periph.e_usci_b1, GlitchFilter::Max50ns)
        .as_single_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
        .configure(scl, sda);

    const SLAVE_ADDR: u8 = 0x68;

    loop {
        // Below are examples of the various I2C methods provided for writing to / reading from the bus.
        // 7- and 10-bit addressing modes are controlled by passing the address as either a u8 or a u16.
        // (Both modes: SLAU445I 24.3.3, p. 630)

        let mut is_ok = true;
        let send_buf = [(1 << 7) + (0b01 << 5), 0b11];

        // Check if anything with this address is present on the bus by
        // sending a zero-byte write and listening for an ACK.
        // (A STOP can follow the address "even if no data was transmitted": SLAU445I 24.3.5.2.1, p. 637)
        if let Ok(is_present) = i2c.is_slave_present(SLAVE_ADDR) {
            if !is_present {
                is_ok = false;
            }
        }

        // Blocking write. Write two bytes (length of buffer) to address SLAVE_ADDR.
        // If a NACK is recieved the transmission is aborted.
        // (On a NACK the master must send a STOP or a repeated START: SLAU445I 24.3.5.2.1, p. 637)
        let wr_res = i2c.write(SLAVE_ADDR, &send_buf);
        if wr_res.is_err() {
            is_ok = false;
        }

        // Blocking read. Read one byte from address SLAVE_ADDR.
        // Each byte recieved is automatically ACKed, except for the last one which is NACKed.
        // (SLAU445I 24.3.5.2.2, p. 639)
        let mut recv = [0];
        let rd_res = i2c.read(SLAVE_ADDR, &mut recv);
        if rd_res.is_err() {
            is_ok = false;
        }

        // Do a write then a read within one transaction.
        // Commonly used to read a specific register from the slave.
        // There is no 'stop' between the write and read, only a repeated start.
        // ("Setting UCTXSTT generates a repeated START condition": SLAU445I 24.3.5.2.1, p. 637)
        let wr_rd_res = i2c.write_read(SLAVE_ADDR, &send_buf, &mut recv);
        if wr_rd_res.is_err() {
            is_ok = false;
        }

        // Do any arbitrary transaction. One initial start, a repeated
        // start between operations of dissimilar types, and a stop at the end.
        // This particular example is equivalent to the write_read call above.
        let tr_res = i2c.transaction(SLAVE_ADDR, &mut [
            Operation::Write(&send_buf), 
            Operation::Read(&mut recv)
        ]);
        if tr_res.is_err() {
            is_ok = false;
        }

        green_led.set_state(is_ok.into()).ok();
        led1.toggle().ok();
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
