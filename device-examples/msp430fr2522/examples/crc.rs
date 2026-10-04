//! The CRC module and a software CRC compute the CRC-16-CCITT signature of the same 16 words, and an LED
//! on P1.0 turns on when the two signatures match.
//!
//! The CRC module starts from the seed FFFFh. The words go in through CRCDIRB, lower byte first, which
//! reverses the bits of each byte, and the signature is read from CRCINIRES. The software version works
//! through the same bytes in the same order.
//! (Polynomial x^16 + x^12 + x^5 + 1: SLASEE4C 6.10.6, p. 53. Seed, data and result registers: SLAU445I
//! 11.3, p. 354. Byte order and bit reversal: SLAU445I 11.3.1, p. 354. No board document covers the LED:
//! there is none for the MSP430FR25x2. P1.0 is a GPIO output, P1SELx = 00 and P1DIR = 1: SLASEE4C
//! Table 6-15, p. 58.)
//!
//! How to test (an LED and a resistor):
//! 1. Connect an LED with a series resistor (about 1 kΩ) from P1.0 to GND.
//! 2. Flash this example.
//! 3. Expected: the LED turns on and stays on: the signatures match. If it stays off, they differ.
//! 4. To see the other case, change the seed given to `calculate_software_sig` from 0xFFFF to 0, and
//!    flash again: the LED stays off.
#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use msp430_rt::entry;
use msp430_hal::{crc::Crc, gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr25x2::Peripherals::take().unwrap();
    // Halt the watchdog, which runs from every PUC (SLAU445I 12.2.2, p. 363)
    let _wdt = Wdt::constrain(periph.wdt_a);

    // Pmm::new clears LOCKLPM5, so the pins take on their configuration (SLAU445I 8.3.1, p. 316)
    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let mut led = p1.pin0.to_output();

    // Some random data to create a signature over. Can be modified.
    let crc_input = [
        0x0fc0, 0x1096, 0x5042, 0x0010, 0x7ff7, 0xf86a, 0xb58e, 0x7651, 0x8b88, 0x0679, 0x0123,
        0x9599, 0xc58c, 0xd1e2, 0xe144, 0xb691,
    ];

    // Configure the hardware CRC module, pass in the data and retrieve the signature. The module uses the
    // CRC-16-CCITT polynomial x^16 + x^12 + x^5 + 1 (SLASEE4C 6.10.6, p. 53).
    let mut crc_hw = Crc::new(periph.crc, 0xFFFF);
    crc_hw.add_words_lsb(&crc_input);
    let hw_sig = crc_hw.result();

    // Calculate the same CRC using a software implementation
    let sw_sig = calculate_software_sig(0xFFFF, &crc_input);

    // Turn on the LED if the signatures match
    led.set_state((sw_sig == hw_sig).into()).ok();

    loop {
        msp430::asm::nop();
    }
}

fn calculate_software_sig(seed: u16, data: &[u16]) -> u16 {
    let mut sig: u16 = seed;

    for val in data {
        let low_byte = (val & 0xFF) as u8;
        let high_byte = (val >> 8) as u8;
        ccitt_update(&mut sig, low_byte);
        ccitt_update(&mut sig, high_byte);
    }

    sig
}

// Software algorithm - CCITT CRC16 code. Derived from msp430fr235x_CRC.c, at:
// https://dev.ti.com/tirex/explore/node?node=A__AIgIaFR0j9SeqBvdp6wD2w__msp430ware__IOGqZri__LATEST
fn ccitt_update(sig: &mut u16, input: u8) {
    let mut new = *sig;
    new = new.rotate_right(8);
    new ^= input as u16;
    new ^= (new & 0xFF) >> 4;
    new ^= new << 12;
    new ^= (new & 0xFF) << 5;
    *sig = new;
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
