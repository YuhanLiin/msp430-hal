//! Power management module
//!
//! Besides the internal voltage reference and temperature sensor, [`Pmm`] reports why the device
//! reset ([`Pmm::take_reset_cause()`]), triggers software resets and controls the high-side supply
//! voltage supervisor (SVSH).
//!
//! The PMM is described in SLAU445I chapter 2: the supply voltage supervisor in SLAU445I 2.2.2, p. 86, the
//! software BOR and POR in SLAU445I 2.2.6, p. 88, the shared reference in SLAU445I 2.2.8, p. 88 and the
//! temperature sensor in SLAU445I 2.2.9, p. 89. The reset vector register SYSRSTIV is a SYS register
//! (SLAU445I 1.15.10, p. 72).

use core::marker::PhantomData;

use crate::{_pac, info_mem::InfoMemory, lpm::SvsState};

/// PMM type (the PMM registers: SLAU445I Table 2-1, p. 90)
pub struct Pmm(_pac::Pmm);

/// How the LPM3.5 switch is controlled, which connects the LPM3.5 domain to the core supply
/// (PM5CTL0.LPM5SM and LPM5SW: SLAU445I 2.2.7, p. 88; SLAU445I Table 2-7, p. 97). The domain holds the RTC
/// counter and the backup memory (SLASE59F Figure 1-1, p. 3; SLASEE4C Figure 1-1, p. 4). Only on the
/// MSP430FR2433 and MSP430FR25x2: "Only available in the FR203x, FR211x, FR2100, FR2000, FR231x, FR2433,
/// FR2422, FR263x, FR253x, FR252x, and FR413x devices" (SLAU445I Table 2-7, p. 97).
#[cfg(feature = "lpm3_5_switch")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Lpm3_5Switch {
    /// The PMM connects and disconnects it, as after a BOR: "This is the recommended mode for general
    /// operation" (LPM5SM = 0)
    Automatic,
    /// Connected under software control: the domain "can accept full-speed read and write operation by
    /// the CPU MCLK" (LPM5SM = 1, LPM5SW = 1)
    Connected,
    /// Disconnected under software control: "all peripherals within this domain can accept clock operation
    /// no faster than 40 kHz" (LPM5SM = 1, LPM5SW = 0)
    Disconnected,
}

/// Struct indicating that the internal voltage reference has been enabled and configured.
/// This can be passed to the ADC to read the reference voltage, which is internally connected to an
/// ADC channel (SLAU445I 2.2.8, p. 88).
#[derive(Debug)]
pub struct InternalVRef(ReferenceVoltage);
impl InternalVRef {
    /// Get the requested internal reference voltage
    pub fn voltage(&self) -> ReferenceVoltage { self.0 }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A list of possible internal reference voltages (PMMCTL2.REFVSEL, SLAU445I Table 2-4, p. 93; 2.0 V
/// and 2.5 V only in enhanced shared reference systems)
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
/// This can be passed to the ADC to read the temperature sensor voltage, which is internally
/// connected to an ADC channel (SLAU445I 2.2.9, p. 89).
#[derive(Debug)]
pub struct InternalTempSensor<'a>(PhantomData<&'a InternalVRef>);

/// Marker trait for the VREF+ pin in its analog mode, which can output the 1.2 V reference: P1.7 on the
/// MSP430FR2x5x, P1.4 on the MSP430FR247x and MSP430FR2433, P1.1 on the MSP430FR25x2 (SLASEC4D 6.10.1,
/// p. 67; SLASEO7C 9.10.1, p. 49; SLASE59F 6.10.1, p. 45; SLASEE4C 6.10.1, p. 49). The output only works
/// with the pin in its ADC function (SLAU445I 2.2.8, p. 89).
pub trait VrefOutputPin {}

/// The 1.2 V reference output on the VREF+ pin, see [`Pmm::enable_vref_output()`]. Pass it to
/// [`Adc::read_count()`](crate::adc::Adc::read_count) to measure it with the pin's ADC channel (SLAU445I
/// 2.2.8, p. 89).
pub struct VrefOutput<PIN>(pub(crate) PIN);

/// A reason for a reset, in priority order (SYSRSTIV: SLASEC4D Table 6-12, p. 70; SLASE59F Table 6-9,
/// p. 48; SLASEO7C Table 9-10, p. 52; SLASEE4C Table 6-10, p. 52). A brownout reset (BOR) resets the
/// most, then a power-on reset (POR), then a power-up clear (PUC) (SLAU445I 1.2, p. 30).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetCause {
    /// Power-up, or the supply dropped below the brownout level (BOR, SYSRSTIV 02h; SLAU445I 2.2.6, p. 88)
    Brownout,
    /// A low level on the RST/NMI pin (BOR, SYSRSTIV 04h; SLAU445I 1.2, p. 30)
    ResetPin,
    /// [`Pmm::software_bor()`] (BOR, SYSRSTIV 06h; PMMSWBOR in SLAU445I Table 2-2, p. 91)
    SoftwareBor,
    /// A wake-up from LPM3.5 or LPM4.5 (BOR, SYSRSTIV 08h; SLAU445I 1.4.3.2, p. 42)
    Lpmx5WakeUp,
    /// A security violation (BOR, SYSRSTIV 0Ah; SLASEC4D Table 6-12, p. 70). Measured on an MSP430FR2476, a
    /// read of the RAM assigned to the protected BSL causes one, see
    /// [`Bsl::set_ram_assigned()`](crate::sys::Bsl::set_ram_assigned).
    SecurityViolation,
    /// The supply dropped below the high-side SVS level (BOR, SYSRSTIV 0Eh; SVSHIFG in SLAU445I Table 2-6,
    /// p. 96)
    Svsh,
    /// [`Pmm::software_por()`] (POR, SYSRSTIV 14h; PMMSWPOR in SLAU445I Table 2-2, p. 91)
    SoftwarePor,
    /// The watchdog timed out (PUC, SYSRSTIV 16h; SLAU445I 1.2, p. 30)
    WatchdogTimeout,
    /// A write to the watchdog without its password (PUC, SYSRSTIV 18h; SLAU445I 1.2, p. 30)
    WatchdogPassword,
    /// A write to the FRAM controller without its password (PUC, SYSRSTIV 1Ah; SLAU445I 1.2, p. 30)
    FramPassword,
    /// The FRAM detected a bit error it couldn't correct, see
    /// [`Fram::set_uncorrectable_bit_error_action()`](crate::fram::Fram::set_uncorrectable_bit_error_action)
    /// (PUC, SYSRSTIV 1Ch; GCCTL0.UBDRSTEN in SLAU445I Table 6-3, p. 307)
    FramBitError,
    /// The CPU fetched an instruction from the peripheral area (PUC, SYSRSTIV 1Eh; SLAU445I 1.2, p. 30)
    PeripheralAreaFetch,
    /// A write to the PMM without its password (PUC, SYSRSTIV 20h; SLAU445I 2.3, p. 90)
    PmmPassword,
    /// The DCO ran too fast for the FLL, see
    /// [`ClockConfig::reset_on_fll_unlock()`](crate::clock::ClockConfig::reset_on_fll_unlock) (PUC,
    /// SYSRSTIV 24h; CSCTL7.FLLULPUC in SLAU445I Table 3-11, p. 121: FLLUNLOCK = 10b, "too fast")
    FllUnlock,
    /// A value the data sheets list as reserved (SLASEC4D Table 6-12, p. 70; SLASE59F Table 6-9, p. 48;
    /// SLASEO7C Table 9-10, p. 52; SLASEE4C Table 6-10, p. 52)
    Reserved(u16),
}

impl Pmm {
    /// Clears the LOCKLPM5 bit, so the I/O pins take on their configured state (SLAU445I 8.3.1,
    /// p. 316), and returns a `Pmm` (and an `InfoMemory`).
    ///
    /// The data sheets ask for the ports to be configured before LOCKLPM5 is cleared: "To enable the I/O
    /// functions after a BOR reset, the ports must be configured first and then the LOCKLPM5 bit must be
    /// cleared" (SLASE59F 6.10.3, p. 46; SLASEO7C 9.10.3, p. 51; SLASEE4C 6.10.3, p. 51; SLASEC4D 6.10.3,
    /// p. 69). To follow that order, use [`Pmm::new_locked`], configure the ports, then call
    /// [`Pmm::unlock_lpm5`]. With `Pmm::new` the pins are released first, in their reset state.
    pub fn new(pmm: _pac::Pmm, sys: _pac::Sys) -> (Pmm, InfoMemory) {
        let mut pmm = Pmm(pmm);
        pmm.unlock_lpm5();
        (pmm, InfoMemory::new(sys))
    }

    /// Like [`Pmm::new`], but leaves the LOCKLPM5 bit set. Configure the GPIO pins, then call
    /// [`Pmm::unlock_lpm5`]: "the ports must be configured first and then the LOCKLPM5 bit must be
    /// cleared" (SLASE59F 6.10.3, p. 46; SLASEO7C 9.10.3, p. 51; SLASEE4C 6.10.3, p. 51; SLASEC4D 6.10.3,
    /// p. 69). After LOCKLPM5 is cleared, "all interrupt flags should be cleared", and only then port
    /// interrupts enabled (SLAU445I 8.3.1, p. 316).
    ///
    /// The order is right after any reset, so the reset cause needn't be checked first: where the pins
    /// aren't locked, `unlock_lpm5` changes nothing. LOCKLPM5 resets to 1 and is "reset by a power cycle"
    /// (SLAU445I Table 2-7, p. 97). Measured on an MSP430FR2476: a software BOR sets it again, while a
    /// software POR and a watchdog PUC leave it as software left it.
    ///
    /// After a wake-up from LPMx.5 the I/O pins, and XT1 if it clocked the RTC, keep the
    /// configuration they had while asleep until LOCKLPM5 is cleared, while their registers
    /// start out reset (SLAU445I 1.4.3.2, p. 42 and SLAU445I 8.3.3, p. 318; for XT1, XT1DRIVE in
    /// SLAU445I Table 3-10, p. 119). Configuring the pins, and XT1 through
    /// [`ClockConfig`](crate::clock::ClockConfig), before clearing LOCKLPM5 lets them carry on
    /// without a glitch (SLAU445I 1.4.3.3, p. 42). With [`Pmm::new`], clearing LOCKLPM5 first would
    /// stop XT1, and the RTC with it, until XT1 is reconfigured (SLAU445I Table 3-10, p. 119:
    /// "reconfiguration is required after wakeup from LPM3.5 and before clearing LOCKLPM5").
    ///
    /// After a cold start the locked pins are held in their power-on state, so XT1 cannot
    /// start before LOCKLPM5 is cleared (SLAU445I 8.3.1, p. 316 and SLAU445I 1.4.3.4, p. 42): call
    /// [`Pmm::unlock_lpm5`] before configuring a clock with XT1 then.
    pub fn new_locked(pmm: _pac::Pmm, sys: _pac::Sys) -> (Pmm, InfoMemory) {
        (Pmm(pmm), InfoMemory::new(sys))
    }

    /// Clears the LOCKLPM5 bit, so the I/O pins take on their configured state (SLAU445I 8.3.1,
    /// p. 316). Only needed after [`Pmm::new_locked`]; [`Pmm::new`] already does this.
    pub fn unlock_lpm5(&mut self) {
        // PM5CTL0 needs no PMM password (SLAU445I 2.3, p. 90; LOCKLPM5 in SLAU445I Table 2-7, p. 97)
        self.0.pm5ctl0().write(|w| w.locklpm5().clear_bit());
    }

    /// Returns the highest-priority reason for a reset that hasn't been read yet and clears it
    /// (SYSRSTIV, SLAU445I 1.3.7, p. 36), or `None` once all have been read.
    ///
    /// The reasons accumulate until they are read, so a reset can have several, for example a
    /// brownout at power-up and a later watchdog timeout (SLAU445I 1.3.7, p. 36; the PMMIFG flags
    /// are cleared "by reading the reset vector word", SLAU445I Table 2-6, p. 96). Call this until it returns
    /// `None` to see them all. Reading a wake-up from LPMx.5 also clears PMMLPM5IFG (SLAU445I
    /// Table 2-6, p. 96).
    ///
    /// A debugger can start the program without a reset, after flashing it for example, and then
    /// there may be no reason at all. On the MSP430FR2433, a PUC for a FRAM bit error that doesn't
    /// exist leaves no reason either: "This PUC will not be recognized by the SYSRSTIV register
    /// (SYSRSTIV = 0x00)" (SLAZ664S GC4), see [`fram`](crate::fram).
    pub fn take_reset_cause(&mut self) -> Option<ResetCause> {
        let sys = unsafe { &*_pac::Sys::ptr() };
        // SYSRSTIV values: SLASEC4D Table 6-12, p. 70; SLASE59F Table 6-9, p. 48; SLASEO7C Table 9-10,
        // p. 52; SLASEE4C Table 6-10, p. 52
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
    /// Table 2-2, p. 91; SLAU445I 1.2, p. 30). [`Pmm::take_reset_cause()`] then returns
    /// [`ResetCause::SoftwareBor`].
    pub fn software_bor(&mut self) -> ! {
        self.0.pmmctl0().modify(|_, w| w.pmmpw().password().pmmswbor().set_bit());
        loop {
            msp430::asm::nop();
        }
    }

    /// Reset the device with a power-on reset (POR), which resets less than a brownout reset
    /// (PMMSWPOR, SLAU445I Table 2-2, p. 91; SLAU445I 1.2, p. 30). [`Pmm::take_reset_cause()`] then returns
    /// [`ResetCause::SoftwarePor`].
    pub fn software_por(&mut self) -> ! {
        self.0.pmmctl0().modify(|_, w| w.pmmpw().password().pmmswpor().set_bit());
        loop {
            msp430::asm::nop();
        }
    }

    /// Whether the high-side supply voltage supervisor (SVSH) stays on in LPM2, LPM3 and LPM4, as
    /// after reset (PMMCTL0.SVSHE, SLAU445I Table 2-2, p. 91). It is always on in active mode, LPM0
    /// and LPM1. Turning it off saves power in the low-power modes, but a supply drop there then
    /// resets the device only once it reaches the brownout level (SLAU445I 2.2.4, p. 87 and SLAU445I
    /// 2.2.6, p. 88). For LPM3.5 and LPM4.5, `enter_lpm3_5()` and `enter_lpm4_5()` set this.
    pub fn set_svsh(&mut self, svs: SvsState) {
        self.unlocked(|pmm| pmm.pmmctl0().modify(|_, w| w.pmmpw().password().svshe().bit(svs == SvsState::Enabled)));
    }

    /// Select how the LPM3.5 switch is controlled, see [`Lpm3_5Switch`] (PM5CTL0.LPM5SM and LPM5SW, SLAU445I
    /// Table 2-7, p. 97). In manual mode the user's guide recommends turning the switch off before LPM3.5
    /// and on again after the wake-up (SLAU445I 2.2.7, p. 88): `enter_lpm3_5()` and `enter_lpm4_5()` turn it
    /// off, and the wake-up, a BOR, brings back automatic mode with the switch connected. [`Pmm::new`] and
    /// [`Pmm::unlock_lpm5`] bring back automatic mode as well.
    #[cfg(feature = "lpm3_5_switch")]
    #[inline]
    pub fn set_lpm3_5_switch(&mut self, switch: Lpm3_5Switch) {
        // PM5CTL0 needs no PMM password (SLAU445I 2.3, p. 90). Manual mode comes first, as only then can
        // LPM5SW be written: "In automatic mode (LPM5SM = 0) ... Any write to this bit has no effect", "In
        // manual mode (LPM5SM = 1), this bit is read/write by software" (SLAU445I Table 2-7, p. 97).
        // The PACs name the LPM5SM and LPM5SW values differently, so they are written as bits: LPM5SM = 1 is
        // manual mode and LPM5SW = 1 connects the switch.
        let pm5ctl0 = self.0.pm5ctl0();
        match switch {
            Lpm3_5Switch::Automatic => unsafe { pm5ctl0.clear_bits(|w| w.lpm5sm().clear_bit()) },
            Lpm3_5Switch::Connected => {
                unsafe { pm5ctl0.set_bits(|w| w.lpm5sm().set_bit()) };
                unsafe { pm5ctl0.set_bits(|w| w.lpm5sw().set_bit()) };
            }
            Lpm3_5Switch::Disconnected => {
                unsafe { pm5ctl0.set_bits(|w| w.lpm5sm().set_bit()) };
                unsafe { pm5ctl0.clear_bits(|w| w.lpm5sw().clear_bit()) };
            }
        }
    }

    /// Whether the LPM3.5 switch is connected, in automatic mode too: there LPM5SW "represents the switch
    /// connection between Vcore and VLPM3.5" (SLAU445I Table 2-7, p. 97)
    #[cfg(feature = "lpm3_5_switch")]
    #[inline]
    pub fn lpm3_5_switch_connected(&self) -> bool { self.0.pm5ctl0().read().lpm5sw().bit_is_set() }

    /// Run `f` with write access to the PMM registers, and lock them again afterwards.
    ///
    /// Writing a PMM register other than PMMCTL0 while they are locked causes a PUC, and so does a
    /// word write of a wrong password, so they are locked again with a byte write to the upper
    /// byte of PMMCTL0 (SLAU445I 2.3, p. 90). Interrupts are disabled meanwhile, so an interrupt handler
    /// can't lock them halfway.
    #[inline]
    fn unlocked<R>(&mut self, f: impl FnOnce(&_pac::Pmm) -> R) -> R {
        critical_section::with(|_| {
            // PMMPW: "Write with 0A5h to unlock the PMM registers" (SLAU445I Table 2-2, p. 91)
            self.0.pmmctl0().modify(|_, w| w.pmmpw().password());
            let ret = f(&self.0);
            // PMMCTL0_H is the byte at offset 01h (SLAU445I Table 2-1, p. 90), as in step 9d of
            // SLAU445I 1.4.3.1, p. 41: "MOV.B #000h, &PMMCTL0_H"
            // The PACs don't have a `pmmctl0_h` register yet, so for now this is a raw byte write of 0 to
            // offset 1 of the little-endian PMMCTL0. This can use `pmmctl0_h().write(|w| w.pmmpw().lock())`
            // once the PACs all have it.
            unsafe { (self.0.pmmctl0().as_ptr() as *mut u8).add(1).write_volatile(0) };
            ret
        })
    }

    /// Configures the internal voltage reference to the specified voltage and enables it (PMMCTL2.REFVSEL
    /// and INTREFEN, SLAU445I Table 2-4, p. 93 to p. 94).
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

    /// Disables the internal reference voltage (clears PMMCTL2.INTREFEN, SLAU445I Table 2-4, p. 94)
    pub fn disable_internal_reference(&mut self, _vref: InternalVRef) {
        self.unlocked(|pmm| unsafe { pmm.pmmctl2().clear_bits(|w| w.intrefen().clear_bit()) });
    }

    /// Enables the internal temperature sensor (PMMCTL2.TSENSOREN, SLAU445I 2.2.9, p. 89 and SLAU445I
    /// Table 2-4, p. 93).
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

    /// Disables the internal temperature sensor (clears PMMCTL2.TSENSOREN, SLAU445I Table 2-4, p. 93)
    pub fn disable_internal_temp_sensor(&mut self, _tsense: InternalTempSensor) {
        self.unlocked(|pmm| unsafe { pmm.pmmctl2().clear_bits(|w| w.tsensoren().clear_bit()) });
    }

    /// Output the 1.2 V reference on the VREF+ pin, buffered (PMMCTL2.EXTREFEN, SLAU445I Table 2-4,
    /// p. 94). It can supply up to 1 mA (SLAU445I 2.2.8, p. 89: "supports up to 1-mA drive capability";
    /// specified with a 1-mA load in SLASE59F Table 5-12, p. 29, SLASEO7C 8.12.5.1, p. 33 and SLASEE4C
    /// Table 5-12, p. 31). The 1.5 V, 2.0 V and 2.5 V internal shared reference can't be output (SLAU445I
    /// 2.2.8, p. 88; SLASEC4D Table 5-10, p. 41; SLASEO7C 8.12.5.1, p. 33).
    ///
    /// The output is the buffered bandgap: "A 1.2-V reference voltage can be buffered, when EXTREFEN = 1
    /// on PMMCTL2 register, and it can be output to" the VREF+ pin (SLASEO7C 9.10.1, p. 49; SLASEE4C
    /// 6.10.1, p. 49). Setting REFBGEN starts it ("If written with a 1, the generation of the buffered bandgap
    /// voltage is started"), and the function waits until it is ready (REFBGRDY, "Buffered bandgap voltage
    /// ready status"; SLAU445I Table 2-4, p. 93). Measured on an MSP430FR2476: with EXTREFEN alone neither
    /// REFBGACT nor REFBGRDY is set, while with REFBGEN both are within a few register reads; REFGENRDY
    /// stays clear until REFGEN starts the variable reference, which isn't the output's.
    pub fn enable_vref_output<PIN: VrefOutputPin>(&mut self, pin: PIN) -> VrefOutput<PIN> {
        self.unlocked(|pmm| unsafe {
            pmm.pmmctl2().set_bits(|w| w.extrefen().set_bit().refbgen().set_bit())
        });
        while self.0.pmmctl2().read().refbgrdy().bit_is_clear() {}
        VrefOutput(pin)
    }

    /// Stop outputting the 1.2 V reference, and return the pin (clears PMMCTL2.EXTREFEN, SLAU445I
    /// Table 2-4, p. 94, and REFBGEN, which the hardware may have cleared already: "this bit is cleared by
    /// hardware or writing 0", SLAU445I Table 2-4, p. 93).
    pub fn disable_vref_output<PIN>(&mut self, output: VrefOutput<PIN>) -> PIN {
        self.unlocked(|pmm| unsafe {
            pmm.pmmctl2().clear_bits(|w| w.extrefen().clear_bit().refbgen().clear_bit())
        });
        output.0
    }
}
