//! Watchdog timer, configurable as either a traditional watchdog or an interval timer, counting with a
//! 32-bit counter (SLAU445I 12.2.1, p. 363).
//!
//! **Note**: MSP430 devices will reset after bootup if watchdog is not stopped after an initial 32
//! ms interval (roughly) (SLAU445I 12.1, p. 361). If this is undesirable, call `Wdt::constrain()` as soon
//! in the application as possible to stop the watchdog.

use crate::_pac::{self, wdt_a::wdtctl::Wdtssel};
use crate::clock::{Aclk, Smclk};
use core::{convert::Infallible, marker::PhantomData};

// WDTPW: every write to WDTCTL carries the password, or the device resets with a PUC (SLAU445I 12.2, p. 363)
const PASSWORD: u8 = 0x5A;

pub use crate::_pac::wdt_a::wdtctl::Wdtis as WdtClkPeriods;

mod sealed {
    use super::*;

    pub trait SealedWatchdogSelect {}

    impl SealedWatchdogSelect for WatchdogMode {}
    impl SealedWatchdogSelect for IntervalMode {}
}

/// Watchdog timer which can be configured to watchdog or interval (timer) mode
pub struct Wdt<MODE> {
    _mode: PhantomData<MODE>,
    periph: _pac::WdtA,
}

impl Wdt<WatchdogMode> {
    /// Convert WDT peripheral into a watchdog timer (watchdog mode) and disable the watchdog. Set
    /// clock source to VLOCLK.
    pub fn constrain(wdt: _pac::WdtA) -> Self {
        // Disable first
        wdt.wdtctl().write(|w| {
            unsafe { w.wdtpw().bits(PASSWORD) }
            .wdthold().hold()
            .wdtssel().variant(Wdtssel::Vloclk)
        });
        Wdt { _mode: PhantomData, periph: wdt }
    }
}

/// Watchdog mode typestate: expiry of the interval resets the device with a PUC (SLAU445I 12.2.2, p. 363)
pub struct WatchdogMode;
/// Interval mode typestate: expiry of the interval sets WDTIFG (SLAU445I 12.2.3, p. 363)
pub struct IntervalMode;

/// Marker trait for watchdog modes
pub trait WatchdogSelect: sealed::SealedWatchdogSelect {
    #[doc(hidden)]
    fn mode_bit() -> bool;
}
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
        // Write argument bits, password, and correct mode bit to the watchdog write proxy (WDTCTL: SLAU445I
        // Table 12-2, p. 366)
        unsafe { w.bits(bits).wdtpw().bits(PASSWORD) }
            .wdttmsel().bit(MODE::mode_bit())
    }

    #[inline]
    fn set_clk(&mut self, clk_src: Wdtssel) -> &mut Self {
        // Halt timer first, as specified in the user's guide (SLAU445I 12.2.3, p. 363)
        self.periph.wdtctl().write(|w| {
            Self::prewrite(w, 0)
                .wdthold().hold()
                // Also reset timer
                .wdtcntcl().set_bit()
        });
        // Set clock src and keep timer halted
        self.periph.wdtctl().write(|w|
            Self::prewrite(w, 0)
            .wdtssel().variant(clk_src)
            .wdthold().hold());
        self
    }

    /// Set watchdog clock source to ACLK and halt timer.
    #[inline]
    pub fn set_aclk(&mut self, _clks: &Aclk) -> &mut Self { self.set_clk(Wdtssel::Aclk) }

    /// Set watchdog clock source to VLOCLK and halt timer.
    #[inline]
    pub fn set_vloclk(&mut self) -> &mut Self { self.set_clk(Wdtssel::Vloclk) }

    /// Set watchdog clock source to SMCLK and halt timer.
    #[inline]
    pub fn set_smclk(&mut self, _clks: &Smclk) -> &mut Self { self.set_clk(Wdtssel::Smclk) }

    /// Reset countdown, unpause timer, and set timeout in a single write, as the user's guide requires
    /// (SLAU445I 12.2.3, p. 363)
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

    /// Pause the timer.
    #[inline]
    pub fn pause(&mut self) {
        self.periph.wdtctl().modify(|r, w|
            Self::prewrite(w, r.bits())
            .wdthold().hold());
    }

    /// Resumes the timer, counting from the previously stored value.
    #[inline]
    pub fn resume(&mut self) {
        self.periph.wdtctl().modify(|r, w| 
            Self::prewrite(w, r.bits())
            .wdthold().unhold());
    }
}

impl Wdt<WatchdogMode> {
    /// Convert to interval mode and pause timer
    #[inline]
    pub fn to_interval(self) -> Wdt<IntervalMode> {
        let mut wdt = Wdt { _mode: PhantomData, periph: self.periph };
        // Change mode bit and pause timer
        wdt.pause();
        wdt
    }

    /// Refreshes the watchdog timer, preventing the processor from being reset.
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
    /// Only available in interval mode. In watchdog mode the expiry resets the device, and the reset clears
    /// WDTIFG (SLAU445I Table 1-10, p. 63), so waiting would only wait for the reset.
    #[inline]
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        let sfr = unsafe { _pac::Sfr::steal() };
        if sfr.sfrifg1().read().wdtifg().bit_is_set() {
            unsafe { sfr.sfrifg1().clear_bits(|w| w.wdtifg().clear_bit()) };
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    /// Convert to watchdog mode and pause timer
    #[inline]
    pub fn to_watchdog(self) -> Wdt<WatchdogMode> {
        let mut wdt = Wdt { _mode: PhantomData, periph: self.periph };
        // Change mode bit and pause timer
        wdt.pause();
        // Wipe out old interrupt flag, which may cause a watchdog reset (SLAU445I 12.2.4, p. 363)
        let sfr = unsafe { _pac::Sfr::steal() };
        unsafe { sfr.sfrifg1().clear_bits(|w| w.wdtifg().clear_bit()) };
        wdt
    }

    /// Enable interrupts for watchdog, which fires when the watchdog interrupt flag is set in
    /// interval mode. This setting does nothing in watchdog mode, but will carry over when
    /// switching to interval mode (WDTIE in SFRIE1: SLAU445I 12.2.4, p. 363).
    #[inline]
    pub fn enable_interrupts(&mut self) -> &mut Self {
        let sfr = unsafe { _pac::Sfr::steal() };
        unsafe { sfr.sfrie1().set_bits(|w| w.wdtie().set_bit()) };
        self
    }

    /// Disable interrupts for watchdog (WDTIE in SFRIE1: SLAU445I 12.2.4, p. 363).
    #[inline]
    pub fn disable_interrupts(&mut self) -> &mut Self {
        let sfr = unsafe { _pac::Sfr::steal() };
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
