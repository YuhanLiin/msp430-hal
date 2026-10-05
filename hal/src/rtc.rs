//! Real time counter
//!
//! Can be used as a periodic 16-bit timer (SLAU445I 15.1, p. 416).
//!
//! Supports SMCLK, ACLK, VLOCLK, and XT1CLK as clock sources (SLAU445I 15.1, p. 416). The MSP430FR2433
//! has no ACLK option (SLASE59F Table 6-7, p. 46).
//!
//! Note: On devices that can clock the RTC from ACLK (FR2x5x, FR247x and FR25x2), ACLK and
//! SMCLK share the same RTCSS bit pattern and are further distinguished via the SYSCFG2
//! RTCCKSEL selection (RTCSS = 01b, "Device specific": SLAU445I Table 15-2, p. 420; RTCCKSEL:
//! SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31, p. 82; devices: SLASEC4D Table 6-9, p. 68;
//! SLASEO7C Table 9-8, p. 50; SLASEE4C Table 6-8, p. 49).

use crate::_pac::{self, rtc::rtcctl::Rtcss};
use crate::clock::{Smclk, Xt1clk};
use core::{convert::Infallible, marker::PhantomData};

#[cfg(feature = "rtc_aclk")]
use crate::clock::Aclk;

mod sealed {
    use super::*;

    pub trait SealedRtcClockSrc {}

    impl SealedRtcClockSrc for RtcSmclk {}
    impl SealedRtcClockSrc for RtcVloclk {}
    #[cfg(feature = "rtc_aclk")]
    impl SealedRtcClockSrc for RtcAclk {}
    impl SealedRtcClockSrc for RtcXt1clk {}
}

/// Marker trait for RTC clock sources
pub trait RtcClockSrc: sealed::SealedRtcClockSrc {
    #[doc(hidden)]
    const CLK_SRC: Rtcss;

    /// Optional hook for clock-specific hardware configuration (e.g., SYSCFG muxes: RTCCKSEL in
    /// SYSCFG2, SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31, p. 82)
    #[doc(hidden)]
    fn apply_sys_config() {}
}

/// Typestate representing the SMCLK clock source for RTC, which only runs in active mode and LPM0
/// (SLAU445I 15.1, p. 416: "SMCLK is functional in AM and LPM0 only")
pub struct RtcSmclk;

impl RtcClockSrc for RtcSmclk {
    // RTCSS = 01b, the device-specific source (SLAU445I Table 15-2, p. 420)
    const CLK_SRC: Rtcss = Rtcss::Smclk;

    #[cfg(feature = "rtc_aclk")]
    fn apply_sys_config() {
        // Ensure the mux is set to SMCLK (0) (RTCCKSEL: SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31,
        // p. 82)
        let sys = unsafe { &*_pac::Sys::ptr() };
        sys.syscfg2().modify(|_, w| w.rtccksel().clear_bit());
    }
}

/// Typestate representing the VLOCLK clock source for RTC, about 10 kHz (SLAU445I 15.1, p. 416)
pub struct RtcVloclk;

impl RtcClockSrc for RtcVloclk {
    // RTCSS = 11b (SLAU445I Table 15-2, p. 420)
    const CLK_SRC: Rtcss = Rtcss::Vloclk;
}

/// Typestate representing the ACLK clock source for RTC, which runs from active mode to LPM3
/// (SLAU445I 15.1, p. 416: "ACLK is functional in AM to LPM3")
#[cfg(feature = "rtc_aclk")]
pub struct RtcAclk;

#[cfg(feature = "rtc_aclk")]
impl RtcClockSrc for RtcAclk {
    // RTCSS = 01b, the device-specific source, which RTCCKSEL = 1 makes ACLK (SLAU445I Table 15-2, p. 420)
    const CLK_SRC: Rtcss = Rtcss::Smclk;

    fn apply_sys_config() {
        // Ensure the mux is set to ACLK (1) (RTCCKSEL: SLAU445I Table 1-26, p. 77; SLAU445I Table 1-31,
        // p. 82)
        let sys = unsafe { &*_pac::Sys::ptr() };
        sys.syscfg2().modify(|_, w| w.rtccksel().set_bit());
    }
}

/// Typestate representing the XT1CLK clock source for RTC, about 32 kHz (SLAU445I 15.1, p. 416)
pub struct RtcXt1clk;

impl RtcClockSrc for RtcXt1clk {
    // RTCSS = 10b (SLAU445I Table 15-2, p. 420)
    const CLK_SRC: Rtcss = Rtcss::Xt1clk;
}

/// Marker trait for RTC clock sources that keep running in LPM3.5 (VLOCLK and XT1CLK: SLAU445I 15.1,
/// p. 416; SLAU445I 15.2.2, p. 417)
pub trait RtcLpm3_5ClockSrc: RtcClockSrc {}

impl RtcLpm3_5ClockSrc for RtcVloclk {}
impl RtcLpm3_5ClockSrc for RtcXt1clk {}

/// 16-bit real-time counter (SLAU445I 15.1, p. 416)
pub struct Rtc<SRC: RtcClockSrc> {
    periph: _pac::Rtc,
    _src: PhantomData<SRC>,
}

impl Rtc<RtcVloclk> {
    /// Convert into RTC object with VLOCLK as clock source. The clock source (RTCSS, SLAU445I
    /// Table 15-2, p. 420) is written when the RTC is started.
    pub fn new(rtc: _pac::Rtc) -> Self {
        Rtc { periph: rtc, _src: PhantomData }
    }
}

// RTCPS predivider settings (SLAU445I Table 15-2, p. 420)
pub use crate::_pac::rtc::rtcctl::Rtcps as RtcDiv;

impl<SRC: RtcClockSrc> Rtc<SRC> {
    /// Configure the RTC to use SMCLK as clock source. Setting comes in effect the next time RTC
    /// is started (RTCSS, SLAU445I Table 15-2, p. 420).
    #[inline]
    pub fn use_smclk(self, _smclk: &Smclk) -> Rtc<RtcSmclk> {
        Rtc { periph: self.periph, _src: PhantomData }
    }

    /// Configure the RTC to use ACLK as clock source. Setting comes in effect the next time RTC
    /// is started (RTCSS, SLAU445I Table 15-2, p. 420; RTCCKSEL, SLAU445I Table 1-26, p. 77;
    /// SLAU445I Table 1-31, p. 82).
    #[inline]
    #[cfg(feature = "rtc_aclk")]
    pub fn use_aclk(self, _aclk: &Aclk) -> Rtc<RtcAclk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Configure the RTC to use VLOCLK as clock source. Setting comes in effect the next time RTC
    /// is started (RTCSS, SLAU445I Table 15-2, p. 420).
    #[inline]
    pub fn use_vloclk(self) -> Rtc<RtcVloclk> {
        Rtc { periph: self.periph, _src: PhantomData }
    }

    /// Configure the RTC to use XT1CLK as clock source. Setting comes in effect the next time RTC
    /// is started (RTCSS, SLAU445I Table 15-2, p. 420).
    ///
    /// XT1 must run in low-frequency mode: the RTC's XT1CLK input only carries a 32 kHz XT1
    /// (SLAU445I 15.1, p. 416: "XT1CLK (approximately 32 kHz)"; device data sheets, clock distribution:
    /// SLASEC4D Table 6-10, p. 68, which gives the RTC no XTHFCLK; SLASE59F Table 6-7, p. 46, SLASEO7C
    /// Table 9-8, p. 50 and SLASEE4C Table 6-8, p. 49: XT1CLK "DC to 40 kHz").
    #[inline]
    pub fn use_xt1clk(self, _xt1clk: &Xt1clk) -> Rtc<RtcXt1clk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Set RTC clock frequency divider (RTCPS, SLAU445I Table 15-2, p. 420)
    #[inline]
    pub fn set_clk_div(&mut self, div: RtcDiv) {
        self.periph.rtcctl().modify(|_, w| w.rtcps().variant(div));
    }

    /// Enable RTC timer interrupts (RTCIE: SLAU445I Table 15-2, p. 420). An overflow from before is
    /// cleared first, so it doesn't fire at once (SLAU445I 15.2.4, p. 418: "TI recommends clearing the
    /// RTCIFG bit by reading the RTCIV register before enabling the RTC counter interrupt").
    #[inline]
    pub fn enable_interrupts(&mut self) {
        self.periph.rtciv().read();
        unsafe { self.periph.rtcctl().set_bits(|w| w.rtcie().set_bit()) };
    }

    /// Disable RTC timer interrupts (RTCIE: SLAU445I Table 15-2, p. 420)
    #[inline]
    pub fn disable_interrupts(&mut self) {
        unsafe { self.periph.rtcctl().clear_bits(|w| w.rtcie().clear_bit()) };
    }

    /// Clear interrupt flag (reading RTCIV clears RTCIFG: SLAU445I 15.2.4, p. 418; RTCIV: SLAU445I
    /// Table 15-3, p. 421)
    #[inline]
    pub fn clear_interrupt(&mut self) { self.periph.rtciv().read(); }

    /// Read current timer count, which goes up from 0 to the `count` given to `start()`, at most 2^16-1
    /// (SLAU445I 15.2.1, p. 417; RTCCNT: SLAU445I Table 15-5, p. 422)
    #[inline]
    pub fn get_count(&self) -> u16 { self.periph.rtccnt().read().bits() }

    #[inline]
    /// Clear the timer contents and start the timer counting up to `count`. The counter wraps to
    /// zero after reaching `count`, so a period lasts `count + 1` ticks of the divided clock (SLAU445I
    /// 15.2.1, p. 417; SLAU445I Figure 15-2, p. 418). A `count` of 0 or 1 is the exception: "RTC counter
    /// always generates an overflow when the RTCMOD is set to either 0x0000 or 0x0001" (SLAU445I 15.2.3,
    /// p. 418). `count` goes to RTCMOD (SLAU445I Table 15-4, p. 422), and RTCSR resets the counter
    /// (SLAU445I Table 15-2, p. 420).
    ///
    /// Erratum RTC15 on the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2: moving the RTC off XT1CLK while
    /// XT1 is stopped makes it hang (SLAZ695J RTC15, p. 11; SLAZ664S RTC15, p. 13; SLAZ705H RTC15, p. 10).
    /// When `start()` moves the RTC off XT1CLK with the XT1 fault flag XT1OFFG set, it clears the flag,
    /// which stays set after a fault has ended, and only if the hardware sets it again because the fault
    /// persists (SLAU445I 3.2.13, p. 109) does it apply the erratum's workaround: XIN becomes a GPIO
    /// output, toggles, and goes back to XT1. After a fault that has ended, the flag stays clear, so
    /// [`Xt1clk::is_faulted`](crate::clock::Xt1clk::is_faulted) returns `false` from then on. OFIFG stays
    /// set, so the clocks the fail-safe moved off XT1 stay on their fallback until
    /// [`Xt1clk::clear_fault`](crate::clock::Xt1clk::clear_fault) clears it (SLAU445I 3.2.13, p. 110,
    /// "Fault logic"). XT1 counts as faulted until its fault logic counter has reached its maximum count
    /// after the oscillation resumed (SLAU445I 3.2.13, p. 110, "Fault logic counters"; see
    /// `Xt1clk::clear_fault`), so after a fault that ended only just before, XIN is toggled although XT1
    /// runs again: in bypass mode the pin then drives against the external clock while it toggles.
    pub fn start(&mut self, count: u16) {
        self.periph.rtcmod().write(|w| unsafe { w.bits(count) });
        SRC::apply_sys_config();
        // Erratum RTC15: moving the RTC off XT1CLK while XT1 is stopped hangs it (SLAZ695J RTC15, p. 11;
        // SLAZ664S RTC15, p. 13; SLAZ705H RTC15, p. 10). XT1CLK is RTCSS = 10b (SLAU445I Table 15-2,
        // p. 420). The new clock is tested first: it is known when compiling, so an RTC started on XT1CLK
        // reads nothing here.
        #[cfg(feature = "erratum_rtc15")]
        let leaving_stopped_xt1 = SRC::CLK_SRC != Rtcss::Xt1clk
            && self.periph.rtcctl().read().rtcss().is_xt1clk()
            && xt1_stopped();
        // Select the clock first, then reset the counter, which also loads `count` into the
        // shadow register (SLAU445I 15.2.3, p. 417). The reset resynchronizes the count with the new
        // clock (SLAU445I 15.2.2, p. 417, note "Clock Source Selection": "TI recommends a software reset
        // by asserting the RTCSR bit after the RTC clock source is switched").
        self.periph.rtcctl().modify(|_, w| w.rtcss().variant(SRC::CLK_SRC));
        #[cfg(feature = "erratum_rtc15")]
        if leaving_stopped_xt1 {
            pulse_xin();
        }
        self.periph.rtcctl().modify(|_, w| w.rtcsr().set_bit());
        // Clear the interrupt flag from the last timer run, and any raised while switching clocks
        // (SLAU445I 15.2.2, p. 417: "An unexpected interrupt may happen during the clock source change";
        // reading RTCIV clears RTCIFG: SLAU445I 15.2.4, p. 418)
        self.periph.rtciv().read();
    }

    #[inline]
    /// Checks if the timer has reached the target value, returns `Ok(())` if so, otherwise `WouldBlock`.
    /// The overflow sets RTCIFG (SLAU445I Table 15-2, p. 420; SLAU445I 15.2.4, p. 418).
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        // RTCIFG is set on each overflow, and reading RTCIV clears it (SLAU445I 15.2.4, p. 418)
        if self.periph.rtcctl().read().rtcifg().bit() {
            self.periph.rtciv().read();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    /// Pauses the timer by selecting no clock (RTCSS = 00b: SLAU445I Table 15-2, p. 420).
    ///
    /// `pause()` doesn't apply the workaround for erratum RTC15 that [`start()`](Rtc::start) applies on
    /// the MSP430FR2x5x, MSP430FR2433 and MSP430FR25x2: the erratum names a change "from XT1CLK to a
    /// different clock source while XT1CLK is stopped" (SLAZ695J RTC15, p. 11; SLAZ664S RTC15, p. 13;
    /// SLAZ705H RTC15, p. 10). To move the RTC off a stopped XT1CLK, start it with the new clock instead.
    pub fn pause(&mut self) {
        // Bit pattern is all 0s, so we can use clear instead of modify (RTCSS = 00b, "No clock (Stop)":
        // SLAU445I Table 15-2, p. 420)
        unsafe {
            self.periph.rtcctl().clear_bits(|w| w
                .rtcss().variant(Rtcss::Disabled))
        };
    }

    #[inline]
    /// Resumes counting from the previous value (the counter runs while RTCSS selects an active clock:
    /// SLAU445I 15.2.1, p. 417).
    pub fn resume(&mut self) {
        unsafe { self.periph.rtcctl().set_bits(|w| w.rtcss().variant(SRC::CLK_SRC)) }
    }
}

/// Whether XT1 is stopped, for erratum RTC15. XT1OFFG reports an XT1 fault, but "Once set, the fault bits
/// remain set until software resets them, even if the fault condition no longer exists", so a set flag is
/// cleared and read again: "If software clears the fault bits and the fault condition still exists, the
/// fault bits are automatically set again; otherwise, they remain cleared" (SLAU445I 3.2.13, p. 109; "If
/// the user clears XT1OFFG and the fault condition still exists, XT1OFFG remains set": SLAU445I
/// Figure 3-6, p. 110; XT1OFFG: SLAU445I Table 3-11, p. 122). OFIFG is left as it is: while it is set, the
/// clocks the fail-safe moved off XT1 stay on their fallback (SLAU445I 3.2.13, p. 110, "Fault logic").
#[cfg(feature = "erratum_rtc15")]
fn xt1_stopped() -> bool {
    let cs = unsafe { _pac::Cs::steal() };
    if cs.csctl7().read().xt1offg().bit_is_clear() {
        return false;
    }
    // Clearing keeps the bits set in the mask, so only XT1OFFG is written 0
    unsafe { cs.csctl7().clear_bits(|w| w.xt1offg().clear_bit()) };
    cs.csctl7().read().xt1offg().bit_is_set()
}

/// The workaround for erratum RTC15, after the RTC was moved off a stopped XT1CLK: "Reconfigure the XIN
/// pin as a GPIO output, then toggle the GPIO twice with at least 2 rising or falling edges. At this
/// point the RTC Counter will be able to resume operation" (SLAZ695J RTC15, p. 11; SLAZ664S RTC15,
/// p. 13; SLAZ705H RTC15, p. 10). It toggles four times, for two rising and two falling edges, and then
/// gives the pin its direction and function back. GPIO output: PxSEL1 = PxSEL0 = 0, PxDIR = 1 (SLAU445I
/// Table 8-1, p. 313; SLAU445I Table 8-3, p. 314).
#[cfg(feature = "erratum_rtc15")]
fn pulse_xin() {
    use crate::clock::Xt1Xin;
    use crate::gpio::AlternatePin;
    use crate::hw_traits::gpio::GpioPeriph;

    let port = unsafe { <Xt1Xin<()> as AlternatePin>::Port::steal() };
    let mask = <Xt1Xin<()> as AlternatePin>::MASK;
    let (sel0, sel1, dir) = (port.pxsel0_rd() & mask, port.pxsel1_rd() & mask, port.pxdir_rd() & mask);

    // The _clear methods keep the bits set in their argument
    port.pxsel0_clear(!mask);
    port.pxsel1_clear(!mask);
    port.pxdir_set(mask);
    for _ in 0..4 {
        port.pxout_toggle(mask);
    }

    if dir == 0 {
        port.pxdir_clear(!mask);
    }
    // The XIN function is a single PxSEL bit, PxSEL0 or PxSEL1 (SLASEC4D Table 6-64, p. 98; SLASE59F
    // Table 6-18, p. 56; SLASEE4C Table 6-16, p. 60), so setting it back passes through no other function
    if sel0 != 0 {
        port.pxsel0_set(mask);
    }
    if sel1 != 0 {
        port.pxsel1_set(mask);
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::timer::{Cancel, CountDown, Periodic};
    use void::Void;

    impl<SRC: RtcClockSrc> CountDown for Rtc<SRC> {
        type Time = u16;

        #[inline]
        fn start<T: Into<Self::Time>>(&mut self, count: T) { self.start(count.into()) }

        #[inline]
        fn wait(&mut self) -> nb::Result<(), Void> {
            self.wait().map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl<SRC: RtcClockSrc> Cancel for Rtc<SRC> {
        type Error = Void;

        #[inline]
        fn cancel(&mut self) -> Result<(), Self::Error> {
            self.pause();
            Ok(())
        }
    }

    impl<SRC: RtcClockSrc> Periodic for Rtc<SRC> {}
}
