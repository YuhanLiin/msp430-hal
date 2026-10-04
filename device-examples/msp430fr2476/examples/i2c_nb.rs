//! A non-blocking I2C master and a polled I2C slave on the same chip: ten times a second the master
//! writes the byte 10 to the slave and reads one byte back, and the slave answers with the byte it got.
//! LED1 lights while the answer is right, and LED2 toggles red after each exchange.
//!
//! eUSCI_B0 is the master, at 100 kHz from SMCLK, with the internal pull-ups on its pins; eUSCI_B1 is
//! the slave, at address 0x1A. The non-blocking master interface is lower-level than the blocking
//! embedded-hal `I2c` trait and needs more care: the code sends each start, schedules the stop and moves
//! each byte itself.
//! (The I2C pins of eUSCI_B0 and eUSCI_B1: SLASEO7C Table 9-11, p. 54. LED1 on P1.0 is green, and LED2's
//! red part is on P5.1: SLAU802 Figure 19, p. 25.)
//!
//! How to test (two jumper wires):
//! 1. Connect SDA, P1.2 (J1 pin 10), to P3.2 (J2 pin 15), and SCL, P1.3 (J1 pin 9), to P3.6 (J2 pin 14).
//!    (Header pins: SLAU802 Figure 10, p. 13.)
//! 2. Flash this example.
//! 3. Expected: LED2 toggles red every 100 ms, and LED1 lights. Without the wires nothing acknowledges the
//!    master, the slave waits forever, and the LEDs don't change.
#![no_main]
#![no_std]

use embedded_hal::{digital::{OutputPin, StatefulOutputPin}, delay::DelayNs};
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, i2c::{GlitchFilter, I2cConfig, I2cEvent, TransmissionMode}, pin_mapping::DefaultMapping, pmm::Pmm, prelude::*, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    let _wdt = Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut green_led = p1.pin0.to_output();
    let mut red_led = Batch::new(periph.p5).split(&pmm).pin1.to_output();
    let p3 = Batch::new(periph.p3).split(&pmm);

    // Slave, eUSCI_B1: P3.6 = UCB1SCL and P3.2 = UCB1SDA with P3SEL = 01 (SLASEO7C Table 9-25, p. 67)
    let sl_scl = p3.pin6.to_alternate1();
    let sl_sda = p3.pin2.to_alternate1();

    // Master, eUSCI_B0: P1.3 = UCB0SCL and P1.2 = UCB0SDA with P1SEL = 01 (SLASEO7C Table 9-23, p. 65).
    // The internal pullups are 20 kΩ to 50 kΩ (SLASEO7C 8.12.4.1, p. 31).
    let m_scl = p1.pin3.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let m_sda = p1.pin2.pullup().to_alternate1();

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
    let mut i2c_master = I2cConfig::<_, _, _, DefaultMapping>::new(periph.e_usci_b0, GlitchFilter::Max50ns)
        .as_single_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
        .configure(m_scl, m_sda);

    // The slave answers to its own address in UCBxI2COA0 (SLAU445I Table 24-11, p. 656)
    const SLAVE_ADDR: u8 = 0x1A;
    let mut i2c_slave = I2cConfig::<_, _, _, DefaultMapping>::new(periph.e_usci_b1, GlitchFilter::Max50ns)
        .as_slave(SLAVE_ADDR)
        .configure(sl_scl, sl_sda);

    loop {
        // The master sends a byte then receives a byte.
        // The slave echoes the master's byte back.

        // Master transmit
        // (The eUSCI_B slave "automatically acknowledges the received data": SLAU445I 24.3.5.1.2, p. 634)
        i2c_master.send_start(SLAVE_ADDR, TransmissionMode::Transmit);
        const ECHO_TX: u8 = 10;
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
        // (SLAU445I 24.3.5.2.2, p. 639)
        i2c_master.schedule_stop();
        let echo_rx = nb::block!(i2c_master.read_rx_buf()).unwrap_or(0); // Safe, slave doesn't send NACKs

        loop {
            if i2c_slave.poll() == Ok(I2cEvent::Stop) { break }
        }

        // Enable the LED if the echoed value matches what was sent.
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
