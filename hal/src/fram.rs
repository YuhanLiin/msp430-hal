//! FRAM controller
//!
//! Besides the wait states (SLAU445I 6.5, p. 302), [`Fram`] configures what happens when the FRAM
//! detects a bit error (SLAU445I 6.6, p. 303). Its error correction fixes single-bit errors, and the
//! controller can report those and the errors it can't correct (SLAU445I 6.3, p. 301: the ECC logic
//! "can correct bit errors and detect multiple bit errors").
//!
//! On the MSP430FR2433 the bit error detection can report errors that don't exist:
//! - after a wake-up from LPM1 to LPM4 (SLAZ664S GC5), which the low-power mode functions work around,
//!   see [`request_lpm3`](crate::lpm::request_lpm3);
//! - while running from FRAM with MCLK from the DCO at 16 MHz, or from a high-frequency clock above
//!   12 MHz (SLAZ664S GC4). With [`UncorrectableBitError::Reset`] this resets the device, and SYSRSTIV
//!   then reads 0, so [`Pmm::take_reset_cause()`](crate::pmm::Pmm::take_reset_cause) returns `None`. The
//!   erratum's workarounds are to "Check the reset source for SYSRSTIV = 0 and ignore the reset", to
//!   leave UBDRSTEN at 0 ([`UncorrectableBitError::Ignore`] or [`UncorrectableBitError::Interrupt`]), or
//!   to "Set the MCLK to maximum 12MHz".

use crate::_pac;

/// FRAM controller
pub struct Fram {
    fram: _pac::Frctl,
}

impl Fram {
    /// Turn FRCTL into `Fram`
    pub fn new(fram: _pac::Frctl) -> Self { Fram { fram } }
}

/// FRAM wait states, `Wait0` to `Wait7` (NWAITS, SLAU445I Table 6-2, p. 306)
pub use crate::_pac::frctl::frctl0::Nwaits as WaitStates;

/// What the FRAM controller does when it detects a bit error it can't correct (GCCTL0.UBDRSTEN, UBDIE,
/// SLAU445I Table 6-3, p. 307)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UncorrectableBitError {
    /// Nothing, as after reset (UBDRSTEN and UBDIE reset to 0, SLAU445I Table 6-3, p. 307)
    Ignore,
    /// Reset the device with a PUC (UBDRSTEN, SLAU445I Table 6-3, p. 307).
    /// [`Pmm::take_reset_cause()`](crate::pmm::Pmm::take_reset_cause) then returns
    /// [`ResetCause::FramBitError`](crate::pmm::ResetCause::FramBitError). On the MSP430FR2433 with MCLK
    /// above 12 MHz it can also reset the device for errors that don't exist, with no reset cause
    /// (SLAZ664S GC4), see the [module documentation](crate::fram).
    Reset,
    /// Request the `SYSNMI` interrupt (UBDIE, SLAU445I Table 6-3, p. 307; SLAU445I 1.3.1, p. 33).
    /// [`take_system_nmi()`](crate::sys::take_system_nmi) then returns
    /// [`SystemNmi::FramUncorrectableBitError`](crate::sys::SystemNmi::FramUncorrectableBitError).
    Interrupt,
}

impl Fram {
    /// Run `f` with write access to the FRAM controller registers, and lock them again afterwards.
    ///
    /// The password in FRCTL0 unlocks them, and a byte write of a wrong password to the upper byte
    /// of FRCTL0 locks them again (SLAU445I 6.10, p. 305; SLAU445I Table 6-2, p. 306). Writing them
    /// while locked causes a PUC (SLAU445I 6.10, p. 305).
    #[inline]
    fn unlocked<R>(&mut self, f: impl FnOnce(&_pac::Frctl) -> R) -> R {
        critical_section::with(|_| {
            self.fram.frctl0().modify(|_, w| w.frctlpw().password());
            let ret = f(&self.fram);
            self.fram.frctl0_h().write(|w| w.frctlpw().lock());
            ret
        })
    }

    /// Set number of FRAM wait states. Could cause issues reading instructions from FRAM if
    /// incorrect (the device resets with a PUC if MCLK is too fast for them, SLAU445I 6.5,
    /// p. 302).
    /// # Safety
    /// Should wait 1 cycle if MCLK > 8MHz and 2 cycles if MCLK > 16MHz (fSYSTEM: SLASEC4D 5.3,
    /// p. 27; SLASE59F 5.3, p. 16; SLASEO7C 8.3, p. 20; SLASEE4C 5.3, p. 17).
    #[inline]
    pub unsafe fn set_wait_states(&mut self, wait: WaitStates) {
        self.unlocked(|fram| fram.frctl0().write(|w| w
            .frctlpw().password()
            .nwaits().variant(wait)));
    }

    /// Select what happens when the FRAM detects a bit error it can't correct (UBDRSTEN, UBDIE:
    /// SLAU445I Table 6-3, p. 307).
    #[inline]
    pub fn set_uncorrectable_bit_error_action(&mut self, action: UncorrectableBitError) {
        self.unlocked(|fram| {
            // UBDRSTEN and UBDIE must not be set at the same time, so clear both first (SLAU445I
            // Table 6-3, p. 307: "not allowed to be set simultaneously")
            fram.gcctl0().modify(|_, w| w.ubdrsten().clear_bit().ubdie().clear_bit());
            match action {
                UncorrectableBitError::Ignore => (),
                UncorrectableBitError::Reset => { fram.gcctl0().modify(|_, w| w.ubdrsten().set_bit()); }
                UncorrectableBitError::Interrupt => { fram.gcctl0().modify(|_, w| w.ubdie().set_bit()); }
            }
        });
    }

    /// Request the `SYSNMI` interrupt when the FRAM detects and corrects a bit error (CBDIE,
    /// SLAU445I Table 6-3, p. 307; SLAU445I 6.6, p. 303).
    /// [`take_system_nmi()`](crate::sys::take_system_nmi) then returns
    /// [`SystemNmi::FramCorrectableBitError`](crate::sys::SystemNmi::FramCorrectableBitError).
    #[inline]
    pub fn enable_correctable_bit_error_interrupts(&mut self) {
        self.unlocked(|fram| fram.gcctl0().modify(|_, w| w.cbdie().set_bit()));
    }

    /// Stop requesting the `SYSNMI` interrupt for corrected bit errors (CBDIE, SLAU445I Table 6-3,
    /// p. 307).
    #[inline]
    pub fn disable_correctable_bit_error_interrupts(&mut self) {
        self.unlocked(|fram| fram.gcctl0().modify(|_, w| w.cbdie().clear_bit()));
    }

    /// Leave the first `kib` KiB of program FRAM writable, while the program FRAM write protection
    /// protects the rest (SYSCFG0.FRWPOA, SLAU445I 1.12.4.1, p. 53). 0, as after reset, protects all
    /// of it (SLAU445I Table 1-24, p. 75; SLAU445I Table 1-29, p. 80).
    ///
    /// Use this for data that changes often, in place of the information memory (SLAU445I 1.12.4.1,
    /// p. 53: "This unprotected range can be used like RAM for random frequent writes"). By default
    /// the linker puts the program at the start of program FRAM, so first reserve this space for
    /// data in `memory.x`, or the program becomes writable instead.
    ///
    /// Only available on the MSP430FR2x5x and the MSP430FR2522 (SLAU445I 1.16, p. 73, SYSCFG0: SLAU445I
    /// Table 1-24, p. 75, "valid only in the MSP430FR235x and MSP430FR215x devices"; SLAU445I
    /// Table 1-29, p. 80, "valid in MSP430FR2522 and MSP430FR2422 devices").
    ///
    /// # Panics
    ///
    /// If `kib` is above 63 (FRWPOA is 6 bits, 0 KB to 63 KB: SLAU445I 1.12.4.1, p. 53).
    #[cfg(feature = "frwpoa")]
    #[inline]
    pub fn set_writable_program_fram(&mut self, kib: u8) {
        assert!(kib <= 63, "at most 63 KiB of program FRAM can be left writable");
        let sys = unsafe { &*_pac::Sys::ptr() };
        critical_section::with(|_| {
            // The password goes in the same word write as FRWPOA (SLAU445I Table 1-24, p. 75;
            // SLAU445I Table 1-29, p. 80: "written with the FRAM protection bits in a word in a
            // single operation")
            sys.syscfg0().modify(|_, w| unsafe { w
                .frwppw().password()
                .frwpoa().bits(kib)
            })
        });
    }
}
