#![no_main]
#![no_std]

// An I2C master using the blocking interface.

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
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut red_led = p1.pin0.to_output();
    let mut green_led = Batch::new(periph.p2).split(&pmm).pin0.to_output();

    let scl = p1.pin3.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let sda = p1.pin2.pullup().to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let mut i2c: I2cSingleMaster<_, DefaultMapping> =
        I2cConfig::new(periph.e_usci_b0, GlitchFilter::Max50ns)
            .as_single_master()
            .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
            .configure(scl, sda);

    const SLAVE_ADDR: u8 = 0x68;

    loop {
        // Below are examples of the various I2C methods provided for writing to / reading from the bus.
        // 7- and 10-bit addressing modes are controlled by passing the address as either a u8 or a u16.

        let mut is_ok = true;
        let send_buf = [(1 << 7) + (0b01 << 5), 0b11];

        // Check if anything with this address is present on the bus by
        // sending a zero-byte write and listening for an ACK.
        if let Ok(is_present) = i2c.is_slave_present(SLAVE_ADDR) {
            if !is_present {
                is_ok = false;
            }
        }

        // Blocking write. Write two bytes (length of buffer) to SLAVE_ADDR.
        // If a NACK is recieved the transmission is aborted.
        let wr_res = i2c.write(SLAVE_ADDR, &send_buf);
        if wr_res.is_err() {
            is_ok = false;
        }

        // Blocking read. Read one byte from SLAVE_ADDR.
        // Each byte recieved is automatically ACKed, except for the last one which is NACKed.
        let mut recv = [0];
        let rd_res = i2c.read(SLAVE_ADDR, &mut recv);
        if rd_res.is_err() {
            is_ok = false;
        }

        // Do a write then a read within one transaction.
        // Commonly used to read a specific register from the slave.
        // There is no 'stop' between the write and read, only a repeated start.
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
