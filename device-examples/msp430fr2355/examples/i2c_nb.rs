#![no_main]
#![no_std]

// Demonstrates a non-blocking master implementation, and a polling-based slave.
// The master sends a byte to the slave, then switches to read mode. The slave receives the value and echoes it back to the master.

// The non-blocking master interface is lower-level than the blocking version (the embedded-hal `I2c` trait)
// and requires more careful usage.

// eUSCI B1 is configured as the slave. eUSCI B0 is configured as the master.
// Connect:
// P1.2 <--> P4.6
// P1.3 <--> P4.7
// (UCB0SDA to UCB1SDA and UCB0SCL to UCB1SCL: SLASEC4D Table 6-14, p. 72. On the BoosterPack header
// that is pin 10 to pin 15 and pin 9 to pin 14: SLAU680 Figure 10, p. 15.)

use embedded_hal::{digital::{OutputPin, StatefulOutputPin}, delay::DelayNs};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, 
    i2c::{GlitchFilter, I2cConfig, I2cEvent, TransmissionMode}, pmm::Pmm, prelude::*, watchdog::Wdt
};
use panic_msp430 as _;

// Blink the red LED on P1.0 every time an I2C transaction occurs. Green LED on P6.6 is on if Tx/Rx echo is successful.
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
    let m_scl = p1.pin3.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let m_sda = p1.pin2.pullup().to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut i2c_master = I2cConfig::new(periph.e_usci_b0, GlitchFilter::Max50ns)
        .as_single_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz (fBitClock = fBRCLK/UCBRx: SLAU445I 24.3.7, p. 642)
        .configure(m_scl, m_sda);

    const SLAVE_ADDR: u8 = 0x1A;
    let mut i2c_slave = I2cConfig::new(periph.e_usci_b1, GlitchFilter::Max50ns)
        .as_slave(SLAVE_ADDR)
        .configure(sl_scl, sl_sda);

    loop {
        // The master sends a byte then receives a byte.
        // The slave echoes the master's byte back.

        // Master transmit
        i2c_master.send_start(SLAVE_ADDR, TransmissionMode::Transmit);
        const ECHO_TX: u8 = 10;
        // The eUSCI_B slave acknowledges its own address and "automatically acknowledges the received
        // data" (SLAU445I 24.3.9.2, p. 644; SLAU445I 24.3.5.1.2, p. 634)
        let _ = nb::block!(i2c_master.write_tx_buf(ECHO_TX)); // Safe, slave doesn't send NACKs

        // Slave receive
        loop {
            if i2c_slave.poll() == Ok(I2cEvent::WriteStart) { break }
        }
        let byte = unsafe { i2c_slave.read_rx_buf_unchecked() }; // Safe since `poll` returned a Write event

        // Master swaps mode
        i2c_master.send_start(SLAVE_ADDR, TransmissionMode::Receive);

        // Slave transmit
        loop {
            if i2c_slave.poll() == Ok(I2cEvent::ReadStart) { break }
        }
        let _ = nb::block!(i2c_slave.write_tx_buf(byte)); // Safe, infallible

        // Master receive
        // A stop must be scheduled now (rather than *after* reading the Rx buffer),
        // as otherwise the bus will start the next byte then stall waiting for more data
        // (SLAU445I 24.3.5.2.2, p. 639: data is received as long as UCTXSTP is not set, and "If
        // UCBxRXBUF is not read, the master holds the bus during reception of the last data bit")
        i2c_master.schedule_stop();
        let echo_rx = nb::block!(i2c_master.read_rx_buf()).unwrap_or(0); // Safe, slave doesn't send NACKs

        loop {
            if i2c_slave.poll() == Ok(I2cEvent::Stop) { break }
        }

        // Enable the LED if the echoed value matches what was sent.
        // (LED2, green, on P6.6 and LED1, red, on P1.0: SLAU680 Figure 18, p. 26)
        green_led.set_state((echo_rx == ECHO_TX).into()).ok();
        red_led.toggle().ok();
        delay.delay_ms(100);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
