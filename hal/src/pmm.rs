//! Power management module
//!
//! Besides the internal voltage reference and temperature sensor, [`Pmm`] reports why the device
//! reset ([`Pmm::take_reset_cause()`]), triggers software resets and controls the high-side supply
//! voltage supervisor (SVSH). The PMM is described in SLAU445I chapter 2, p. 84.

use core::marker::PhantomData;

use crate::{_pac, info_mem::InfoMemory, lpm::SvsState};

/// PMM type
pub struct Pmm(_pac::Pmm);

/// Struct indicating that the internal voltage reference has been enabled and configured.
/// This can be passed to the ADC to read the reference voltage.
#[derive(Debug)]
pub struct InternalVRef(ReferenceVoltage);
impl InternalVRef {
    /// Get the requested internal reference voltage
    pub fn voltage(&self) -> ReferenceVoltage { self.0 }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A list of possible internal reference voltages
pub enum ReferenceVoltage {
    /// 1.5V
    _1V5 = 0b00,

    #[cfg(feature = "enhanced_ref")]
    /// 2.0V
    _2V0 = 0b01,

    #[cfg(feature = "enhanced_ref")]
    /// 2.5V
    _2V5 = 0b10,
}

/// Token indicating that the internal temperature sensor has been enabled.
/// This can be passed to the ADC to read the temperature sensor voltage.
#[derive(Debug)]
pub struct InternalTempSensor<'a>(PhantomData<&'a InternalVRef>);

/// Marker trait for the VREF+ pin in its analog mode, which can output the 1.2 V reference. The output
/// only works with the pin in its ADC function (SLAU445I 2.2.8, p. 89).
pub trait VrefOutputPin {}

/// The 1.2 V reference output on the VREF+ pin, see [`Pmm::enable_vref_output()`]. Pass it to
/// [`Adc::read_count()`](crate::adc::Adc::read_count) to measure it with the pin's ADC channel.
pub struct VrefOutput<PIN>(pub(crate) PIN);

/// A reason for a reset, in priority order (SYSRSTIV, SLAU445I 1.3.7, p. 36). A brownout reset (BOR)
/// resets the most, then a power-on reset (POR), then a power-up clear (PUC) (SLAU445I 1.2, p. 30).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetCause {
    /// Power-up, or the supply dropped below the brownout level (BOR)
    Brownout,
    /// A low level on the RST/NMI pin (BOR)
    ResetPin,
    /// [`Pmm::trigger_bor_reset()`] (BOR)
    SoftwareBor,
    /// A wake-up from LPM3.5 or LPM4.5 (BOR)
    Lpmx5WakeUp,
    /// A security violation (BOR)
    SecurityViolation,
    /// The supply dropped below the high-side SVS level (BOR)
    Svsh,
    /// [`Pmm::trigger_por_reset()`] (POR)
    SoftwarePor,
    /// The watchdog timed out (PUC)
    WatchdogTimeout,
    /// A write to the watchdog without its password (PUC)
    WatchdogPassword,
    /// A write to the FRAM controller without its password (PUC)
    FramPassword,
    /// The FRAM detected a bit error it couldn't correct (PUC)
    FramBitError,
    /// The CPU fetched an instruction from the peripheral area (PUC)
    PeripheralAreaFetch,
    /// A write to the PMM without its password (PUC)
    PmmPassword,
    /// The DCO ran too fast for the FLL (PUC)
    FllUnlock,
    /// A value the data sheets list as reserved
    Reserved(u16),
}

impl Pmm {
    /// Clears the LOCKLPM5 bit, so the I/O pins take on their configured state (SLAU445I 8.3.1,
    /// p. 316), and returns a `Pmm` (and an `InfoMemory`).
    ///
    /// The data sheets ask for the ports to be configured before LOCKLPM5 is cleared. To follow that
    /// order, use [`Pmm::new_locked`], configure the ports, then call [`Pmm::unlock_lpm5`]. With
    /// `Pmm::new` the pins are released first, in their reset state.
    pub fn new(pmm: _pac::Pmm, sys: _pac::Sys) -> (Pmm, InfoMemory) {
        let mut pmm = Pmm(pmm);
        pmm.unlock_lpm5();
        (pmm, InfoMemory::new(sys))
    }

    /// Like [`Pmm::new`], but leaves the LOCKLPM5 bit set. Configure the GPIO pins, then call
    /// [`Pmm::unlock_lpm5`]: the ports must be configured first and then LOCKLPM5 cleared. After LOCKLPM5
    /// is cleared, "all interrupt flags should be cleared", and only then port interrupts enabled
    /// (SLAU445I 8.3.1, p. 316).
    ///
    /// The order is right after any reset, so the reset cause needn't be checked first: where the pins
    /// aren't locked, `unlock_lpm5` changes nothing. Measured on an MSP430FR2476: a software BOR sets
    /// LOCKLPM5 again, while a software POR and a watchdog PUC leave it as software left it.
    ///
    /// After a wake-up from LPMx.5 the I/O pins keep the configuration they had while asleep until
    /// LOCKLPM5 is cleared, while their registers start out reset (SLAU445I 1.4.3.2, p. 42). Configuring
    /// the pins before clearing LOCKLPM5 lets them carry on without a glitch.
    pub fn new_locked(pmm: _pac::Pmm, sys: _pac::Sys) -> (Pmm, InfoMemory) {
        (Pmm(pmm), InfoMemory::new(sys))
    }

    /// Clears the LOCKLPM5 bit, so the I/O pins take on their configured state (SLAU445I 8.3.1,
    /// p. 316). Only needed after [`Pmm::new_locked`]; [`Pmm::new`] already does this.
    pub fn unlock_lpm5(&mut self) {
        // PM5CTL0 needs no PMM password (SLAU445I 2.3, p. 90)
        self.0.pm5ctl0().write(|w| w.locklpm5().clear_bit());
    }

    /// Returns the highest-priority reason for a reset that hasn't been read yet and clears it
    /// (SYSRSTIV, SLAU445I 1.3.7, p. 36), or `None` once all have been read.
    ///
    /// The reasons accumulate until they are read, so a reset can have several, for example a
    /// brownout at power-up and a later watchdog timeout. Call this until it returns `None` to see
    /// them all.
    ///
    /// A debugger can start the program without a reset, after flashing it for example, and then
    /// there may be no reason at all. On the MSP430FR2433, a PUC for a FRAM bit error that doesn't
    /// exist leaves no reason either (SLAZ664S GC4), see [`fram`](crate::fram).
    pub fn take_reset_cause(&mut self) -> Option<ResetCause> {
        let sys = unsafe { _pac::Sys::steal() };
        match sys.sysrstiv().read().bits() {
            0x00 => None,
            0x02 => Some(ResetCause::Brownout),
            0x04 => Some(ResetCause::ResetPin),
            0x06 => Some(ResetCause::SoftwareBor),
            0x08 => Some(ResetCause::Lpmx5WakeUp),
            0x0A => Some(ResetCause::SecurityViolation),
            0x0E => Some(ResetCause::Svsh),
            0x14 => Some(ResetCause::SoftwarePor),
            0x16 => Some(ResetCause::WatchdogTimeout),
            0x18 => Some(ResetCause::WatchdogPassword),
            0x1A => Some(ResetCause::FramPassword),
            0x1C => Some(ResetCause::FramBitError),
            0x1E => Some(ResetCause::PeripheralAreaFetch),
            0x20 => Some(ResetCause::PmmPassword),
            0x24 => Some(ResetCause::FllUnlock),
            other => Some(ResetCause::Reserved(other)),
        }
    }

    /// Reset the device with a brownout reset (BOR), the reset of a power-up (PMMSWBOR, SLAU445I
    /// Table 2-2, p. 91). [`Pmm::take_reset_cause()`] then returns [`ResetCause::SoftwareBor`].
    ///
    /// Use this to restart on purpose, for example when the program detects a state it can't recover
    /// from. Nothing else reports this reset cause, so after the reset the program can tell that it reset
    /// itself. To also know why, store an error code in [backup memory](crate::bak_mem) before the call,
    /// or in [information memory](crate::info_mem) if it has to survive a power loss.
    pub fn trigger_bor_reset() -> ! {
        // Nothing can use the PMM after the reset, so it is safe to take its registers here
        let pmm = unsafe { _pac::Pmm::steal() };
        pmm.pmmctl0().modify(|_, w| w.pmmpw().password().pmmswbor().set_bit());
        // The reset happens on the write above, so this is never reached
        #[allow(clippy::empty_loop)]
        loop {}
    }

    /// Reset the device with a power-on reset (POR), which resets less than a brownout reset
    /// (PMMSWPOR, SLAU445I Table 2-2, p. 91). [`Pmm::take_reset_cause()`] then returns
    /// [`ResetCause::SoftwarePor`].
    pub fn trigger_por_reset() -> ! {
        // Nothing can use the PMM after the reset, so it is safe to take its registers here
        let pmm = unsafe { _pac::Pmm::steal() };
        pmm.pmmctl0().modify(|_, w| w.pmmpw().password().pmmswpor().set_bit());
        // The reset happens on the write above, so this is never reached
        #[allow(clippy::empty_loop)]
        loop {}
    }

    /// Whether the high-side supply voltage supervisor (SVSH) stays on in LPM2, LPM3 and LPM4, as
    /// after reset (PMMCTL0.SVSHE, SLAU445I Table 2-2, p. 91). It is always on in active mode, LPM0
    /// and LPM1. Turning it off saves power in the low-power modes, but a supply drop there then
    /// resets the device only once it reaches the brownout level (SLAU445I 2.2.4, p. 87). For LPM3.5 and
    /// LPM4.5, `enter_lpm3_5()` and `enter_lpm4_5()` set this.
    pub fn set_voltage_supervisor_in_deep_lpm(&mut self, svs: SvsState) {
        self.unlocked(|pmm| pmm.pmmctl0().modify(|_, w| w.pmmpw().password().svshe().variant(svs)));
    }

    /// Run `f` with write access to the PMM registers, and lock them again afterwards.
    ///
    /// Writing a PMM register other than PMMCTL0 while they are locked causes a PUC, and so does a
    /// word write of a wrong password, so they are locked again with a byte write to the upper
    /// byte of PMMCTL0 (SLAU445I 2.3, p. 90). Interrupts are disabled meanwhile, so an interrupt handler
    /// can't lock them halfway.
    #[inline]
    fn unlocked<R>(&mut self, f: impl FnOnce(&_pac::Pmm) -> R) -> R {
        critical_section::with(|_| {
            self.0.pmmctl0().modify(|_, w| w.pmmpw().password());
            let ret = f(&self.0);
            // PMMCTL0_H is the byte at offset 01h. The PACs don't have a `pmmctl0_h` register yet, so for
            // now this is a raw byte write of 0 to offset 1 of the little-endian PMMCTL0. This can use
            // `pmmctl0_h().write(|w| w.pmmpw().lock())` once the PACs all have it.
            unsafe { (self.0.pmmctl0().as_ptr() as *mut u8).add(1).write_volatile(0) };
            ret
        })
    }

    /// Configures the internal voltage reference to the specified voltage and enables it.
    /// Returns a token signifying that the voltage reference has been enabled, unless it was *already* enabled.
    ///
    /// Waits until the reference has settled (REFGENRDY), as the user's guide recommends (SLAU445I
    /// Table 2-4, note 1, p. 93: "TI recommends checking this bit before using the reference").
    pub fn enable_internal_reference(&mut self, vref: ReferenceVoltage) -> Option<InternalVRef> {
        if self.0.pmmctl2().read().intrefen().bit() {
            return None;
        }
        self.unlocked(|pmm| pmm.pmmctl2().modify(|_, w| unsafe { w
            .refvsel().bits(vref as u8)
            .intrefen().set_bit()
        }));
        while self.0.pmmctl2().read().refgenrdy().bit_is_clear() {}
        Some(InternalVRef(vref))
    }

    /// Disables the internal reference voltage
    pub fn disable_internal_reference(&mut self, _vref: InternalVRef) {
        self.unlocked(|pmm| unsafe { pmm.pmmctl2().clear_bits(|w| w.intrefen().clear_bit()) });
    }

    /// Enables the internal temperature sensor.
    /// Returns a token signifying that the temp sensor has been enabled, unless it was *already* enabled.
    pub fn enable_internal_temp_sensor<'a>(
        &mut self,
        _vref: &'a InternalVRef,
    ) -> Option<InternalTempSensor<'a>> {
        match self.0.pmmctl2().read().tsensoren().bit() {
            true  => None,
            false => {
                self.unlocked(|pmm| unsafe { pmm.pmmctl2().set_bits(|w| w.tsensoren().set_bit()) });
                Some(InternalTempSensor(PhantomData))
            }
        }
    }

    /// Disables the internal temperature sensor
    pub fn disable_internal_temp_sensor(&mut self, _tsense: InternalTempSensor) {
        self.unlocked(|pmm| unsafe { pmm.pmmctl2().clear_bits(|w| w.tsensoren().clear_bit()) });
    }

    /// Output the 1.2 V reference on the VREF+ pin, buffered. It can supply up to 1 mA. The 1.5 V, 2.0 V
    /// and 2.5 V internal shared reference can't be output (SLAU445I 2.2.8, p. 88).
    ///
    /// The output is the buffered bandgap (EXTREFEN and REFBGEN in PMMCTL2: SLAU445I Table 2-4, p. 93 to
    /// p. 94). This function starts it and waits until it is ready (REFBGRDY). Measured on an MSP430FR2476:
    /// with EXTREFEN alone neither REFBGACT nor REFBGRDY is set, while with REFBGEN both are within a few
    /// register reads.
    pub fn enable_vref_output<PIN: VrefOutputPin>(&mut self, pin: PIN) -> VrefOutput<PIN> {
        self.unlocked(|pmm| unsafe {
            pmm.pmmctl2().set_bits(|w| w.extrefen().set_bit().refbgen().set_bit())
        });
        while self.0.pmmctl2().read().refbgrdy().bit_is_clear() {}
        VrefOutput(pin)
    }

    /// Stop outputting the 1.2 V reference, and return the pin (clears EXTREFEN and REFBGEN, which the
    /// hardware may have cleared already: SLAU445I Table 2-4, p. 93 to p. 94).
    pub fn disable_vref_output<PIN>(&mut self, output: VrefOutput<PIN>) -> PIN {
        self.unlocked(|pmm| unsafe {
            pmm.pmmctl2().clear_bits(|w| w.extrefen().clear_bit().refbgen().clear_bit())
        });
        output.0
    }
}
