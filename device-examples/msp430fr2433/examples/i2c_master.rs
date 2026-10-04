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
    let periph = msp430fr2433::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.fram);
    // Hold the watchdog (WDTHOLD, SLAU445I Table 12-2, p. 366: after a PUC the WDT runs, SLAU445I 12.2.2,
    // p. 363)
    let _wdt = Wdt::constrain(periph.watchdog_timer);

    // Pmm::new clears LOCKLPM5 (SLAU445I Table 2-7, p. 97). SLASE59F 6.10.3, p. 46 sets the ports up before
    // that; clearing it first leaves the pins inputs until they are set up (SLAU445I 8.3.1, p. 316).
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let port1 = Batch::new(periph.p1).split(&pmm);
    // Red LED1 on P1.0 and green LED2 on P1.1 (SLAU739 Figure 18, p. 23)
    let mut red_led   = port1.pin0.to_output();
    let mut green_led = port1.pin1.to_output();
    red_led.set_low();
    green_led.set_low();

    // P1.2 UCB0SDA and P1.3 UCB0SCL: P1SELx = 01 (SLASE59F Table 6-17, p. 55), LaunchPad header pins
    // J1.10 and J1.9 (SLAU739 Figure 18, p. 23). Both lines need pullups (SLAU445I 24.3, p. 629); the
    // internal ones are 20 to 50 kOhm (SLASE59F Table 5-10, p. 27).
    let sda = port1.pin2.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let scl = port1.pin3.pullup().to_alternate1();

    // MCLK = SMCLK = about 8 MHz: DCORSEL = 011b with the FLL locked to REFO (SLAU445I Table 3-5, p. 114;
    // SLAU445I 3.2.5, p. 104), DIVM and DIVS /1 (SLAU445I Table 3-9, p. 118); no FRAM wait state is needed
    // up to 8 MHz (SLASE59F 5.3, p. 16). ACLK = REFO: SELA = 01b (SLAU445I Table 3-8, p. 117).
    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // 50-ns deglitching: UCGLITx = 00b (SLAU445I Table 24-5, p. 652). Single master: UCMST = 1, UCMM = 0;
    // SMCLK as the bit clock source: UCSSELx = 10b (SLAU445I Table 24-4, p. 649).
    let mut i2c = I2cConfig::new(periph.usci_b0_i2c_mode, GlitchFilter::Max50ns)
        .as_single_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz (fBitClock = fBRCLK/UCBRx, SLAU445I 24.3.7, p. 642)
        .configure(scl, sda);

    loop {
        // Below are examples of the various I2C methods provided for writing to / reading from the bus.
        // 7- and 10-bit addressing modes are controlled by passing the address as either a u8 or a u16.
        // (SLAU445I 24.3.3, p. 630: "The I2C mode supports 7-bit and 10-bit addressing modes.")

        let mut is_ok = true;
        let send_buf = [(1 << 7) + (0b01 << 5), 0b11];

        // Check if anything with this address is present on the bus by
        // sending a zero-byte write and listening for an ACK.
        // (A STOP right after the address "is generated, even if no data was transmitted to the slave",
        // SLAU445I 24.3.5.2.1, p. 637.)
        if let Ok(true) = i2c.is_slave_present(0x12_u8) {}
        else {is_ok = false};

        // Blocking write. Write two bytes (length of buffer) to address 0x12.
        // If a NACK is recieved the transmission is aborted.
        // (A NACK sets UCNACKIFG, and the master must then send a STOP or a repeated START: SLAU445I
        // 24.3.5.2.1, p. 637.)
        let _ = i2c.write(0x12_u8, &send_buf)
            .inspect_err(|_e| is_ok = false);

        // Blocking read. Read one byte from address 0x12.
        // Each byte recieved is automatically ACKed, except for the last one which is NACKed.
        // (SLAU445I 24.3.5.2.2, p. 639: after UCTXSTP, "The next byte received from the slave is followed
        // by a NACK and a STOP condition.")
        let mut recv = [0];
        let _ = i2c.read(0x12_u8, &mut recv)
            .inspect_err(|_e| is_ok = false);

        // Do a write then a read within one transaction.
        // Commonly used to read a specific register from the slave.
        // There is no 'stop' between the write and read, only a repeated start (SLAU445I 24.3.3.3, p. 631).
        let _ = i2c.write_read(0x12_u8, &send_buf, &mut recv)
            .inspect_err(|_e| is_ok = false);

        // Do any arbitrary transaction. One initial start, a repeated
        // start between operations of dissimilar types, and a stop at the end.
        // This particular example is equivalent to the write_read call above.
        let _ = i2c.transaction(0x12_u8, &mut [
            Operation::Write(&send_buf), 
            Operation::Read(&mut recv)
        ]).inspect_err(|_e| is_ok = false);

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
