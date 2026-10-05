//! Low Power Mode (LPM) control
//!
//! The MSP430FR2x5x series supports several low power modes, namely LPM0, LPM3, LPM4, as well as
//! LPM3.5 and LPM4.5 (SLASEC4D Table 6-1, p. 61).
//! # LPM0
//! LPM0 turns off the CPU, while the rest of the system continues unimpeded. Entering LPM0 has no
//! special requirements (SLAU445I Table 1-2, p. 39). [`enter_lpm0_fram_off`] also powers the FRAM down
//! until the next interrupt (SLAU445I 6.8, p. 303).
//!
//! # LPM3
//! LPM3 turns off most high frequency clocks (FLL and DCO subsystems, MODCLK, etc.), most notably
//! SMCLK (SLASEC4D Table 6-1, p. 61).
//! Since most peripherals can be clocked by a low frequency clock source, this allows many peripherals
//! to continue operating at a reduced speed.
//!
//! LPM3 will only be entered if no peripherals have been configured to use SMCLK, otherwise LPM0 will
//! be entered instead (SLAU445I Table 1-3, p. 39).
//!
//! GPIO pins will maintain the value they had when LPM3 was entered (SLAU445I 1.4, p. 36).
//!
//! # LPM4
//! LPM4 turns off all clock sources (SLAU445I Table 1-2, p. 39). The RTC or Watchdog peripherals can
//! request very low power oscillators (VLOCLK or XTCLK) if needed, though this will increase power
//! consumption and is not considered 'true' LPM4 (SLASEC4D 6.2, p. 62: "XT1CLK and VLOCLK can be
//! active during LPM4 if requested by low-frequency peripherals"; SLAU445I 12.2.5, p. 364). Some
//! analog peripherals (eCOMP, SAC) continue to function in LPM4 (SLASEC4D Table 6-1, p. 62).
//! Wake-up events are limited to GPIO or RTC interrupts (SLASEC4D Table 6-1, p. 61 to p. 62).
//!
//! LPM4 will only be entered if no peripherals have been configured to use SMCLK or ACLK. If any
//! peripherals have requested SMCLK then LPM0 will be entered, otherwise if any peripherals have
//! requested ACLK then LPM3 will be entered (SLAU445I Table 1-3, p. 39).
//!
//! GPIO pins will maintain the value they had when LPM4 was entered (SLAU445I 1.4, p. 36).
//!
//! # Errata of LPM3 and LPM4
//! On the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2 the transition to LPM3 or LPM4 can lock up the
//! device, or make it run unintended code, when an interrupt is requested at the same time (SLAZ695J
//! CS13, p. 8, and PMM32, p. 9 to p. 10; SLAZ664S CS13, p. 9, and PMM32, p. 11 to p. 12; SLAZ705H CS13,
//! p. 8, and PMM32, p. 8 to p. 9). [`request_lpm3`] and [`request_lpm4`] apply TI's workarounds to the
//! entry they make, and on the MSP430FR2433 also the one for the bit errors that the FRAM can report
//! after a wake-up although it has none (SLAZ664S GC5, p. 10 to p. 11).
//!
//! The errata name entries "during ISR exits" too. An interrupt handler runs from FRAM, which powers the
//! FRAM up again (SLAU445I 6.8, p. 303), so a handler that returns to the sleep enters LPM3 or LPM4 again
//! with the FRAM active, without the workaround for PMM32. On those devices, let each interrupt handler
//! wake the CPU (`#[interrupt(wake_cpu)]`) and call `request_lpm3()` or `request_lpm4()` again, as
//! [`request_lpm3`] explains. No handler can run between the workaround and the start of the sleep:
//! the functions keep interrupts disabled for that part.
//!
//! # LPM3.5
//! LPM3.5 is an extension of LPM4 that also disables the RAM and analog peripherals. Unlike in LPM4,
//! the very low power oscillators (VLOCLK or XTCLK) are expected to be used in LPM3.5 (SLASEC4D
//! Table 6-1, p. 61 to p. 62).
//! Because the RAM is unpowered, all internal state is lost and when the MCU wakes up execution will
//! restart from the beginning of the program (SLAU445I 1.4.3, p. 40). The 32-byte region of Backup
//! Memory is powered through LPM3.5, which can be used to maintain some state between iterations
//! (SLASEC4D Table 6-1, p. 62).
//!
//! Unlike with LPM3 and 4, a peripheral requesting a high-speed clock source like SMCLK will not stop
//! LPM3.5 from being entered (SLAU445I Table 3-2, p. 109).
//!
//! During LPM3.5 GPIO pins will maintain the value they had when LPM3.5 was entered but the register
//! contents are lost, so after a wake-up the GPIO pins will take on their reset values when LOCKLPM5
//! is cleared (SLAU445I 8.3.3, p. 318).
//!
//! # LPM4.5
//! LPM4.5 is an extension of LPM3.5 that also disables the very low power oscillators, backup memory,
//! and RTC (SLASEC4D Table 6-1, p. 61 to p. 62). Barely anything is powered in this mode. The only
//! methods to wake from LPM4.5 are a GPIO interrupt, the reset pin, or a power cycle (SLAU445I 1.4,
//! p. 37).
//! Like LPM3.5, all internal state is lost and program execution restarts from the beginning when a
//! wake-up occurs. Unlike LPM3.5, the backup memory is unpowered, so can't be used to store state.
//! The non-volatile Information Memory (`INFO_MEM_SIZE` bytes) can however be used to store data
//! while the MCU is in active mode, and can be read back after a wake-up event.
//!
//! Unlike with LPM3 and 4, a peripheral requesting a clock source like SMCLK or ACLK will not stop
//! LPM4.5 from being entered (SLAU445I Table 3-2, p. 109; the HAL clears ACLKREQEN, as SLAU445I
//! 3.2.12.1, p. 109 requires).
//!
//! During LPM4.5 GPIO pins will maintain the value they had when LPM4.5 was entered but the register
//! contents are lost, so after a wake-up the GPIO pins will take on their reset values when LOCKLPM5
//! is cleared (SLAU445I 8.3.3, p. 318).

use crate::_pac;
use core::any::TypeId;
use core::arch::asm;

use crate::{
    clock::{Xt1Xin, Xt1Xout},
    device_specific::lpm::reset_all_pin_functions,
    gpio::{AlternatePin, PortNum},
    rtc::{Rtc, RtcLpm3_5ClockSrc},
    watchdog::{WatchdogSelect, Wdt},
};

/// Whether the high-side supply voltage supervisor (SVSH) stays on in the low-power modes (SVSHE,
/// SLAU445I Table 2-2, p. 91). `Disabled` switches SVSH off in LPM2, LPM3, LPM4, LPM3.5 and LPM4.5,
/// which saves power (SLAU445I 2.2.4, p. 87); it stays on in active mode, LPM0 and LPM1. `Enabled`
/// keeps it on always.
pub use crate::_pac::pmm::pmmctl0::Svshe as SvsState;

// Status register (SLAU445I Figure 4-9, p. 130):
// SCG1 SCG0 OSC_OFF CPU_OFF GIE N Z C
// 7    6    5       4       3   2 1 0
// The low-power modes set these bits as in SLAU445I Table 1-2, p. 39.
const SCG1:    u8 = 1 << 7;
const SCG0:    u8 = 1 << 6;
const OSC_OFF: u8 = 1 << 5;
const CPU_OFF: u8 = 1 << 4;
const GIE:     u8 = 1 << 3;

/// For each set bit in the bitmask, set the corresponding bit in the status register (SLAU445I
/// Figure 4-9, p. 130; BIS changes SR bits: SLAU445I 4.3.3, p. 130).
#[inline(always)]
fn set_sr_bits<const MASK: u8>() {
    unsafe { asm!("bis.b #{mask}, SR", mask = const MASK, options(nomem, nostack)) };
}

/// Enter a low-power mode by setting the status register bits in `MASK`. Unlike `set_sr_bits` it
/// is a compiler barrier, so memory accesses aren't moved across the sleep. With GIE in the mask,
/// interrupts are enabled in the same instruction that starts the sleep, followed by the NOP TI
/// recommends after enabling interrupts (SLAU445I 1.3.4.1, p. 34: "It is recommended to always
/// insert at least one instruction between EINT and DINT").
#[inline(always)]
fn sleep<const MASK: u8>() {
    unsafe { asm!("bis.b #{mask}, SR", "nop", mask = const MASK, options(nostack)) };
}

/// Request LPM3 or LPM4, the status register bits in `MASK`, as the `request_lpm*` functions do.
///
/// On the MSP430FR2x5x, "HFXT must be disabled before entering into LPM3, LPM4, or LPMx.5 mode"
/// (SLASEC4D Table 6-1, note 2, p. 61). While XT1 runs in high-frequency mode, LPM0 is entered
/// instead, which HFXT may stay on in (SLASEC4D Table 6-1, p. 61), as the device itself does when a
/// peripheral still needs SMCLK (SLAU445I Table 1-3, p. 39).
///
/// On the devices with errata CS13 and PMM32 the entry works around them, see
/// [`lpm3_4_with_workarounds`] (SLAZ695J CS13, p. 8 to p. 9, and PMM32, p. 9 to p. 11; SLAZ664S CS13,
/// p. 9 to p. 10, and PMM32, p. 11 to p. 12; SLAZ705H CS13, p. 7 to p. 8, and PMM32, p. 8 to p. 10).
#[inline(always)]
fn request_lpm<const MASK: u8>() {
    #[cfg(feature = "xt1_high_frequency")]
    if hfxt_in_use() {
        if MASK & GIE != 0 {
            sleep::<{ CPU_OFF | GIE }>();
        } else {
            sleep::<CPU_OFF>();
        }
        return;
    }
    #[cfg(any(feature = "erratum_cs13", feature = "erratum_pmm32"))]
    lpm3_4_with_workarounds(MASK);
    #[cfg(not(any(feature = "erratum_cs13", feature = "erratum_pmm32")))]
    sleep::<MASK>();
}

/// Whether XT1 runs in high-frequency mode: XTS = 1 (SLAU445I Table 3-10, p. 119), with XIN in its
/// XT1 function (SLAU445I 3.2.4, p. 103)
#[cfg(feature = "xt1_high_frequency")]
#[inline(always)]
fn hfxt_in_use() -> bool {
    let cs = unsafe { _pac::Cs::steal() };
    cs.csctl6().read().xts().bit_is_set() && Xt1Xin::<()>::function_matches_type()
}

/// Enter LPM3 or LPM4 (the status register bits in `mask`) with the workarounds for the errata that
/// can lock up the device on the way in, on the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2:
///
/// - CS13, a lock-up when an interrupt arrives during the entry with the DCO above 2 MHz. Workaround
///   4: "Set DCOCLK to 2MHz or lower before entering LPM3/4, then restore DCOCLK after wake-up"
///   (SLAZ695J CS13, p. 9; SLAZ664S CS13, p. 9 to p. 10; SLAZ705H CS13, p. 8). Interrupt handlers run
///   on the slowed DCO, with the FLL off until a handler that wakes the CPU clears SCG0, and so do
///   peripherals clocked from it if one keeps the device in LPM0 (the erratum: "peripherals using
///   clocks derived from DCOCLK might be affected during this interval").
/// - PMM32, a lock-up or unintended code execution when an interrupt coincides with the entry and with
///   a MODCLK request or removal, or with SMCLK at another frequency than MCLK while neither MODCLK nor
///   SMCLK runs (SLAZ695J PMM32, p. 9 to p. 11; SLAZ664S PMM32, p. 11 to p. 12; SLAZ705H PMM32, p. 8 to
///   p. 10). Workaround 2: "Place the FRAM in INACTIVE mode before any entry to LPM3/4 by clearing the
///   FRPWR bit and FRLPMPWR bit (if exist) in the GCCTL0 register. This must be performed from RAM". It
///   covers the entry made here, not an interrupt handler's return to the sleep: see [`request_lpm3`].
/// - GC5 on the MSP430FR2433, bit errors reported after a wake-up although the FRAM has none.
///   Workaround 1 clears UBDRSTEN, UBDIE and CBDIE before the entry. Workaround 2 clears UBDIFG and
///   CBDIFG after the wake-up and sets the three bits again "after the first valid FRAM access has been
///   completed" (SLAZ664S GC5, p. 11), which [`valid_fram_access`] makes sure of.
#[cfg(any(feature = "erratum_cs13", feature = "erratum_pmm32"))]
#[inline(never)]
fn lpm3_4_with_workarounds(mask: u8) {
    #[cfg(feature = "erratum_cs13")]
    let saved_dco = lower_dco();

    // GC5, workaround 1: switch the bit error handling off before the sleep (UBDRSTEN, UBDIE and CBDIE
    // in GCCTL0, SLAU445I Table 6-3, p. 307), unless it is off already
    #[cfg(feature = "erratum_gc5")]
    let bit_error_handling = {
        let saved = unsafe { _pac::Frctl::steal() }.gcctl0().read();
        if saved.ubdrsten().bit() || saved.ubdie().bit() || saved.cbdie().bit() {
            fram_unlocked(|fram| unsafe {
                fram.gcctl0().clear_bits(|w| w
                    .ubdrsten().clear_bit()
                    .ubdie().clear_bit()
                    .cbdie().clear_bit())
            });
        }
        saved
    };

    #[cfg(feature = "erratum_pmm32")]
    {
        // FRPWR and FRLPMPWR, GCCTL0 bits 2 and 1 (SLAU445I Table 6-3, p. 307). They're cleared in the
        // routine that runs from RAM, which can't call the PAC, so they're passed as a mask.
        const GCCTL0_FRPWR_FRLPMPWR: u16 = 1 << 2 | 1 << 1;
        // An interrupt handler that ran between the FRAM switch-off and the sleep instruction would power
        // the FRAM up again (SLAU445I 6.8, p. 303), and the sleep would start with the FRAM active. With
        // interrupts enabled, they're disabled for the routine (DINT followed by a NOP, as SLAU445I
        // 4.6.2.19, p. 189 asks of "any code sequence" that "needs to be protected from interruption"),
        // and the instruction that starts the sleep enables them again, as `request_lpm3_with_interrupts`
        // does (GIE: SLAU445I Figure 4-9, p. 130).
        let mask = if msp430::register::sr::read().gie() {
            msp430::interrupt::disable();
            mask | GIE
        } else {
            mask
        };
        unsafe { sleep_from_ram(mask as u16, _pac::Frctl::ptr() as *mut u16, GCCTL0_FRPWR_FRLPMPWR) };
    }
    #[cfg(not(feature = "erratum_pmm32"))]
    unsafe { asm!("bis {mask}, SR", "nop", mask = in(reg) mask as u16, options(nostack)) };

    // The DCO comes back first, so that the rest runs at full speed, and so that the FLL, which a handler
    // that wakes the CPU switches on again by clearing SCG0, runs on the slowed DCO for a few instructions
    // only
    #[cfg(feature = "erratum_cs13")]
    restore_dco(saved_dco);

    // GC5, workaround 2: "After LPM wake up, clear GCCTL1.UBDIFG and GCCTL1.CBDIFG, and then reinitialize
    // the GCCTL0 register after the first valid FRAM access has been completed" (SLAZ664S GC5, p. 11). The
    // flags are cleared after every wake-up, as the workaround says, also when the bit error handling is
    // off and GCCTL0 stays as it is. They are cleared after `valid_fram_access()`, so that an error flagged
    // up to that access is cleared as well.
    #[cfg(feature = "erratum_gc5")]
    {
        valid_fram_access();
        let saved = bit_error_handling;
        fram_unlocked(|fram| unsafe {
            // UBDIFG and CBDIFG are cleared by writing 0 (SLAU445I Table 6-4, p. 308)
            fram.gcctl1().clear_bits(|w| w.ubdifg().clear_bit().cbdifg().clear_bit());
            fram.gcctl0().set_bits(|w| w
                .ubdrsten().bit(saved.ubdrsten().bit())
                .ubdie().bit(saved.ubdie().bit())
                .cbdie().bit(saved.cbdie().bit()));
        });
    }
}

/// The code of workaround 2 of erratum PMM32, run from RAM (SLAZ695J PMM32, p. 10 to p. 11; SLAZ664S
/// PMM32, p. 12; SLAZ705H PMM32, p. 9 to p. 10): unlock FRCTL, clear `gcctl0_clear` in GCCTL0 (FRPWR and
/// FRLPMPWR), lock FRCTL, and set the status register bits in `sr_bits`. LPM0 with the FRAM powered
/// down uses it too, with FRPWR alone, see [`enter_lpm0_fram_off`]. The
/// erratum's code writes `FRCTL0 = FRCTLPW`, which would also clear NWAITS; this writes the password
/// over the current low byte instead (FRCTLPW, NWAITS: SLAU445I Table 6-2, p. 306). A byte write of a
/// wrong password to the upper byte locks FRCTL again (SLAU445I 6.10, p. 305). An access to the FRAM
/// after the wake-up powers it up again ("Memory accesses pointing into the FRAM address space
/// automatically set FRPWR = 1", SLAU445I 6.8, p. 303).
///
/// The `.data` section is copied to RAM at start-up (msp430-rt's link.x), so the function runs from
/// RAM. FRCTL0 is at offset 00h and GCCTL0 at 04h of the FRCTL registers (SLAU445I Table 6-1, p. 305).
///
/// It gets a symbol name of its own, as code outside the program could call it, so that the compiler
/// leaves its arguments to the callers. A program with one call passing constants otherwise had the
/// constants loaded in the function itself, 12 more bytes of RAM. The name holds the HAL's version, so
/// two versions of the HAL in one program don't clash, and the linker still leaves the function out of
/// programs that don't call it.
#[link_section = ".data.lpm_from_ram"]
#[inline(never)]
#[export_name = concat!(
    "__msp430_hal_",
    env!("CARGO_PKG_VERSION_MAJOR"), "_", env!("CARGO_PKG_VERSION_MINOR"), "_", env!("CARGO_PKG_VERSION_PATCH"),
    "_lpm_sleep_from_ram"
)]
unsafe extern "C" fn sleep_from_ram(sr_bits: u16, frctl: *mut u16, gcctl0_clear: u16) {
    asm!(
        "mov.b 0({frctl}), {tmp}",
        "bis #0xA500, {tmp}",
        "mov {tmp}, 0({frctl})",
        "bic {clear}, 4({frctl})",
        "mov.b #0, 1({frctl})",
        "bis {sr}, SR",
        "nop",
        frctl = in(reg) frctl,
        clear = in(reg) gcctl0_clear,
        sr = in(reg) sr_bits,
        tmp = out(reg) _,
        options(nostack),
    );
}

/// Run `f` with write access to the FRAM controller registers, as `Fram` does (SLAU445I 6.10,
/// p. 305; FRCTLPW: SLAU445I Table 6-2, p. 306)
#[cfg(feature = "erratum_gc5")]
fn fram_unlocked(f: impl FnOnce(&_pac::Frctl)) {
    let fram = unsafe { _pac::Frctl::steal() };
    critical_section::with(|_| {
        fram.frctl0().modify(|_, w| w.frctlpw().password());
        f(&fram);
        fram.frctl0_h().write(|w| w.frctlpw().lock());
    });
}

/// Read the FRAM itself, not only its cache, for workaround 2 of erratum GC5: "For the valid FRAM access
/// the user has to consider possible cache hits which depends on implementation" (SLAZ664S GC5, p. 11).
///
/// The cache serves a read without the FRAM: "If one of the four words stored in one of the cache lines
/// is requested (a cache hit), no FRAM access occurs" (SLAU445I 6.5.1, p. 303), and "Accesses to FRAM that
/// can be served from cache do not change the power state of the FRAM power control" (SLAU445I 6.8,
/// p. 303). It is "a 2-way associative cache with 4 cache lines of 64 bits each" (SLAU445I 6.9, p. 304),
/// four words each. Five words 8 bytes apart lie in five different lines, one more than the cache holds,
/// so at least one of the five reads below reads the FRAM. They read five vectors of the interrupt vector
/// table, from the reset vector at FFFEh down (SLASE59F 6.4, p. 41; SLASE59F Table 6-2, p. 41 to p. 42).
/// The reads are volatile, so they stay before the register writes that follow, which set GCCTL0 again.
#[cfg(feature = "erratum_gc5")]
#[inline(always)]
fn valid_fram_access() {
    // The reset vector, and the vectors 8, 16, 24 and 32 bytes below it
    unsafe {
        core::ptr::read_volatile(0xFFFE as *const u16);
        core::ptr::read_volatile(0xFFF6 as *const u16);
        core::ptr::read_volatile(0xFFEE as *const u16);
        core::ptr::read_volatile(0xFFE6 as *const u16);
        core::ptr::read_volatile(0xFFDE as *const u16);
    }
}

/// CSCTL0 and CSCTL1 before the DCO was slowed for erratum CS13, and whether the FLL was on
#[cfg(feature = "erratum_cs13")]
struct SavedDco {
    csctl0: u16,
    csctl1: u16,
    fll_was_on: bool,
}

/// Bring the DCO to 2 MHz or lower for erratum CS13 (SLAZ695J CS13, p. 9; SLAZ664S CS13, p. 9 to p. 10;
/// SLAZ705H CS13, p. 8), if it may run faster. In the lowest range, DCORSEL = 000b (SLAU445I Table 3-5,
/// p. 114), with DCOFTRIM = 000b the DCO runs at 0.85 MHz to 0.90 MHz at its highest tap (SLASEC4D
/// Table 5-6, p. 38; SLASE59F Table 5-6, p. 25; SLASEE4C Table 5-6, p. 27). In that range already,
/// the FLL keeps it near 1 MHz, and nothing changes.
#[cfg(feature = "erratum_cs13")]
#[inline(always)]
fn lower_dco() -> Option<SavedDco> {
    let cs = unsafe { _pac::Cs::steal() };
    let csctl1 = cs.csctl1().read();
    if csctl1.dcorsel().is_range_1mhz() {
        return None;
    }
    let saved = SavedDco {
        csctl0: cs.csctl0().read().bits(),
        csctl1: csctl1.bits(),
        fll_was_on: !msp430::register::sr::read().scg0(),
    };
    // Switch the FLL off first, so it can't move the DCO tap while the range is low (SLAU445I 3.2.8,
    // p. 105: "The FLL is disabled when the status register bits SCG0 or SCG1 are set"). It stays off
    // in interrupt handlers: an interrupt "does not clear SCG0" (SLAU445I 3.2.10, p. 106).
    set_sr_bits::<SCG0>();
    // DCOFTRIMEN = 1, DCOFTRIM = 000b, DCORSEL = 000b, DISMOD kept (CSCTL1, SLAU445I Table 3-5, p. 114)
    cs.csctl1().write(|w| w
        .dismod().bit(csctl1.dismod().bit())
        .dcoftrimen().set_bit()
        .dcoftrim().set(0)
        .dcorsel().range_1mhz());
    Some(saved)
}

/// Undo [`lower_dco`]: with the FLL off, put back the DCO tap (CSCTL0, SLAU445I Table 3-4, p. 113),
/// which is harmless in the lowest range, then the range and trim (CSCTL1, SLAU445I Table 3-5,
/// p. 114), which brings the DCO straight back to the frequency it had, and then the FLL's state
/// (SCG0, SLAU445I Figure 4-9, p. 130).
#[cfg(feature = "erratum_cs13")]
#[inline(always)]
fn restore_dco(saved: Option<SavedDco>) {
    if let Some(saved) = saved {
        let cs = unsafe { _pac::Cs::steal() };
        set_sr_bits::<SCG0>();
        cs.csctl0().write(|w| unsafe { w.bits(saved.csctl0) });
        cs.csctl1().write(|w| unsafe { w.bits(saved.csctl1) });
        if saved.fll_was_on {
            unsafe { asm!("bic #{scg0}, SR", scg0 = const SCG0 as u16, options(nomem, nostack)) };
        }
    }
}

/// Enter Low Power Mode 0 (LPM0).
///
/// In LPM0 the CPU and MCLK are disabled (SLAU445I Table 1-2, p. 39).
///
/// Power draw in LPM0: Approx 40 uA / MHz (MSP430FR2355: SLASEC4D Table 6-1, p. 61).
///
/// Only an interrupt wakes the CPU (SLAU445I 1.4, p. 36), so interrupts must be enabled already.
/// To check a condition and then sleep without missing an interrupt that arrives in between, check
/// it with interrupts disabled and use [`enter_lpm0_with_interrupts`] instead.
#[inline(always)]
pub fn enter_lpm0() {
    const LPM0: u8 = CPU_OFF;
    sleep::<LPM0>();
}

/// Enable interrupts and enter Low Power Mode 0 (LPM0) in one instruction, as the user's guide
/// does (`BIS #GIE+CPUOFF,SR`, SLAU445I 1.4.2, p. 40). An interrupt that is already pending is
/// taken right after (SLAU445I 1.3.4, p. 33), so it can't be missed between checking a condition
/// and going to sleep.
#[inline(always)]
pub fn enter_lpm0_with_interrupts() {
    const LPM0: u8 = CPU_OFF | GIE;
    sleep::<LPM0>();
}

/// FRPWR, GCCTL0 bit 2 (SLAU445I Table 6-3, p. 307), for the routine that runs from RAM
const GCCTL0_FRPWR: u16 = 1 << 2;

/// Enter Low Power Mode 0 (LPM0) with the FRAM powered down (GCCTL0.FRPWR = 0: SLAU445I 6.8, p. 303;
/// SLAU445I Table 6-3, p. 307), which saves the FRAM's supply current while the CPU sleeps.
///
/// "For LPM0, the FRAM power state during LPM0 is saved from the previous state in active mode", and
/// "Memory accesses pointing into the FRAM address space automatically set FRPWR = 1" (SLAU445I 6.8,
/// p. 303), so the FRAM is switched off from RAM, right before the sleep, by the routine the workaround
/// for erratum PMM32 uses (SLAZ695J PMM32, p. 10 to p. 11; SLAZ664S PMM32, p. 12; SLAZ705H PMM32, p. 9 to
/// p. 10).
///
/// The first interrupt handler powers the FRAM up again, as it runs from FRAM: "If FRAM power is disabled,
/// any memory access automatically inserts wait states to ensure sufficient time for the FRAM power up and
/// access" (SLAU445I 6.8, p. 303). If the handler leaves the CPU in LPM0, the FRAM stays on for the rest of
/// it, so return from the sleep and call this again to have it off again.
///
/// Interrupts must be enabled already; see [`enter_lpm0_fram_off_with_interrupts`].
#[inline(always)]
pub fn enter_lpm0_fram_off() {
    const LPM0: u8 = CPU_OFF;
    unsafe { sleep_from_ram(LPM0 as u16, _pac::Frctl::ptr() as *mut u16, GCCTL0_FRPWR) };
}

/// Enable interrupts and enter Low Power Mode 0 (LPM0) with the FRAM powered down, in one instruction,
/// like [`enter_lpm0_with_interrupts`]. See [`enter_lpm0_fram_off`].
#[inline(always)]
pub fn enter_lpm0_fram_off_with_interrupts() {
    const LPM0: u8 = CPU_OFF | GIE;
    unsafe { sleep_from_ram(LPM0 as u16, _pac::Frctl::ptr() as *mut u16, GCCTL0_FRPWR) };
}

/// Request Low Power Mode 3 (LPM3).
///
/// LPM3 can only be reached if no peripherals have been configured to use SMCLK.
/// If any peripherals are configured to use SMCLK then LPM0 will be entered instead (SLAU445I
/// Table 1-3, p. 39).
///
/// In LPM3 the CPU, FLL, and all clocks (except ACLK) are disabled (SLAU445I Table 1-2, p. 39).
///
/// Power draw in LPM3: Approx 1.4 uA (MSP430FR2355 with the RTC counter on XT1: SLASEC4D
/// Table 6-1, p. 61).
///
/// Interrupts must be enabled already; see [`request_lpm3_with_interrupts`].
///
/// On the MSP430FR2x5x with XT1 in high-frequency mode this enters LPM0 instead, because "HFXT
/// must be disabled before entering into LPM3, LPM4, or LPMx.5 mode" (SLASEC4D Table 6-1, note 2,
/// p. 61).
///
/// Errata: on the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2 the transition to LPM3 or LPM4 can lock
/// up the device when an interrupt is requested at the same time (SLAZ695J CS13, p. 8 to p. 9, and
/// PMM32, p. 9 to p. 11; SLAZ664S CS13, p. 9 to p. 10, and PMM32, p. 11 to p. 12; SLAZ705H CS13, p. 7
/// to p. 8, and PMM32, p. 8 to p. 10). On those devices the entry applies TI's workarounds: the FRAM is
/// switched to INACTIVE from RAM, and a DCO above 2 MHz is slowed to below 1 MHz until the CPU carries
/// on in this function after the sleep. On the MSP430FR2433 the FRAM bit error handling is also paused
/// around the sleep (SLAZ664S GC5, p. 10 to p. 11).
///
/// While the DCO is slowed, interrupt handlers run slower, and so do peripherals clocked from the DCO,
/// as erratum CS13 warns: "peripherals using clocks derived from DCOCLK might be affected during this
/// interval". One that keeps SMCLK running, such as a UART that is still sending (SLAU445I 22.3.14,
/// p. 590), makes the device enter LPM0 instead (SLAU445I Table 1-3, p. 39) and runs on the slowed clock
/// for the whole sleep. Handlers run with the FLL off, as an interrupt "does not clear SCG0" (SLAU445I
/// 3.2.10, p. 106). `#[interrupt(wake_cpu)]` clears it with the other low-power bits, so the FLL runs on
/// the slowed DCO from such a handler's return until this function restores the DCO, a few instructions
/// later. FLLUNLOCK can report the DCO as too slow meanwhile (SLAU445I 3.2.9, p. 105), which
/// [`fll_unlock_history`] then keeps, and which requests the NMI after [`enable_fll_unlock_interrupt`]
/// (FLLUNLOCKHIS, FLLWARNEN: SLAU445I Table 3-11, p. 121).
///
/// The FRAM workaround covers only the entry this function makes. PMM32 also names entries "during
/// ISR exits" (SLAZ695J PMM32, p. 9; SLAZ664S PMM32, p. 11; SLAZ705H PMM32, p. 9), and an interrupt
/// handler, which runs from FRAM, powers the FRAM up again (SLAU445I 6.8, p. 303), so a handler that
/// returns to the sleep enters LPM3 or LPM4 again with the FRAM active. On those devices, let each
/// handler wake the CPU (`#[interrupt(wake_cpu)]`) and call this function again. A handler that ran
/// after this function has switched the FRAM off, before the sleep has started, would power it up in
/// the same way, so the function disables interrupts for that part, and the instruction that starts
/// the sleep enables them again, as [`request_lpm3_with_interrupts`] does (SLAU445I 4.6.2.19,
/// p. 189).
///
/// [`fll_unlock_history`]: crate::clock::fll_unlock_history
/// [`enable_fll_unlock_interrupt`]: crate::clock::enable_fll_unlock_interrupt
#[inline(always)]
pub fn request_lpm3() {
    const LPM3: u8 = SCG1 | SCG0 | CPU_OFF;
    request_lpm::<LPM3>();
}

/// Enable interrupts and request Low Power Mode 3 (LPM3) in one instruction, like
/// [`enter_lpm0_with_interrupts`].
///
/// High-frequency XT1, and the errata workarounds with what they need from interrupt handlers: see
/// [`request_lpm3`] (SLASEC4D Table 6-1, note 2, p. 61; SLAZ695J CS13, p. 8 to p. 9, and PMM32, p. 9 to
/// p. 11; SLAZ664S CS13, p. 9 to p. 10, PMM32, p. 11 to p. 12, and GC5, p. 10 to p. 11; SLAZ705H CS13,
/// p. 7 to p. 8, and PMM32, p. 8 to p. 10).
/// Called with interrupts disabled, it lets no interrupt handler run between the FRAM workaround of
/// erratum PMM32 and the sleep.
#[inline(always)]
pub fn request_lpm3_with_interrupts() {
    const LPM3: u8 = SCG1 | SCG0 | CPU_OFF | GIE;
    request_lpm::<LPM3>();
}

/// Request Low Power Mode 4 (LPM4).
///
/// LPM4 can only be reached if no peripherals have been configured to use SMCLK or ACLK.
///
/// If any peripherals use SMCLK then LPM0 will be entered.
/// If no peripherals use SMCLK but at least one uses ACLK then LPM3 will be entered (SLAU445I
/// Table 1-3, p. 39).
///
/// In LPM4 the CPU, FLL, and all clocks (except optionally the very low power oscillators VLOCLK or
/// XTCLK) are disabled (SLAU445I Table 1-2, p. 39; SLASEC4D 6.2, p. 62).
///
/// Power draw in LPM4: Approx 820 nA (MSP430FR2355 without SVS: SLASEC4D Table 6-1, p. 61).
///
/// Interrupts must be enabled already; see [`request_lpm4_with_interrupts`].
///
/// High-frequency XT1, and the errata workarounds with what they need from interrupt handlers: see
/// [`request_lpm3`] (SLASEC4D Table 6-1, note 2, p. 61; SLAZ695J CS13, p. 8 to p. 9, and PMM32, p. 9 to
/// p. 11; SLAZ664S CS13, p. 9 to p. 10, PMM32, p. 11 to p. 12, and GC5, p. 10 to p. 11; SLAZ705H CS13,
/// p. 7 to p. 8, and PMM32, p. 8 to p. 10).
#[inline(always)]
pub fn request_lpm4() {
    const LPM4: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF;
    request_lpm::<LPM4>();
}

/// Enable interrupts and request Low Power Mode 4 (LPM4) in one instruction, like
/// [`enter_lpm0_with_interrupts`].
///
/// High-frequency XT1, and the errata workarounds with what they need from interrupt handlers: see
/// [`request_lpm3`] (SLASEC4D Table 6-1, note 2, p. 61; SLAZ695J CS13, p. 8 to p. 9, and PMM32, p. 9 to
/// p. 11; SLAZ664S CS13, p. 9 to p. 10, PMM32, p. 11 to p. 12, and GC5, p. 10 to p. 11; SLAZ705H CS13,
/// p. 7 to p. 8, and PMM32, p. 8 to p. 10).
/// Called with interrupts disabled, it lets no interrupt handler run between the FRAM workaround of
/// erratum PMM32 and the sleep.
#[inline(always)]
pub fn request_lpm4_with_interrupts() {
    const LPM4: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF | GIE;
    request_lpm::<LPM4>();
}

/// Enter Low Power Mode 3.5 (LPM3.5).
///
/// In LPM3.5 everything except the backup memory, the RTC and its clock (VLOCLK or XT1) are
/// disabled (SLASEC4D Table 6-1, p. 61 to p. 62; SLAU445I 15.2.2, p. 417). The only enabled
/// interrupts are from the RTC, I/O pins, an LF crystal fault, the RST pin, or a power cycle
/// (SLAU445I 1.4, p. 37).
///
/// I/O pins have their state latched while in LPM3.5, but the IO register values are reset on
/// wake-up (SLAU445I 8.3.3, p. 318).
/// If XT1 clocks the RTC, use [`Pmm::new_locked`](crate::pmm::Pmm::new_locked) after the wake-up
/// to keep it running (SLAU445I 1.4.3.3, p. 42). Only a low-frequency XT1 keeps running: on the
/// MSP430FR2x5x, a high-frequency XT1 is switched off, as "HFXT must be disabled before entering
/// into LPM3, LPM4, or LPMx.5 mode" (SLASEC4D Table 6-1, note 2, p. 61).
///
/// **Waking up from LPM3.5 requires a full system reset** (SLAU445I 1.4.3.2, p. 42: "Any exit
/// from LPMx.5 causes a BOR").
///
/// Power draw in LPM3.5: Approx 620 nA (MSP430FR2355 with the RTC counter on XT1: SLASEC4D
/// Table 6-1, p. 61).
#[inline(always)]
pub fn enter_lpm3_5<MODE: WatchdogSelect, SRC: RtcLpm3_5ClockSrc>(
    wdt: Wdt<MODE>,
    _rtc: Rtc<SRC>,
    svs: SvsState,
) -> ! {
    lpm3_5(wdt, svs);
}

/// Enter LPM3.5 without providing a correctly configured RTC (because it has already been configured in a prior iteration).
/// # Safety
/// If the RTC was not correctly configured previously then the system will not enter LPM3.5
/// (SLAU445I 1.4.3.1, p. 41: "The device enters LPM4.5 if none of the modules that are connected to
/// the RTC LDO are enabled").
#[inline(always)]
pub unsafe fn enter_lpm3_5_unchecked<MODE: WatchdogSelect>(wdt: Wdt<MODE>, svs: SvsState) -> ! {
    lpm3_5(wdt, svs)
}

fn lpm3_5<MODE: WatchdogSelect>(wdt: Wdt<MODE>, svs: SvsState) -> ! {
    // Every pin returns to GPIO, except the XT1 pins while XT1 is in use, so it can keep
    // clocking the RTC (SLAU445I 1.4.3.1, p. 41, step 2)
    reset_all_pin_functions(KeepXt1Pins::in_use());
    enter_lpmx_5(wdt, svs)
}

/// Enter Low Power Mode 4.5 (LPM4.5).
///
/// In LPM4.5 *everything* is disabled (SLAU445I Table 1-2, p. 39). The only available interrupt
/// sources are from I/O pins, the RST pin, or a power cycle (SLAU445I 1.4, p. 37).
///
/// I/O pins have their state latched while in LPM4.5, but the register values are reset on
/// wake-up (SLAU445I 8.3.3, p. 318).
///
/// **Waking up from LPM4.5 requires a full system reset** (SLAU445I 1.4.3.2, p. 42: "Any exit
/// from LPMx.5 causes a BOR").
///
/// Power draw in LPM4.5: Approx 42 nA (MSP430FR2355 without SVS: SLASEC4D Table 6-1, p. 61).
#[inline]
pub fn enter_lpm4_5<MODE: WatchdogSelect>(wdt: Wdt<MODE>, rtc_reg: _pac::Rtc, svs: SvsState) -> ! {
    // Disable RTC (RTCSS = 00b, no clock: SLAU445I Table 15-2, p. 420), so the device enters
    // LPM4.5 rather than LPM3.5 (SLAU445I 1.4.3.1, p. 41)
    unsafe { rtc_reg.rtcctl().clear_bits(|w| w.rtcss().disabled()) };

    // LPM4.5 stops every oscillator, so every pin returns to GPIO, the XT1 pins included
    // (SLAU445I 1.4.3.1, p. 41, step 2)
    reset_all_pin_functions(KeepXt1Pins::NONE);
    enter_lpmx_5(wdt, svs)
}

/// The XT1 pins to leave in their XT1 function when entering LPMx.5 (SLAU445I 1.4.3.1, p. 41,
/// step 2)
#[derive(Clone, Copy)]
pub(crate) struct KeepXt1Pins {
    xin: bool,
    xout: bool,
}

impl KeepXt1Pins {
    const NONE: Self = Self { xin: false, xout: false };

    /// The XT1 pins currently in their XT1 function. XT1 is in use when XIN is selected for it;
    /// XOUT only belongs to XT1 in crystal mode and may be a GPIO in bypass mode
    /// (SLAU445I 3.2.4, p. 103). A high-frequency XT1 keeps neither, which switches it off: "HFXT
    /// must be disabled before entering into LPM3, LPM4, or LPMx.5 mode" (SLASEC4D Table 6-1, note 2,
    /// p. 61).
    fn in_use() -> Self {
        #[cfg(feature = "xt1_high_frequency")]
        if unsafe { _pac::Cs::steal() }.csctl6().read().xts().bit_is_set() {
            return Self::NONE;
        }
        let xin = Xt1Xin::<()>::function_matches_type();
        Self { xin, xout: xin && Xt1Xout::<()>::function_matches_type() }
    }

    /// Bit mask of the kept pins on `PORT`, or `None` if neither XT1 pin is on `PORT`, which the compiler
    /// knows from the types
    fn mask_on<PORT: PortNum + 'static>(self) -> Option<u8> {
        fn pin_mask<PIN: AlternatePin, PORT: 'static>(keep: bool) -> Option<u8>
        where
            PIN::Port: 'static,
        {
            if TypeId::of::<PIN::Port>() != TypeId::of::<PORT>() {
                None
            } else if keep {
                Some(PIN::MASK)
            } else {
                Some(0)
            }
        }
        match (pin_mask::<Xt1Xin<()>, PORT>(self.xin), pin_mask::<Xt1Xout<()>, PORT>(self.xout)) {
            (None, None) => None,
            (xin, xout) => Some(xin.unwrap_or(0) | xout.unwrap_or(0)),
        }
    }
}

/// Return every pin of `PORT` to GPIO (PxSEL0 and PxSEL1 cleared), except the XT1 pins in `keep`
/// (SLAU445I 1.4.3.1, p. 41, step 2)
pub(crate) fn reset_pin_functions<PORT: PortNum + 'static>(keep: KeepXt1Pins) {
    let port = unsafe { PORT::steal() };
    match keep.mask_on::<PORT>() {
        // Clearing leaves only the bits in the mask set
        Some(keep) => {
            port.pxsel0_clear(keep);
            port.pxsel1_clear(keep);
        }
        // No XT1 pin on this port: one write each
        None => {
            port.pxsel0_wr(0);
            port.pxsel1_wr(0);
        }
    }
}

/// Define `reset_all_pin_functions()` in a device's `lpm` module, from the list of the device's
/// ports
macro_rules! reset_all_pin_functions_impl {
    ($($port:ident),+ $(,)?) => {
        /// Return every pin of the device to GPIO before LPMx.5, except the XT1 pins in `keep`
        /// (SLAU445I 1.4.3.1, p. 41, step 2)
        pub(crate) fn reset_all_pin_functions(keep: $crate::lpm::KeepXt1Pins) {
            $($crate::lpm::reset_pin_functions::<$crate::gpio::$port>(keep);)+
        }
    };
}
pub(crate) use reset_all_pin_functions_impl;

/// Configuration common to LPM3.5 and 4.5, following the entry steps of SLAU445I 1.4.3.1, p. 41
fn enter_lpmx_5<MODE: WatchdogSelect>(mut wdt: Wdt<MODE>, svs: SvsState) -> ! {
    // Take the CS and the PMM. Execution won't return from this fn.
    let (cs, pmm) = unsafe { (_pac::Cs::steal(), _pac::Pmm::steal()) };

    // Pause WDT (SLAU445I 1.4.3.1, p. 41, step 7: with the WDT in watchdog mode "the device does
    // not enter LPMx.5")
    wdt.pause();

    // A module that still requests ACLK keeps the device out of LPMx.5 (SLAU445I 3.2.12.1, p. 109;
    // ACLKREQEN: SLAU445I Table 3-12, p. 123)
    unsafe { cs.csctl8().clear_bits(|w| w.aclkreqen().clear_bit()) };
    // The low-power REFO mode must be switched off before LPMx.5, or it draws extra current
    // (SLAU445I Table 3-7, p. 116)
    #[cfg(feature = "enhanced_cs")]
    unsafe { cs.csctl3().clear_bits(|w| w.refolp().clear_bit()) };

    // Clear GIE (SLAU445I 1.4.3.1, p. 41, step 8)
    let interrupts_were_enabled = msp430::register::sr::read().gie();
    msp430::interrupt::disable();

    // Write PMM password to get PMM control regs
    // Set PMMREGOFF
    // (SLAU445I 1.4.3.1, p. 41, steps 9a to 9c; PMMPW, SVSHE and PMMREGOFF: SLAU445I Table 2-2,
    // p. 91)
    pmm.pmmctl0().write(|w| w
        .pmmpw().password()
        .svshe().variant(svs)
        .pmmregoff().set_bit()
    );

    // Write incorrect password to PMM to lock
    // Only write to the upper byte of PMMCTL0
    // (SLAU445I 1.4.3.1, p. 41, step 9d; a word write with a wrong password causes a PUC:
    // SLAU445I 2.3, p. 90)
    pmm.pmmctl0_h().write(|w| w.pmmpw().lock());

    // In manual mode, disconnect the LPM3.5 switch before the entry (SLAU445I 2.2.7, p. 88: "It is
    // recommended to turn off the switch to avoid unnecessary leakage before the device enters LPM3.5").
    // The BOR at the wake-up puts it back in automatic mode, connected (LPM5SM "rw-[0]", LPM5SW "rw-[1]":
    // SLAU445I Table 2-7, p. 97, with the key in SLAU445I Table 0-1, p. 28).
    #[cfg(feature = "lpm3_5_switch")]
    if pmm.pm5ctl0().read().lpm5sm().is_manual() {
        unsafe { pmm.pm5ctl0().clear_bits(|w| w.lpm5sw().disconnected()) };
    }

    // Enter LPMx.5 with CPUOFF, OSCOFF, SCG0 and SCG1 (SLAU445I 1.4.3.1, p. 41, step 10). If
    // interrupts were enabled, GIE is set again in the same instruction, as SLAU445I 8.3.3, p. 318
    // recommends ("TI also recommends setting GIE = 1 before entry into LPMx.5"), although step 8
    // of SLAU445I 1.4.3.1, p. 41 clears it.
    if interrupts_were_enabled {
        const LPMX_5: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF | GIE;
        set_sr_bits::<LPMX_5>();
    } else {
        const LPMX_5: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF;
        set_sr_bits::<LPMX_5>();
    }

    // LPMx.5 achieved.

    // This part won't actually run, but just to appease compiler about '!'
    #[allow(clippy::empty_loop)]
    loop {}
}
