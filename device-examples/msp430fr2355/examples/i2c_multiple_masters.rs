//! Two I2C masters on one bus: ten times a second eUSCI_B0, a multi-master, writes a byte to eUSCI_B1
//! and reads it back. eUSCI_B1, a master that is also a slave, answers in its interrupt, and then, as
//! master, checks whether a device answers at address 0x09. LED1 lights when the byte came back, and
//! LED2 when no device answered at 0x09.
//!
//! When another master addresses it, eUSCI_B1 stops being a master and works as a slave; after the STOP
//! its interrupt makes it a master again. Its own address is 26 (0x1A). Both run at 100 kHz from SMCLK,
//! and eUSCI_B1 has the internal pull-ups on its pins.
//! (A master addressed as a slave "becomes a slave": SLAU445I Table 24-2, p. 646. The I2C pins of
//! eUSCI_B0 and eUSCI_B1: SLASEC4D Table 6-14, p. 72. LED1 on P1.0 is red and LED2 on P6.6 green:
//! SLAU680 Figure 18, p. 26.)
//!
//! How to test (two jumper wires):
//! 1. Connect SDA, P1.2 (J1 pin 10), to P4.6 (J2 pin 15), and SCL, P1.3 (J1 pin 9), to P4.7 (J2 pin 14).
//!    (Header pins: SLAU680 Figure 10, p. 15.)
//! 2. Flash this example.
//! 3. Expected: LED1 and LED2 light and stay on.
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// We use UnsafeCell here over RefCell to minimise binary size. Binary size suffers when panics are possible, as they pull in lots of
// strings and formatting. Debug builds from old compiler versions suffer in particular.
use core::cell::UnsafeCell;

use critical_section::Mutex;
use embedded_hal::{delay::DelayNs, digital::OutputPin, i2c::I2c};
use msp430::interrupt::enable as enable_interrupts;
use msp430_rt::entry;
use msp430fr2355::{interrupt, EUsciB1};
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, 
    i2c::{GlitchFilter, I2cConfig, I2cInterruptFlags as Flags, I2cMasterSlave, I2cVector}, pmm::Pmm, prelude::*, watchdog::Wdt
};
use panic_msp430 as _;

static I2C_MULTI_MASTER: Mutex<UnsafeCell<Option< I2cMasterSlave<EUsciB1> >>> = Mutex::new(UnsafeCell::new(None));
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
    // UCB1SCL on P4.7 and UCB1SDA on P4.6, P4SELx = 01 (SLASEC4D Table 6-66, p. 102). The internal
    // pull-ups are 20 to 50 kOhm (SLASEC4D Table 5-11, p. 43), and I2C needs pull-ups on SDA and SCL
    // (SLAU445I 24.3, p. 629).
    let ms_scl = p4.pin7.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let ms_sda = p4.pin6.pullup().to_alternate1();

    // UCB0SCL on P1.3 and UCB0SDA on P1.2, P1SELx = 01 (SLASEC4D Table 6-63, p. 96)
    let m_scl = p1.pin3.to_alternate1();
    let m_sda = p1.pin2.to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // Configure an I2C device as both master and slave. The device will automatically failover from master to slave when addressed.
    // (SLAU445I Table 24-2, p. 646: arbitration is lost "when the eUSCI_B operates as master but is
    // addressed as a slave by another master in the system", and then "the UCMST bit is cleared and
    // the I2C controller becomes a slave".)
    // Attempting any master actions will fail until the slave event has been handled.
    const MASTER_SLAVE_ADDR: u8 = 26;
    let mut i2c_master_slave = I2cConfig::new(periph.e_usci_b1, GlitchFilter::Max50ns)
        .as_master_slave(MASTER_SLAVE_ADDR)
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz (fBitClock = fBRCLK/UCBRx: SLAU445I 24.3.7, p. 642)
        .configure(ms_scl, ms_sda);

    // Make another I2C device to test the master-slave. Since there are now two masters present
    // on the bus this has to be a multi-master, rather than a single-master.
    let mut i2c_master = I2cConfig::new(periph.e_usci_b0, GlitchFilter::Max50ns)
        .as_multi_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz (fBitClock = fBRCLK/UCBRx: SLAU445I 24.3.7, p. 642)
        .configure(m_scl, m_sda);

    critical_section::with(|cs| {
        i2c_master_slave.set_interrupts(Flags::StartReceived);
        unsafe { *I2C_MULTI_MASTER.borrow(cs).get() = Some(i2c_master_slave) }
    });
    unsafe { enable_interrupts() };

    loop {
        // The master sends a byte then receives a byte from the master-slave.
        // The master-slave echoes the master's byte back.
        let mut echo_rx = [0; 1];
        const ECHO_TX: [u8; 1] = [128; 1];

        // Start a transaction using the master. This will force the master-slave into slave mode.
        // We don't care about errors for this example, but should be handled in a real implementation.
        let _ = i2c_master.write_read(MASTER_SLAVE_ADDR, &ECHO_TX, &mut echo_rx);

        // The master-slave is set back into master mode in the StopReceived interrupt, so now we can just use it like a master.
        // Here we check if a device with address 0x9 is on the bus (aka a zero-byte write)
        critical_section::with(|cs| {
            let Some(i2c_master_slave) = unsafe { &mut *I2C_MULTI_MASTER.borrow(cs).get() }.as_mut() else { return; };
            if let Ok(false) = i2c_master_slave.is_slave_present(9u8) {
                green_led.set_high().ok(); // Turn on the green LED if address 0x9 is not on bus
            }
        });

        // If the I2C devices echoed correctly set the red LED
        // (LED1, red, on P1.0 and LED2, green, on P6.6: SLAU680 Figure 18, p. 26)
        red_led.set_state((echo_rx == ECHO_TX).into()).ok();
        delay.delay_ms(100);
    }
}

// Static mut variables defined inside an interrupt handler are safe. See: https://docs.rust-embedded.org/book/start/interrupts.html
// The eUSCI_B1 receive or transmit vector at FFDEh (SLASEC4D Table 6-2, p. 64)
#[allow(static_mut_refs)]
#[interrupt]
fn EUSCI_B1() {
    static mut TEMP_VAR: u8 = 0;
    critical_section::with(|cs| {
        let Some(i2c_master_slave) = unsafe { &mut *I2C_MULTI_MASTER.borrow(cs).get() }.as_mut() else { return; };
        match i2c_master_slave.interrupt_source() {
            I2cVector::StartReceived => {
                // We have been addressed as a slave. Enable Rx, Tx and Stop interrupts.
                // (UCSTTIFG is set when the module "detects a START condition together with its own address":
                // SLAU445I Table 24-2, p. 646)
                i2c_master_slave.set_interrupts(Flags::TxBufEmpty | Flags::RxBufFull | Flags::StopReceived);
            }
            I2cVector::RxBufFull => {
                // Store the received value so we can echo it back later when the master switches to read mode
                // (UCRXIFG0 is set when a data byte is received: SLAU445I 24.3.5.1.2, p. 634)
                *TEMP_VAR = unsafe { i2c_master_slave.read_rx_buf_as_slave_unchecked() };
            }
            I2cVector::TxBufEmpty => {
                // Echo back the stored value
                // (When the master reads, "UCTR and UCTXIFG0 become set": SLAU445I 24.3.5.1.1, p. 633)
                unsafe { i2c_master_slave.write_tx_buf_as_slave_unchecked(*TEMP_VAR) };
            }
            I2cVector::StopReceived => {
                // Slave addressing concluded. Disable Rx, Tx, and Stop interrupts. We don't want these to trigger when acting as a master.
                // (UCSTPIFG "is set when the I2C module detects a STOP condition on the bus" and is "used in
                // slave and master mode": SLAU445I Table 24-2, p. 646)
                i2c_master_slave.clear_interrupts(Flags::TxBufEmpty | Flags::RxBufFull | Flags::StopReceived);
                i2c_master_slave.return_to_master();
            }
            _ => (), // unreachable
        }
    })
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
