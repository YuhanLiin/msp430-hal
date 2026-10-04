//! Watchdog timer, configurable as either a traditional watchdog or an interval timer, counting with a
//! 32-bit counter (SLAU445I 12.1, p. 361; SLAU445I 12.2.1, p. 363: "The WDTCNT is a 32-bit up
//! counter").
//!
//! **Note**: MSP430 devices will reset after bootup if watchdog is not stopped after an initial 32
//! ms interval (roughly) (SLAU445I 12.1, p. 361, note "Watchdog timer powers up active"; SLAU445I
//! 12.2.2, p. 363).
//! If this is undesirable, call `Wdt::constrain()` as soon in the application as possible to stop the
//! watchdog.

use crate::_pac::{self, wdt_a::wdtctl::Wdtssel};
use crate::clock::{Aclk, Smclk};
use core::{convert::Infallible, marker::PhantomData};

const PASSWORD: u8 = 0x5A;

/// Watchdog interval (WDTIS), in cycles of the watchdog clock: `_2g` is 2^31 cycles (18 h 12 min 16 s at
/// 32.768 kHz) down to `_64`, 2^6 cycles (1.95 ms) (SLAU445I Table 12-2, p. 366).
pub use crate::_pac::wdt_a::wdtctl::Wdtis as WdtClkPeriods;

mod sealed {
    use super::*;

    pub trait SealedWatchdogSelect {}

    impl SealedWatchdogSelect for WatchdogMode {}
    impl SealedWatchdogSelect for IntervalMode {}
}

/// Watchdog timer which can be configured to watchdog or interval (timer) mode (WDTTMSEL, SLAU445I
/// Table 12-2, p. 366; SLAU445I 12.2.2 and 12.2.3, p. 363)
pub struct Wdt<MODE> {
    _mode: PhantomData<MODE>,
    periph: _pac::WdtA,
}

impl Wdt<WatchdogMode> {
    /// Convert WDT peripheral into a watchdog timer (watchdog mode) and disable the watchdog. Set
    /// clock source to VLOCLK (WDTTMSEL = 0, WDTHOLD = 1, WDTSSEL = 10b: SLAU445I Table 12-2,
    /// p. 366).
    pub fn constrain(wdt: _pac::WdtA) -> Self {
        // Disable first (WDTHOLD stops the watchdog timer; WDTSSEL selects VLOCLK: SLAU445I Table 12-2,
        // p. 366)
        // Every write to WDTCTL carries the password, 05Ah, or the device resets with a PUC (WDTPW:
        // SLAU445I 12.2, p. 363; SLAU445I Table 12-2, p. 366)
        wdt.wdtctl().write(|w| {
            unsafe { w.wdtpw().bits(PASSWORD) }
            .wdthold().hold()
            .wdtssel().variant(Wdtssel::Vloclk)
        });
        Wdt { _mode: PhantomData, periph: wdt }
    }
}

/// Watchdog mode typestate: expiry of the interval resets the device with a PUC (SLAU445I 12.2.2,
/// p. 363)
pub struct WatchdogMode;
/// Interval mode typestate: expiry of the interval sets WDTIFG instead (SLAU445I 12.2.3, p. 363)
pub struct IntervalMode;

/// Marker trait for watchdog modes
pub trait WatchdogSelect: sealed::SealedWatchdogSelect {
    #[doc(hidden)]
    fn mode_bit() -> bool;
}
// WDTTMSEL: 0 = watchdog mode, 1 = interval timer mode (SLAU445I Table 12-2, p. 366)
impl WatchdogSelect for WatchdogMode {
    #[inline(always)]
    fn mode_bit() -> bool { false }
}
impl WatchdogSelect for IntervalMode {
    #[inline(always)]
    fn mode_bit() -> bool { true }
}

type WdtWriter = _pac::wdt_a::wdtctl::W;

impl<MODE: WatchdogSelect> Wdt<MODE> {
    #[inline(always)]
    fn prewrite(w: &mut WdtWriter, bits: u16) -> &mut WdtWriter {
        // Write argument bits, password, and correct mode bit (WDTTMSEL) to the watchdog write proxy
        // (SLAU445I Table 12-2, p. 366). WDTCTL reads 069h in the upper byte, so the password is always
        // written over it (SLAU445I 12.2, p. 363).
        unsafe { w.bits(bits) }
            .wdtpw().password()
            .wdttmsel().bit(MODE::mode_bit())
    }

    #[inline]
    fn set_clk(&mut self, clk_src: Wdtssel) -> &mut Self {
        // Halt timer first, as specified in the user's guide (SLAU445I 12.2.3, p. 363, note "Modifying the
        // watchdog timer": "The watchdog timer should be halted before changing the clock source")
        self.periph.wdtctl().write(|w| {
            Self::prewrite(w, 0)
                .wdthold().hold()
                // Also reset timer (WDTCNTCL: SLAU445I Table 12-2, p. 366)
                .wdtcntcl().set_bit()
        });
        // Set clock src and keep timer halted (WDTSSEL: SLAU445I Table 12-2, p. 366)
        self.periph.wdtctl().write(|w|
            Self::prewrite(w, 0)
            .wdtssel().variant(clk_src)
            .wdthold().hold());
        self
    }

    /// Set watchdog clock source to ACLK and halt timer (WDTSSEL = 01b: SLAU445I Table 12-2,
    /// p. 366).
    #[inline]
    pub fn set_aclk(&mut self, _clks: &Aclk) -> &mut Self { self.set_clk(Wdtssel::Aclk) }

    /// Set watchdog clock source to VLOCLK and halt timer (WDTSSEL = 10b: SLAU445I Table 12-2,
    /// p. 366).
    #[inline]
    pub fn set_vloclk(&mut self) -> &mut Self { self.set_clk(Wdtssel::Vloclk) }

    /// Set watchdog clock source to SMCLK and halt timer (WDTSSEL = 00b: SLAU445I Table 12-2,
    /// p. 366).
    #[inline]
    pub fn set_smclk(&mut self, _clks: &Smclk) -> &mut Self { self.set_clk(Wdtssel::Smclk) }

    /// Reset countdown, unpause timer, and set timeout in a single write (SLAU445I 12.2.3, p. 363: "The
    /// watchdog timer interval should be changed together with WDTCNTCL = 1 in a single instruction")
    #[inline]
    pub fn set_interval_and_start(&mut self, periods: WdtClkPeriods) {
        self.periph.wdtctl().modify(|r, w| {
            Self::prewrite(w, r.bits())
                .wdtcntcl()
                .set_bit()
                .wdthold()
                .unhold()
                .wdtis()
                .variant(periods)
        });
    }

    /// Pause the timer (WDTHOLD = 1: SLAU445I Table 12-2, p. 366).
    #[inline]
    pub fn pause(&mut self) {
        self.periph.wdtctl().modify(|r, w|
            Self::prewrite(w, r.bits())
            .wdthold().hold());
    }

    /// Resumes the timer, counting from the previously stored value (WDTHOLD = 0: SLAU445I Table 12-2,
    /// p. 366).
    #[inline]
    pub fn resume(&mut self) {
        self.periph.wdtctl().modify(|r, w| 
            Self::prewrite(w, r.bits())
            .wdthold().unhold());
    }
}

impl Wdt<WatchdogMode> {
    /// Convert to interval mode and pause timer (WDTTMSEL = 1, WDTHOLD = 1: SLAU445I Table 12-2,
    /// p. 366)
    #[inline]
    pub fn to_interval(self) -> Wdt<IntervalMode> {
        let mut wdt = Wdt { _mode: PhantomData, periph: self.periph };
        // Change mode bit and pause timer
        wdt.pause();
        wdt
    }

    /// Refreshes the watchdog timer, preventing the processor from being reset (WDTCNTCL: SLAU445I
    /// Table 12-2, p. 366; SLAU445I Example 12-1, p. 364).
    pub fn feed(&mut self) {
        self.periph.wdtctl().modify(|r, w| 
            Self::prewrite(w, r.bits())
            .wdtcntcl().set_bit());
    }
}

impl Wdt<IntervalMode> {
    /// Checks if the timer has expired, returning `Ok(())` if it has, otherwise `WouldBlock`.
    /// If called while the timer is not running, this will always return `WouldBlock`.
    ///
    /// Only available in interval mode: in watchdog mode the flag only tells that the last reset
    /// came from the watchdog (WDTIFG in SFRIFG1: SLAU445I 12.2.4, p. 363; SLAU445I Table 1-10,
    /// p. 63).
    #[inline]
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        let sfr = unsafe { &*_pac::Sfr::ptr() };
        if sfr.sfrifg1().read().wdtifg().bit_is_set() {
            unsafe { sfr.sfrifg1().clear_bits(|w| w.wdtifg().clear_bit()) };
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    /// Convert to watchdog mode and pause timer (WDTTMSEL = 0, WDTHOLD = 1: SLAU445I Table 12-2,
    /// p. 366)
    #[inline]
    pub fn to_watchdog(self) -> Wdt<WatchdogMode> {
        let mut wdt = Wdt { _mode: PhantomData, periph: self.periph };
        // Change mode bit and pause timer
        wdt.pause();
        // Wipe out old interrupt flag, which may cause a watchdog reset (in watchdog mode "the WDTIFG flag
        // sources a reset vector interrupt": SLAU445I 12.2.4, p. 363)
        let sfr = unsafe { &*_pac::Sfr::ptr() };
        unsafe { sfr.sfrifg1().clear_bits(|w| w.wdtifg().clear_bit()) };
        wdt
    }

    /// Enable interrupts for watchdog, which fires when the watchdog interrupt flag is set in
    /// interval mode. This setting does nothing in watchdog mode, but will carry over when
    /// switching to interval mode (WDTIE in SFRIE1: SLAU445I 12.2.3 and 12.2.4, p. 363; SLAU445I
    /// Table 1-9, p. 62).
    #[inline]
    pub fn enable_interrupts(&mut self) -> &mut Self {
        let sfr = unsafe { &*_pac::Sfr::ptr() };
        unsafe { sfr.sfrie1().set_bits(|w| w.wdtie().set_bit()) };
        self
    }

    /// Disable interrupts for watchdog (WDTIE in SFRIE1: SLAU445I 12.2.4, p. 363; SLAU445I
    /// Table 1-9, p. 62).
    #[inline]
    pub fn disable_interrupts(&mut self) -> &mut Self {
        let sfr = unsafe { &*_pac::Sfr::ptr() };
        unsafe { sfr.sfrie1().clear_bits(|w| w.wdtie().clear_bit()) };
        self
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::timer::{Cancel, CountDown, Periodic};
    use embedded_hal_02::watchdog::{Watchdog, WatchdogDisable, WatchdogEnable};

    impl Watchdog for Wdt<WatchdogMode> {
        #[inline]
        fn feed(&mut self) { self.feed() }
    }

    impl WatchdogEnable for Wdt<WatchdogMode> {
        type Time = WdtClkPeriods;

        #[inline]
        fn start<T>(&mut self, period: T)
        where T: Into<Self::Time> {
            self.set_interval_and_start(period.into());
        }
    }

    impl WatchdogDisable for Wdt<WatchdogMode> {
        #[inline]
        fn disable(&mut self) { self.pause(); }
    }

    impl CountDown for Wdt<IntervalMode> {
        type Time = WdtClkPeriods;

        #[inline]
        fn start<T>(&mut self, count: T)
        where T: Into<Self::Time> {
            self.set_interval_and_start(count.into());
        }

        /// If called while timer is not running, this will always return WouldBlock.
        #[inline]
        fn wait(&mut self) -> nb::Result<(), void::Void> {
            self.wait().map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl Cancel for Wdt<IntervalMode> {
        type Error = void::Void;

        /// This implementation will never return error even if watchdog has already been paused, hence
        /// the `Void` error type.
        #[inline]
        fn cancel(&mut self) -> Result<(), Self::Error> {
            self.pause();
            Ok(())
        }
    }

    impl Periodic for Wdt<IntervalMode> {}
}
