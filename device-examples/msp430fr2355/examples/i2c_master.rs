#![no_main]
#![no_std]

// An I2C master using the blocking interface.

use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}, i2c::{I2c, Operation}};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    i2c::{GlitchFilter, I2cConfig},
    pmm::Pmm,
    prelude::*,
    watchdog::Wdt,
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    // LED1 (red) on P1.0, LED2 (green) on P6.6 (SLAU680 Figure 18, p. 26)
    let mut red_led = Batch::new(periph.p1).split(&pmm).pin0.to_output();
    let mut green_led = Batch::new(periph.p6).split(&pmm).pin6.to_output();
    let p4 = Batch::new(periph.p4).split(&pmm);

    // UCB1SCL on P4.7 and UCB1SDA on P4.6, P4SELx = 01 (SLASEC4D Table 6-66, p. 102), pins 14 and 15 of
    // the BoosterPack header (SLAU680 Figure 10, p. 15). SDA and SCL "must be connected to a positive
    // supply voltage using a pullup resistor" (SLAU445I 24.3, p. 629); the internal pull-ups are 20 to
    // 50 kOhm (SLASEC4D Table 5-11, p. 43).
    let scl = p4.pin7.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let sda = p4.pin6.pullup().to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut i2c = I2cConfig::new(periph.e_usci_b1, GlitchFilter::Max50ns)
        .as_single_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz (fBitClock = fBRCLK/UCBRx: SLAU445I 24.3.7, p. 642)
        .configure(scl, sda);

    loop {
        // Below are examples of the various I2C methods provided for writing to / reading from the bus.
        // 7- and 10-bit addressing modes are controlled by passing the address as either a u8 or a u16.

        let mut is_ok = true;
        let send_buf = [(1 << 7) + (0b01 << 5), 0b11];

        // Check if anything with this address is present on the bus by
        // sending a zero-byte write and listening for an ACK.
        if let Ok(is_present) = i2c.is_slave_present(0x12_u8) {
            if !is_present {
                is_ok = false;
            }
        }

        // Blocking write. Write two bytes (length of buffer) to address 0x12.
        // If a NACK is recieved the transmission is aborted. (On a NACK "The master must react with either
        // a STOP condition or a repeated START condition": SLAU445I 24.3.5.2.1, p. 637)
        let wr_res = i2c.write(0x12_u8, &send_buf);
        if wr_res.is_err() {
            is_ok = false;
        }

        // Blocking read. Read one byte from address 0x12.
        // Each byte recieved is automatically ACKed, except for the last one which is NACKed.
        // (SLAU445I 24.3.5.2.2, p. 639: after UCTXSTP is set, "The next byte received from the slave is
        // followed by a NACK and a STOP condition.")
        let mut recv = [0];
        let rd_res = i2c.read(0x12_u8, &mut recv);
        if rd_res.is_err() {
            is_ok = false;
        }

        // Do a write then a read within one transaction.
        // Commonly used to read a specific register from the slave.
        // There is no 'stop' between the write and read, only a repeated start.
        // ("Setting UCTXSTT generates a repeated START condition": SLAU445I 24.3.5.2.1, p. 637)
        let wr_rd_res = i2c.write_read(0x12_u8, &send_buf, &mut recv);
        if wr_rd_res.is_err() {
            is_ok = false;
        }

        // Do any arbitrary transaction. One initial start, a repeated
        // start between operations of dissimilar types, and a stop at the end.
        // This particular example is equivalent to the write_read call above.
        let tr_res = i2c.transaction(0x12_u8, &mut [
            Operation::Write(&send_buf), 
            Operation::Read(&mut recv)
        ]);
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
