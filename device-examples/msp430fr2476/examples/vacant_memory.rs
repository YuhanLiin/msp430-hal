//! The vacant memory access interrupt: reading an address where no memory exists requests the system
//! NMI. Each press of button S1 reads address 4000h, between the RAM and the FRAM, and the `SYSNMI`
//! handler reports the access. LED1 toggles for each access the handler sees, and the backchannel UART
//! prints what was read.
//! (Vacant memory: SLAU445I 1.9.2, p. 45. The MSP430FR2476 has RAM from 2000h to 3FFFh and FRAM from
//! 8000h: SLASEO7C Table 9-31, p. 73. S1 is P4.0 and LED1 on P1.0 is green: SLAU802 Figure 19, p. 25.)
//!
//! How to test:
//! 1. Flash this example, with the TXD jumper of J101 on (SLAU802 Table 2, p. 8).
//! 2. Open the COM port of "MSP Application UART1" at 9600 baud in a serial terminal such as PuTTY
//!    (SLAU802 2.2.4, p. 9).
//! 3. Press and release S1.
//!
//! Expected for each press: `Read 3FFF from 4000h, vacant memory accesses: 1` (then 2, 3, ...), and
//! LED1 toggles.
//! A read from vacant memory gives 3FFFh (SLAU445I 1.9.2, p. 45: "Reads from vacant memory result in
//! the value 3FFFh").
#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use embedded_hal::digital::*;
use embedded_io::Write;
use msp430_rt::entry;
use msp430_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pin_mapping::DefaultMapping,
    pmm::Pmm,
    serial::*,
    sys::{self, SysParts, SystemNmi},
    watchdog::Wdt,
};
use msp430_atomic::AtomicU16;
use msp430fr247x::interrupt;
use panic_msp430 as _;

/// An address between the RAM (2000h to 3FFFh) and the FRAM (from 8000h) (SLASEO7C Table 9-31, p. 73)
const VACANT_ADDRESS: usize = 0x4000;

/// Vacant memory accesses the `SYSNMI` handler has seen. The system NMI is non-maskable, so it can't
/// share data through a critical section, and this uses an atomic counter from `msp430-atomic`.
/// (NMIs "are not masked by the general interrupt enable (GIE) bit": SLAU445I 1.3.1, p. 33)
static VACANT_ACCESSES: AtomicU16 = AtomicU16::new(0);

#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.frctl);
    // Stop the watchdog (WDTHOLD = 1: SLAU445I Table 12-2, p. 366)
    Wdt::constrain(periph.wdt_a);

    let (pmm, _) = Pmm::new(periph.pmm, periph.sys);
    let p1 = Batch::new(periph.p1).split(&pmm);
    // S1 pulls P4.0 low, with the internal pullup on (PxDIR = 0, PxREN = 1, PxOUT = 1: SLAU445I
    // Table 8-1, p. 313) besides R9 (SLAU802 Figure 19, p. 25)
    let p4 = Batch::new(periph.p4)
        .config_pin0(|p| p.pullup())
        .split(&pmm);
    let mut led1 = p1.pin0.to_output_low();
    let mut s1 = p4.pin0;

    // MCLK = SMCLK = DCOCLKDIV in the 1 MHz range and ACLK from REFO (SELMS = 000b, SELA = 01b:
    // SLAU445I Table 3-8, p. 117; DIVM, DIVS: SLAU445I Table 3-9, p. 118)
    let (smclk, _aclk, _delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    // The backchannel UART: eUSCI_A0's TXD on P1.4, P1SEL = 01, 8N1 (SLAU802 2.2.4, p. 9; SLASEO7C
    // Table 9-23, p. 65; SLAU445I Table 22-8, p. 593)
    let mut tx = SerialConfig::<_, _, DefaultMapping>::new(
        periph.e_usci_a0,
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::OneStopBit,
        Parity::NoParity,
        Loopback::NoLoop,
        9600,
    )
    .use_smclk(&smclk)
    .tx_only(p1.pin4.to_alternate1());

    // VMAIE requests the system NMI on vacant memory accesses (SLAU445I 1.9.2, p. 45; VMAIE: SLAU445I
    // Table 1-9, p. 62)
    let mut sys = SysParts::new(periph.sfr);
    sys.vacant_memory.enable_interrupts();

    writeln!(tx, "\r\nPress S1 to read vacant memory\r").ok();
    let mut seen = 0;
    loop {
        if s1.is_low().unwrap() {
            // Wait for the release, and for the bouncing to stop
            while s1.is_low().unwrap() {}
            for _ in 0..10_000 {
                msp430::asm::nop();
            }
            let value = unsafe { core::ptr::read_volatile(VACANT_ADDRESS as *const u16) };
            write!(tx, "Read {:04X} from {:04X}h, ", value, VACANT_ADDRESS).ok();
        }

        // The NMI arrives a few instructions after the access (measured on an MSP430FR2476), so the
        // count is checked here rather than right after the read
        let accesses = VACANT_ACCESSES.load();
        if accesses != seen {
            seen = accesses;
            led1.toggle().ok();
            writeln!(tx, "vacant memory accesses: {}\r", accesses).ok();
        }
    }
}

// The system NMI vector: vacant memory accesses, the JTAG mailbox and FRAM bit errors (FFFCh:
// SLASEO7C Table 9-2, p. 46). Reading SYSSNIV returns the highest-priority source and clears its flag
// (SLAU445I 1.3.7, p. 36).
#[interrupt]
fn SYSNMI() {
    while let Some(nmi) = sys::take_system_nmi() {
        if nmi == SystemNmi::VacantMemoryAccess {
            VACANT_ACCESSES.add(1);
        }
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
