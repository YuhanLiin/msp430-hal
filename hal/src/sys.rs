//! System control: the RST/NMI pin, the vacant memory access interrupt, the JTAG mailbox, the
//! non-maskable interrupts (NMIs), the interrupt vectors in RAM, the bootloader (BSL) settings, and the
//! JTAG pin and PMM register protection
//!
//! Begin with [`SysParts::new()`], which splits the special function registers (SFR, SLAU445I 1.14,
//! p. 61) into these functions. The reset cause and software resets are on [`Pmm`](crate::pmm::Pmm), FRAM
//! bit error handling on [`Fram`](crate::fram::Fram).
//!
//! # Non-maskable interrupts
//!
//! The general interrupt enable (GIE) doesn't mask NMIs: once a source is enabled, it interrupts even
//! with interrupts disabled (SLAU445I 1.3.1, p. 33 and SLAU445I 1.3.4, p. 33). Two vectors take them
//! (SLAU445I 1.3.1, p. 33; vector tables: SLASEC4D Table 6-2, p. 63, SLASE59F Table 6-2, p. 41,
//! SLASEO7C Table 9-2, p. 46, SLASEE4C Table 6-2, p. 46; SYSUNIV and SYSSNIV: SLASEC4D Table 6-12,
//! p. 70, SLASE59F Table 6-9, p. 48, SLASEO7C Table 9-10, p. 53, SLASEE4C Table 6-10, p. 52):
//!
//! | Vector   | Sources                                                    | In the handler, call                  |
//! |:--------:|:----------------------------------------------------------:|:-------------------------------------:|
//! | `UNMI`   | An edge on the RST/NMI pin in NMI mode, oscillator faults  | [`take_nmi_pin_interrupt()`], [`clock::take_fault_interrupt()`](crate::clock::take_fault_interrupt) |
//! | `SYSNMI` | Vacant memory access, the JTAG mailbox, FRAM bit errors    | [`take_system_nmi()`]                 |
//!
//! While one NMI of a vector is handled, further NMIs of that vector wait until it returns (SLAU445I
//! 1.3.1, p. 33). A handler must clear the flag it handles, or the NMI repeats as soon as it returns
//! (SLAU445I 1.3.4.1, step 5, p. 33: "Multiple source flags remain set for servicing by software";
//! SLAU445I 1.3.7, p. 36).

use crate::_pac;
use crate::gpio::{Pin, Pin4, Pin5, Pin6, Pin7, P1};
use core::{convert::Infallible, marker::PhantomData};

/// The system control functions of the special function registers (SFR, SLAU445I Table 1-8, p. 61)
pub struct SysParts {
    /// The RST/NMI pin, in reset mode as after a brownout reset (SLAU445I 1.2.1, p. 32)
    pub rst_nmi_pin: RstNmiPin<ResetMode>,
    /// The vacant memory access interrupt (SLAU445I 1.9.2, p. 45)
    pub vacant_memory: VacantMemory,
    /// The JTAG mailbox, in 16-bit mode as after reset (JMBMODE, SLAU445I Table 1-15, p. 68)
    pub jtag_mailbox: JtagMailbox<Mode16>,
    /// Where the CPU takes the interrupt vectors from (SYSRIVECT, SLAU445I 1.3.6.1, p. 36)
    pub interrupt_vectors: InterruptVectors,
    /// The bootloader (BSL) settings (SYSBSLC, SLAU445I Table 1-14, p. 67)
    pub bsl: Bsl,
    /// The JTAG pins, which can be dedicated to JTAG until the next BOR (SYSJTAGPIN, SLAU445I Table 1-13,
    /// p. 66)
    pub jtag_pins: JtagPins,
    /// The protection of the PMM registers, until the next BOR (SYSPMMPE, SLAU445I Table 1-13, p. 66)
    pub pmm_protection: PmmProtection,
}

impl SysParts {
    /// Split the special function registers into their system control functions. The RST/NMI pin and
    /// the JTAG mailbox keep their current configuration.
    #[inline]
    pub fn new(_sfr: _pac::Sfr) -> Self {
        SysParts {
            rst_nmi_pin: RstNmiPin(PhantomData),
            vacant_memory: VacantMemory(()),
            jtag_mailbox: JtagMailbox(PhantomData),
            interrupt_vectors: InterruptVectors(()),
            bsl: Bsl(()),
            jtag_pins: JtagPins(()),
            pmm_protection: PmmProtection(()),
        }
    }
}

// The SFRs, base address 00100h (SLAU445I Table 1-7, p. 61)
#[inline(always)]
fn sfr() -> &'static _pac::sfr::RegisterBlock { unsafe { &*_pac::Sfr::ptr() } }

// The SYS registers (SLAU445I Table 1-12, p. 65)
#[inline(always)]
fn sys() -> &'static _pac::sys::RegisterBlock { unsafe { &*_pac::Sys::ptr() } }

/// Typestate for the RST/NMI pin in reset mode: a low level resets the device (SLAU445I 1.2, p. 30)
pub struct ResetMode;
/// Typestate for the RST/NMI pin in NMI mode: an edge requests the user NMI (SLAU445I 1.7, p. 43)
pub struct NmiMode;

/// The resistor on the RST/NMI pin (SFRRPCR.SYSRSTRE, SYSRSTUP, SLAU445I Table 1-11, p. 64)
///
/// The MSP430FR2433 has no `None`: clearing SYSRSTRE there also disables the pull-down of the TEST/SBWTCK
/// pin (erratum PORT28: SLAZ664S PORT28, p. 12 to p. 13).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RstPull {
    /// Pull-up, as after reset. The user's guide requires this or an external resistor if the pin is unused
    /// (SLAU445I 1.7, p. 43).
    Up,
    /// Pull-down (SYSRSTRE = 1, SYSRSTUP = 0: SLAU445I Table 1-11, p. 64)
    Down,
    /// No resistor (SYSRSTRE = 0: SLAU445I Table 1-11, p. 64).
    ///
    /// Not on the MSP430FR2433, because of erratum PORT28: clearing SYSRSTRE there also disables the
    /// internal pull-down of the TEST/SBWTCK pin, which "can lead to increased current consumption and
    /// unintentionally-enabled JTAG access to the device". The HAL follows the erratum's first workaround:
    /// "Do not clear the SFRRPCR.SYSRSTRE bit, use the SFRRPCR.SYSRSTRUP bit to define direction of the
    /// internal resistor on RST/NMI/SBWTDIO pin instead" (SLAZ664S PORT28, p. 12 to p. 13).
    #[cfg(not(feature = "erratum_port28"))]
    None,
}

/// The edge of the RST/NMI pin that requests the NMI, `Rising` (SYSNMIIES = 0) or `Falling` (SYSNMIIES =
/// 1) (SFRRPCR.SYSNMIIES, SLAU445I Table 1-11, p. 64)
pub use crate::_pac::sfr::sfrrpcr::Sysnmiies as NmiEdge;

/// The RST/NMI pin (SFRRPCR, SLAU445I 1.7, p. 43). It is also the Spy-Bi-Wire data pin (SBWTDIO:
/// SLASEC4D Table 4-1, p. 18; SLASE59F Table 4-1, p. 10; SLASEO7C Table 7-1, p. 11; SLASEE4C Table 4-1,
/// p. 11).
pub struct RstNmiPin<MODE>(PhantomData<MODE>);

impl<MODE> RstNmiPin<MODE> {
    /// Select the resistor on the pin (SFRRPCR.SYSRSTRE and SYSRSTUP, SLAU445I Table 1-11, p. 64).
    #[inline]
    pub fn set_pull(&mut self, pull: RstPull) {
        sfr().sfrrpcr().modify(|_, w| with_pull(w, pull));
    }

    /// Use the pin as reset input, as after a brownout reset (SLAU445I 1.2.1, p. 32). A low level then
    /// resets the device, right away if the pin is low (SLAU445I 1.2, p. 30).
    #[inline]
    pub fn into_reset(self, pull: RstPull) -> RstNmiPin<ResetMode> {
        disable_nmi_pin_interrupt();
        // SYSNMI = 0 selects the reset function (SLAU445I Table 1-11, p. 64)
        sfr().sfrrpcr().modify(|_, w| with_pull(w, pull).sysnmi().reset());
        RstNmiPin(PhantomData)
    }

    /// Use the pin as NMI input: `edge` sets the NMI flag, which requests the `UNMI` interrupt once
    /// enabled with [`RstNmiPin::enable_interrupts()`]. The pin no longer resets the device (SLAU445I
    /// 1.7, p. 43).
    #[inline]
    pub fn into_nmi(self, edge: NmiEdge, pull: RstPull) -> RstNmiPin<NmiMode> {
        let enabled = nmi_pin_interrupt_enabled();
        disable_nmi_pin_interrupt();
        let rpcr = sfr().sfrrpcr();
        // Changing SYSNMIIES in NMI mode can set the flag, so set it before switching to NMI mode with
        // SYSNMI (SLAU445I Table 1-11, p. 64: "Modify this bit when SYSNMI = 0 to avoid triggering an
        // accidental NMI")
        rpcr.modify(|_, w| with_pull(w, pull).sysnmiies().variant(edge));
        rpcr.modify(|_, w| w.sysnmi().nmi());
        clear_nmi_pin_flag();
        if enabled {
            enable_nmi_pin_interrupt();
        }
        RstNmiPin(PhantomData)
    }
}

impl RstNmiPin<ResetMode> {
    /// Enable or disable the digital filter that suppresses short pulses on the pin, so they don't
    /// reset the device (SFRRPCR.SYSFLTE, SLAU445I 1.7, p. 43 and SLAU445I Table 1-11, p. 64). It is enabled
    /// after reset. See the data sheet for the shortest pulse that resets the device (tRESET: SLASEC4D
    /// Table 5-2, p. 34; SLASE59F Table 5-3, p. 22; SLASEO7C 8.12.2.1, p. 26; SLASEE4C Table 5-3, p. 24).
    #[inline]
    pub fn set_filter(&mut self, enabled: bool) {
        sfr().sfrrpcr().modify(|_, w| w.sysflte().bit(enabled));
    }
}

impl RstNmiPin<NmiMode> {
    /// Change the edge that requests the NMI. This can set the NMI flag, so this clears it (SLAU445I
    /// Table 1-11, p. 64).
    #[inline]
    pub fn set_edge(&mut self, edge: NmiEdge) {
        let enabled = nmi_pin_interrupt_enabled();
        disable_nmi_pin_interrupt();
        // SYSNMIIES (SLAU445I Table 1-11, p. 64)
        sfr().sfrrpcr().modify(|_, w| w.sysnmiies().variant(edge));
        clear_nmi_pin_flag();
        if enabled {
            enable_nmi_pin_interrupt();
        }
    }

    /// Request the `UNMI` interrupt on the selected edge (NMIIE, SLAU445I Table 1-9, p. 62). Clear the
    /// flag with [`take_nmi_pin_interrupt()`] in the handler.
    #[inline]
    pub fn enable_interrupts(&mut self) { enable_nmi_pin_interrupt(); }

    /// Stop requesting the `UNMI` interrupt (NMIIE, SLAU445I Table 1-9, p. 62).
    #[inline]
    pub fn disable_interrupts(&mut self) { disable_nmi_pin_interrupt(); }
}

// SYSRSTRE and SYSRSTUP (SLAU445I Table 1-11, p. 64)
#[inline(always)]
fn with_pull(w: &mut _pac::sfr::sfrrpcr::W, pull: RstPull) -> &mut _pac::sfr::sfrrpcr::W {
    match pull {
        RstPull::Up => w.sysrstre().enable().sysrstup().pullup(),
        RstPull::Down => w.sysrstre().enable().sysrstup().pulldown(),
        #[cfg(not(feature = "erratum_port28"))]
        RstPull::None => w.sysrstre().disable().sysrstup().pulldown(),
    }
}

// SFRIE1.NMIIE, bit 4 (SLAU445I Table 1-9, p. 62)
#[inline(always)]
fn nmi_pin_interrupt_enabled() -> bool { sfr().sfrie1().read().nmiie().bit_is_set() }

// SFRIE1.NMIIE, bit 4 (SLAU445I Table 1-9, p. 62)
#[inline(always)]
fn enable_nmi_pin_interrupt() { unsafe { sfr().sfrie1().set_bits(|w| w.nmiie().set_bit()) } }

// SFRIE1.NMIIE, bit 4 (SLAU445I Table 1-9, p. 62)
#[inline(always)]
fn disable_nmi_pin_interrupt() { unsafe { sfr().sfrie1().clear_bits(|w| w.nmiie().clear_bit()) } }

// SFRIFG1.NMIIFG, bit 4 (SLAU445I Table 1-10, p. 63)
#[inline(always)]
fn clear_nmi_pin_flag() { unsafe { sfr().sfrifg1().clear_bits(|w| w.nmiifg().clear_bit()) } }

/// Returns `true`, and clears the flag (NMIIFG, SLAU445I Table 1-10, p. 63), if an edge on the RST/NMI
/// pin requested the user NMI. Call this in the `UNMI` handler, where an oscillator fault can request it
/// too (SLAU445I 1.3.1, p. 33).
#[inline]
pub fn take_nmi_pin_interrupt() -> bool {
    let sfr = sfr();
    let requested = sfr.sfrie1().read().nmiie().bit_is_set() && sfr.sfrifg1().read().nmiifg().bit_is_set();
    if requested {
        clear_nmi_pin_flag();
    }
    requested
}

/// The vacant memory access interrupt (SFRIE1.VMAIE, SLAU445I Table 1-9, p. 62)
///
/// Vacant memory is address space with nothing behind it. Reads return 3FFFh, and executing from it
/// runs `JMP $`, which hangs the CPU (SLAU445I 1.9.2, p. 45). This catches such accesses, for example
/// through a corrupted pointer, with the `SYSNMI` interrupt.
///
/// Erratum CPU46, on every supported device: after the POPM instruction "the last Stack Pointer increment
/// is followed by an unintended read access to the memory. If this read access is performed on vacant
/// memory, the VMAIFG will be set", which happens when POPM pops "up to the top of the STACK" (SLAZ695J
/// CPU46, p. 7; SLAZ664S CPU46, p. 8; SLAZ726B CPU46, p. 7; SLAZ705H CPU46, p. 7). The code rustc
/// generates doesn't use POPM: a disassembly of the example programs on 2026-10-05 found none, see
/// REFERENCES.md. Hand-written assembly that pops up to the top of the stack needs the erratum's
/// workaround.
pub struct VacantMemory(());

impl VacantMemory {
    /// Request the `SYSNMI` interrupt when the CPU accesses vacant memory (SLAU445I 1.9.2, p. 45), after
    /// clearing VMAIFG (SLAU445I Table 1-10, p. 63). [`take_system_nmi()`] then returns
    /// [`SystemNmi::VacantMemoryAccess`].
    #[inline]
    pub fn enable_interrupts(&mut self) {
        unsafe { sfr().sfrifg1().clear_bits(|w| w.vmaifg().clear_bit()) };
        unsafe { sfr().sfrie1().set_bits(|w| w.vmaie().set_bit()) };
    }

    /// Stop requesting the `SYSNMI` interrupt on vacant memory accesses (VMAIE, SLAU445I Table 1-9, p. 62).
    #[inline]
    pub fn disable_interrupts(&mut self) {
        unsafe { sfr().sfrie1().clear_bits(|w| w.vmaie().clear_bit()) };
    }
}

/// Typestate for 16-bit JTAG mailbox transfers, through SYSJMBI0 and SYSJMBO0 (SLAU445I 1.10.1 to
/// 1.10.3, p. 46)
pub struct Mode16;
/// Typestate for 32-bit JTAG mailbox transfers, through SYSJMBI0-1 and SYSJMBO0-1 (SLAU445I 1.10.1 to
/// 1.10.3, p. 46)
pub struct Mode32;

/// The JTAG mailbox, which exchanges messages with a debugger through the JTAG or Spy-Bi-Wire
/// connection (SLAU445I 1.10, p. 46). The debugger needs to support it.
pub struct JtagMailbox<MODE>(PhantomData<MODE>);

impl<MODE> JtagMailbox<MODE> {
    /// Request the `SYSNMI` interrupt when a message from the debugger arrives (JMBINIE, SLAU445I
    /// Table 1-9, p. 62; SLAU445I 1.10.4, p. 47). [`take_system_nmi()`] then returns
    /// [`SystemNmi::JtagMailboxIn`].
    #[inline]
    pub fn enable_rx_interrupts(&mut self) { unsafe { sfr().sfrie1().set_bits(|w| w.jmbinie().set_bit()) } }

    /// Stop requesting the `SYSNMI` interrupt for messages from the debugger (JMBINIE, SLAU445I Table 1-9,
    /// p. 62).
    #[inline]
    pub fn disable_rx_interrupts(&mut self) { unsafe { sfr().sfrie1().clear_bits(|w| w.jmbinie().clear_bit()) } }

    /// Request the `SYSNMI` interrupt when the debugger has read the outgoing message, so the next one
    /// can be written (JMBOUTIE, SLAU445I Table 1-9, p. 62; SLAU445I 1.10.4, p. 46). [`take_system_nmi()`]
    /// then returns [`SystemNmi::JtagMailboxOut`]. The mailbox starts out empty, so this requests the
    /// interrupt right away until a message is written (JMBOUTIFG resets to 1, SLAU445I Table 1-10, p. 63).
    #[inline]
    pub fn enable_tx_interrupts(&mut self) { unsafe { sfr().sfrie1().set_bits(|w| w.jmboutie().set_bit()) } }

    /// Stop requesting the `SYSNMI` interrupt for outgoing messages (JMBOUTIE, SLAU445I Table 1-9, p. 62).
    #[inline]
    pub fn disable_tx_interrupts(&mut self) { unsafe { sfr().sfrie1().clear_bits(|w| w.jmboutie().clear_bit()) } }
}

impl JtagMailbox<Mode16> {
    /// Switch to 32-bit transfers (JMBMODE). Read and write any partial message first, the user's
    /// guide warns that it can be lost otherwise (SLAU445I Table 1-15, p. 68: "pad and flush out any
    /// partial content to avoid data drops").
    #[inline]
    pub fn into_32bit(self) -> JtagMailbox<Mode32> {
        unsafe { sys().sysjmbc().set_bits(|w| w.jmbmode().set_bit()) };
        JtagMailbox(PhantomData)
    }

    /// Send `msg` to the debugger. Returns `WouldBlock` until the debugger has read the previous
    /// message (JMBOUT0FG, SLAU445I Table 1-15, p. 68).
    #[inline]
    pub fn write(&mut self, msg: u16) -> nb::Result<(), Infallible> {
        let sys = sys();
        if sys.sysjmbc().read().jmbout0fg().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }
        // SYSJMBO0 (SLAU445I Table 1-18, p. 70)
        sys.sysjmbo0().write(|w| unsafe { w.bits(msg) });
        Ok(())
    }

    /// A message from the debugger, or `WouldBlock` if none has arrived (JMBIN0FG). Reading it
    /// clears the flag, with JMBCLR0OFF = 0 as after reset (SLAU445I Table 1-15, p. 68).
    #[inline]
    pub fn read(&mut self) -> nb::Result<u16, Infallible> {
        let sys = sys();
        if sys.sysjmbc().read().jmbin0fg().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }
        // SYSJMBI0 (SLAU445I Table 1-16, p. 69)
        Ok(sys.sysjmbi0().read().bits())
    }
}

impl JtagMailbox<Mode32> {
    /// Switch to 16-bit transfers (JMBMODE). Read and write any partial message first, the user's
    /// guide warns that it can be lost otherwise (SLAU445I Table 1-15, p. 68: "pad and flush out any
    /// partial content to avoid data drops").
    #[inline]
    pub fn into_16bit(self) -> JtagMailbox<Mode16> {
        unsafe { sys().sysjmbc().clear_bits(|w| w.jmbmode().clear_bit()) };
        JtagMailbox(PhantomData)
    }

    /// Send `msg` to the debugger, the low half through SYSJMBO0 and the high half through SYSJMBO1.
    /// Returns `WouldBlock` until the debugger has read the previous message (JMBOUT0FG, JMBOUT1FG,
    /// SLAU445I 1.10.2, p. 46).
    #[inline]
    pub fn write(&mut self, msg: u32) -> nb::Result<(), Infallible> {
        let sys = sys();
        let jmbc = sys.sysjmbc().read();
        if jmbc.jmbout0fg().bit_is_clear() || jmbc.jmbout1fg().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }
        // SYSJMBO0 and SYSJMBO1 (SLAU445I Table 1-18, p. 70 and SLAU445I Table 1-19, p. 70)
        sys.sysjmbo0().write(|w| unsafe { w.bits(msg as u16) });
        sys.sysjmbo1().write(|w| unsafe { w.bits((msg >> 16) as u16) });
        Ok(())
    }

    /// A message from the debugger, or `WouldBlock` if none has arrived (JMBIN0FG, JMBIN1FG, SLAU445I
    /// 1.10.3, p. 46). The low half comes from SYSJMBI0 and the high half from SYSJMBI1. Reading it
    /// clears the flags, with JMBCLR0OFF = JMBCLR1OFF = 0 as after reset (SLAU445I Table 1-15, p. 68).
    #[inline]
    pub fn read(&mut self) -> nb::Result<u32, Infallible> {
        let sys = sys();
        let jmbc = sys.sysjmbc().read();
        if jmbc.jmbin0fg().bit_is_clear() || jmbc.jmbin1fg().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }
        // SYSJMBI0 and SYSJMBI1 (SLAU445I Table 1-16, p. 69 and SLAU445I Table 1-17, p. 69)
        let low = sys.sysjmbi0().read().bits() as u32;
        let high = sys.sysjmbi1().read().bits() as u32;
        Ok(high << 16 | low)
    }
}

// One past the last byte of RAM (SLASEC4D Table 6-4, p. 65, with the MSP430FR215x RAM sizes in SLASEC4D 1.1,
// p. 2; SLASE59F Table 6-23, p. 61; SLASEO7C Table 9-31, p. 73; SLASEE4C Table 6-19, p. 62)
#[cfg(any(feature = "msp430fr2355", feature = "msp430fr2155", feature = "msp430fr2433"))]
const RAM_END: usize = 0x3000;
#[cfg(any(feature = "msp430fr2353", feature = "msp430fr2153", feature = "msp430fr2512", feature = "msp430fr2522"))]
const RAM_END: usize = 0x2800;
#[cfg(feature = "msp430fr2475")]
const RAM_END: usize = 0x3800;
#[cfg(feature = "msp430fr2476")]
const RAM_END: usize = 0x4000;

/// Where the CPU takes the interrupt vectors from (SYSCTL.SYSRIVECT, SLAU445I 1.3.6.1, p. 36; SLAU445I
/// Table 1-13, p. 66): the table in program FRAM, FF80h to FFFFh, as after a BOR, or a copy in the top 128
/// bytes of RAM, whose handlers the program can change while it runs. (The FRAM table: "interrupt vectors
/// and signatures" in SLASEC4D Table 6-4, p. 65, SLASE59F Table 6-23, p. 61, SLASEO7C Table 9-31, p. 73,
/// SLASEE4C Table 6-19, p. 62.)
///
/// The RAM table needs the top 128 bytes of RAM to itself: shorten RAM in `memory.x` by 0x80, as the stack
/// starts at the end of RAM.
///
/// Only a BOR switches back to the FRAM table (SYSRIVECT `rw-[0]`, SLAU445I Table 1-13, p. 66, with the key
/// in SLAU445I Table 0-1, p. 28): a power cycle, a low level on the RST/NMI pin in reset mode, or
/// [`Pmm::software_bor`], among others (SLAU445I 1.2, p. 30). After other resets the RAM table stays in use,
/// with its handlers from before the reset, until the program calls [`InterruptVectors::use_fram`] or
/// [`InterruptVectors::use_ram`] again. Measured on an MSP430FR2476, a watchdog PUC with the RAM table in use
/// restarted the program at the reset vector in FRAM, not at the one in the RAM table; the user's guide
/// requires the one in FRAM to stay valid for the BOR (SLAU445I 1.3.6.1, p. 36). SYSRIVECT also stayed set
/// when mspdebug flashed a new program, so the new program's interrupts go to the old program's handlers
/// until it switches tables or a BOR happens.
///
/// [`Pmm::software_bor`]: crate::pmm::Pmm::software_bor
pub struct InterruptVectors(());

impl InterruptVectors {
    /// Copy the interrupt vectors from FRAM to the RAM table, and take them from there (SYSRIVECT = 1:
    /// "Interrupt vectors generated with end address TOP of RAM", SLAU445I Table 1-13, p. 66). Then change
    /// handlers with [`InterruptVectors::set_handler`].
    ///
    /// # Safety
    ///
    /// Nothing else may use the top 128 bytes of RAM: shorten RAM in `memory.x` by 0x80. The table stays in
    /// use through every reset but a BOR, see [`InterruptVectors`].
    #[inline]
    pub unsafe fn use_ram(&mut self) {
        // The FRAM table, FF80h to FFFFh, 64 words. Copied a word at a time with volatile accesses, which
        // the compiler keeps as a loop: `copy_nonoverlapping` links the library's memcpy (186 bytes) for
        // this one copy.
        let fram_table = 0xFF80 as *const u16;
        for i in 0..64 {
            ram_table().add(i).write_volatile(fram_table.add(i).read_volatile());
        }
        sys().sysctl().set_bits(|w| w.sysrivect().ram());
    }

    /// Take the interrupt vectors from the FRAM table again (SYSRIVECT = 0), as after a BOR (SLAU445I
    /// Table 1-13, p. 66).
    #[inline]
    pub fn use_fram(&mut self) {
        unsafe { sys().sysctl().clear_bits(|w| w.sysrivect().fram()) };
    }

    /// Point `interrupt`'s vector in the RAM table at `handler`, at the same place below the top of RAM as
    /// the vector has below FFFFh in FRAM (SLAU445I 1.3.6.1, p. 36). It takes effect while the RAM table is
    /// in use, see [`InterruptVectors::use_ram`], which overwrites it with the FRAM table, so call it
    /// afterwards. One word is written, so an interrupt can't see half a vector.
    ///
    /// # Safety
    ///
    /// As for [`InterruptVectors::use_ram`]. `handler` must be an interrupt handler.
    #[inline]
    pub unsafe fn set_handler(&mut self, interrupt: _pac::Interrupt, handler: unsafe extern "msp430-interrupt" fn()) {
        // The PAC numbers each interrupt by its place in its own vector table, which msp430-rt links to end
        // just below the reset vector at FFFEh. So interrupt n is at FFFEh - 2 * (len - n), slot 63 - len + n
        // of the 64-word table. The MSP430FR247x and MSP430FR25x2 PACs list 63 vectors, from FF80h; the
        // MSP430FR2433 PAC 59, from FF88h; the MSP430FR2355 PAC 45, from FFA4h.
        let first = 63 - _pac::__INTERRUPTS.len();
        // Volatile: only the interrupt logic reads the table
        ram_table().add(first + interrupt as u16 as usize).write_volatile(handler as usize as u16);
    }
}

// The RAM table: the 64 words below the end of RAM (SLAU445I 1.3.6.1, p. 36)
#[inline(always)]
fn ram_table() -> *mut u16 { (RAM_END - 0x80) as *mut u16 }

/// The bootloader (BSL) settings (SYSBSLC, SLAU445I Table 1-14, p. 67) and the BSL entry indication
/// (SYSBSLIND, SLAU445I Table 1-13, p. 66). A BOR resets the settings (`rw-[0]`, SLAU445I Table 1-14, p. 67,
/// with the key in SLAU445I Table 0-1, p. 28). Measured on an MSP430FR2476, SYSBSLC read 0000h when the
/// program started, after a software BOR too: the boot code left the BSL unprotected.
///
/// Erratum BSL18, on revision A of the MSP430FR2433 (hardware revision 10h, SLAZ664S 5.3, p. 5): "An empty
/// reset vector (for example, as on an un-programmed device) should invoke the BSL, but it does not". The
/// workaround: "Use the dedicated TEST and RST pins to perform hardware BSL invocation, or perform
/// software BSL invocation from the main application" (SLAZ664S BSL18, p. 6; revisions: SLAZ664S 1, p. 2).
pub struct Bsl(());

impl Bsl {
    /// Whether a BSL entry sequence was detected on the Spy-Bi-Wire pins (SYSBSLIND, SLAU445I Table 1-13,
    /// p. 66)
    #[inline]
    pub fn entry_detected(&self) -> bool { sys().sysctl().read().sysbslind().is_set() }

    /// Protect the BSL memory (`true`), or leave it unprotected: "Read, program, and erase of BSL memory is
    /// possible" (`false`) (SYSBSLPE, SLAU445I Table 1-14, p. 67). A BOR clears it, and "the boot code that
    /// checks for an available BSL may set this bit in software to protect the BSL". The protection covers
    /// the RAM assigned with [`Bsl::set_ram_assigned`] as well. Measured on an MSP430FR2476, the program
    /// could still read the BSL memory with the protection on.
    #[inline]
    pub fn set_protection(&mut self, protect: bool) {
        if protect {
            unsafe { sys().sysbslc().set_bits(|w| w.sysbslpe().prot()) };
        } else {
            unsafe { sys().sysbslc().clear_bits(|w| w.sysbslpe().notprot()) };
        }
    }

    /// Switch the BSL memory off (`true`): it then "behaves like vacant memory. Reads cause 3FFFh to be read.
    /// Fetches cause JMP $ to be executed" (SYSBSLOFF, SLAU445I Table 1-14, p. 67). `false`, as after a BOR,
    /// switches it back on.
    #[inline]
    pub fn set_memory_off(&mut self, off: bool) {
        if off {
            unsafe { sys().sysbslc().set_bits(|w| w.sysbsloff().off()) };
        } else {
            unsafe { sys().sysbslc().clear_bits(|w| w.sysbsloff().on()) };
        }
    }

    /// Assign the lowest 16 bytes of RAM to the BSL (`true`), or give them back (`false`, as after a BOR)
    /// (SYSBSLR, SLAU445I Table 1-14, p. 67).
    ///
    /// # Safety
    ///
    /// The program must not use those 16 bytes, 2000h to 200Fh: start RAM 0x10 later in `memory.x`. With the
    /// BSL protected, "access to these RAM locations is only possible from within the protected BSL memory
    /// segments" (SLAU445I 1.9.4, p. 45). Measured on an MSP430FR2476, a read of them from the program then
    /// reset the device with a security violation, a BOR
    /// ([`ResetCause::SecurityViolation`](crate::pmm::ResetCause::SecurityViolation)).
    #[inline]
    pub unsafe fn set_ram_assigned(&mut self, assigned: bool) {
        if assigned {
            sys().sysbslc().set_bits(|w| w.sysbslr().ram());
        } else {
            sys().sysbslc().clear_bits(|w| w.sysbslr().noram());
        }
    }
}

/// The JTAG pins, P1.4 (TCK), P1.5 (TMS), P1.6 (TDI/TCLK) and P1.7 (TDO) on every supported device
/// (SLASEC4D Table 4-2, p. 23; SLASE59F Table 4-2, p. 12; SLASEO7C Table 7-2, p. 15; SLASEE4C Table 4-2,
/// p. 13)
pub struct JtagPins(());

impl JtagPins {
    /// Dedicate the JTAG pins to 4-wire JTAG until the next BOR (SYSJTAGPIN: "Setting this bit disables the
    /// shared digital functionality of the JTAG pins and permanently enables the JTAG function. This bit can
    /// only be set once. After the bit is set, it remains set until a BOR occurs", SLAU445I Table 1-13,
    /// p. 66). It takes the pins, which can't be used for anything else afterwards. A debugger then selects
    /// the mode with "explicit 4-wire JTAG mode selection" instead of the JTAG/SBW sequence.
    #[inline]
    pub fn dedicate<TCK, TMS, TDI, TDO>(
        self,
        _tck: Pin<P1, Pin4, TCK>,
        _tms: Pin<P1, Pin5, TMS>,
        _tdi: Pin<P1, Pin6, TDI>,
        _tdo: Pin<P1, Pin7, TDO>,
    ) {
        unsafe { sys().sysctl().set_bits(|w| w.sysjtagpin().dedicated()) };
    }
}

/// The protection of the PMM registers (SYSPMMPE, SLAU445I Table 1-13, p. 66)
pub struct PmmProtection(());

impl PmmProtection {
    /// Protect the PMM registers until the next BOR: "After the bit is set to 1, it only can be cleared by
    /// a BOR", with "Access only from the protected BSL segments" (SYSPMMPE, SLAU445I Table 1-13, p. 66).
    /// Afterwards the HAL can't change them either: not through [`Pmm`](crate::pmm::Pmm) (`set_svsh()`, the
    /// references and the software resets, say), and not on the way into LPM3.5 or LPM4.5, which sets
    /// PMMREGOFF (SLAU445I 1.4.3.1, p. 41).
    #[inline]
    pub fn enable(self) {
        unsafe { sys().sysctl().set_bits(|w| w.syspmmpe().en()) };
    }
}

/// The sources of the system NMI, in priority order (SYSSNIV: SLASEC4D Table 6-12, p. 70; SLASE59F
/// Table 6-9, p. 48; SLASEO7C Table 9-10, p. 53; SLASEE4C Table 6-10, p. 52)
///
/// - `SvsLowPowerResetEntry`: SVS low-power reset entry (the low-power reset state: SLAU445I 2.2.5, p. 87).
/// - `FramUncorrectableBitError`: the FRAM detected a bit error it couldn't correct, see
///   [`Fram::set_uncorrectable_bit_error_action()`](crate::fram::Fram::set_uncorrectable_bit_error_action)
///   (UBDIFG, SLAU445I 6.6, p. 303).
/// - `VacantMemoryAccess`: the CPU accessed vacant memory, see [`VacantMemory`] (SLAU445I 1.9.2, p. 45).
/// - `JtagMailboxIn`: a message from the debugger arrived in the JTAG mailbox (JMBINIFG, SLAU445I
///   1.10.4, p. 47).
/// - `JtagMailboxOut`: the debugger read the outgoing JTAG mailbox message (JMBOUTIFG, SLAU445I 1.10.4,
///   p. 46).
/// - `FramCorrectableBitError`: the FRAM detected and corrected a bit error, see
///   [`Fram::enable_correctable_bit_error_interrupts()`](crate::fram::Fram::enable_correctable_bit_error_interrupts)
///   (CBDIFG, SLAU445I 6.6, p. 303).
pub use crate::_pac::sys::syssniv::Syssniv as SystemNmi;

/// Returns the highest-priority pending system NMI source and clears its flag (SYSSNIV, SLAU445I
/// 1.3.7, p. 36), or `None` if none is pending. Call this in the `SYSNMI` handler, until it returns
/// `None` if several sources are enabled.
#[inline]
pub fn take_system_nmi() -> Option<SystemNmi> {
    // 00h, no interrupt pending, has no variant, nor do the values the data sheets reserve
    sys().syssniv().read().syssniv().variant()
}
