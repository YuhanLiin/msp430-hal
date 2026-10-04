//! Interrupt Compare Controller (ICC): interrupt priorities and nesting
//!
//! Only on the MSP430FR2x5x (SLASEC4D 1.1, p. 1). Without the ICC, the interrupt vector table fixes the
//! interrupt priorities, and an interrupt handler runs to the end unless it enables interrupts itself, in
//! which case any interrupt can interrupt it (SLAU445I 1.3.5, p. 35 and SLAU445I 5.2, p. 282). With the
//! ICC enabled, each maskable interrupt source has one of four priorities, and a handler can only be
//! interrupted by a higher priority (SLAU445I 5.2, p. 282). Sources with the same priority are served in
//! vector table order (SLAU445I 5.2, p. 282).
//!
//! Nesting needs interrupts enabled in each handler: call `msp430::interrupt::enable()` in every interrupt
//! handler that may be interrupted, after clearing its interrupt flag (SLASEC4D 6.10.7, p. 71: "It is
//! required to enable GIE in ISR for proper ICC operation"; the order of TI's recommended flow, SLAU445I
//! 5.2.6.2, p. 287).
//!
//! ```ignore
//! let mut icc = Icc::new(periph.icc);
//! icc.set_priority(IccSource::Timer0B0, Priority::Highest);
//! icc.enable();
//! ```

use crate::_pac;

/// The priority of an interrupt source (ILSRx, SLAU445I 5.2.2, p. 283): `Highest` (level 0), `High`,
/// `Low`, or `Lowest` (level 3, as after reset). The ICC serves a higher priority first, and lets it
/// interrupt the handler of a lower one (SLAU445I 5.2, p. 282).
pub use crate::_pac::icc::iccilsr::Ilsr as Priority;
use crate::_pac::icc::iccsc::Icmc;

/// The maskable interrupt sources the ICC manages, by their level setting fields ILSR0 to ILSR21 (SLASEC4D
/// Table 6-13, p. 71 to p. 72)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IccSource {
    /// Port 4 (`PORT4`; ILSR0, SLASEC4D Table 6-13, p. 71)
    Port4 = 0,
    /// Port 3 (`PORT3`; ILSR1, SLASEC4D Table 6-13, p. 71)
    Port3 = 1,
    /// Port 2 (`PORT2`; ILSR2, SLASEC4D Table 6-13, p. 71)
    Port2 = 2,
    /// Port 1 (`PORT1`; ILSR3, SLASEC4D Table 6-13, p. 71)
    Port1 = 3,
    /// The DACs of SAC1 and SAC3 (`SAC1_SAC3`), MSP430FR235x only (ILSR4, SLASEC4D Table 6-13, note 1, p. 71)
    #[cfg(feature = "sac")]
    Sac1Sac3 = 4,
    /// The DACs of SAC0 and SAC2 (`SAC0_SAC2`), MSP430FR235x only (ILSR5, SLASEC4D Table 6-13, note 1, p. 71)
    #[cfg(feature = "sac")]
    Sac0Sac2 = 5,
    /// eCOMP0 and eCOMP1 (`ECOMP0_ECOMP1`; ILSR6, SLASEC4D Table 6-13, p. 71)
    EComp = 6,
    /// The ADC (`ADC`; ILSR7, SLASEC4D Table 6-13, p. 71)
    Adc = 7,
    /// eUSCI_B1 (`EUSCI_B1`; ILSR8, SLASEC4D Table 6-13, p. 71)
    EUsciB1 = 8,
    /// eUSCI_B0 (`EUSCI_B0`; ILSR9, SLASEC4D Table 6-13, p. 71)
    EUsciB0 = 9,
    /// eUSCI_A1 (`EUSCI_A1`; ILSR10, SLASEC4D Table 6-13, p. 71)
    EUsciA1 = 10,
    /// eUSCI_A0 (`EUSCI_A0`; ILSR11, SLASEC4D Table 6-13, p. 71)
    EUsciA0 = 11,
    /// The watchdog in interval mode (`WDT`; ILSR12, SLASEC4D Table 6-13, p. 71)
    Watchdog = 12,
    /// The RTC (`RTC`; ILSR13, SLASEC4D Table 6-13, p. 71)
    Rtc = 13,
    /// TB3's CCR1 to CCR6 and overflow (`TIMER3_B1`; ILSR14, SLASEC4D Table 6-13, p. 71)
    Timer3B1 = 14,
    /// TB3's CCR0 (`TIMER3_B0`; ILSR15, SLASEC4D Table 6-13, p. 71)
    Timer3B0 = 15,
    /// TB2's CCR1, CCR2 and overflow (`TIMER2_B1`; ILSR16, SLASEC4D Table 6-13, p. 72)
    Timer2B1 = 16,
    /// TB2's CCR0 (`TIMER2_B0`; ILSR17, SLASEC4D Table 6-13, p. 72)
    Timer2B0 = 17,
    /// TB1's CCR1, CCR2 and overflow (`TIMER1_B1`; ILSR18, SLASEC4D Table 6-13, p. 72)
    Timer1B1 = 18,
    /// TB1's CCR0 (`TIMER1_B0`; ILSR19, SLASEC4D Table 6-13, p. 72)
    Timer1B0 = 19,
    /// TB0's CCR1, CCR2 and overflow (`TIMER0_B1`; ILSR20, SLASEC4D Table 6-13, p. 72)
    Timer0B1 = 20,
    /// TB0's CCR0 (`TIMER0_B0`; ILSR21, SLASEC4D Table 6-13, p. 72)
    Timer0B0 = 21,
}

/// The Interrupt Compare Controller (SLAU445I chapter 5, p. 280; its registers: SLAU445I Table 5-1, p. 292)
pub struct Icc(_pac::Icc);

impl Icc {
    /// Take the ICC. It stays disabled until [`Icc::enable`], with every source at the lowest priority (reset
    /// values, SLAU445I Table 5-1, p. 292 and SLAU445I Table 5-2, p. 293).
    #[inline]
    pub fn new(icc: _pac::Icc) -> Self { Icc(icc) }

    // Eight 2-bit fields per register: source n is ILSRn, field n % 8 of ICCILSR(n / 8) (SLAU445I
    // Table 5-1, p. 292 and SLAU445I Table 5-4, p. 294)
    #[inline(always)]
    fn ilsr_index(source: IccSource) -> (usize, u8) {
        let index = source as u8;
        ((index / 8) as usize, index % 8)
    }

    /// Set the priority of an interrupt source. Its priority can change at any time (SLAU445I 5.2.2, p. 283).
    #[inline]
    pub fn set_priority(&mut self, source: IccSource, priority: Priority) {
        let (reg, field) = Self::ilsr_index(source);
        critical_section::with(|_| {
            self.0.iccilsr(reg).modify(|_, w| w.ilsr(field).variant(priority));
        });
    }

    /// The priority of an interrupt source (its ILSRx field, SLAU445I Table 5-4, p. 294).
    #[inline]
    pub fn priority(&self, source: IccSource) -> Priority {
        let (reg, field) = Self::ilsr_index(source);
        self.0.iccilsr(reg).read().ilsr(field).variant()
    }

    /// Serve interrupts by priority, with nesting (ICCEN, SLAU445I Table 5-2, p. 293).
    ///
    /// Interrupts are disabled while ICCEN changes, as the user's guide recommends ("It is recommended to
    /// disable the GIE bit before enabling or disabling the ICC", SLAU445I 5.2.6.3, p. 288 and note "ICC
    /// Bypass", p. 291). Call it from the main loop, not from an interrupt handler ("It is recommended to
    /// enable or disable the ICC module only in the main loop of the software code", same note).
    #[inline]
    pub fn enable(&mut self) {
        critical_section::with(|_| unsafe { self.0.iccsc().set_bits(|w| w.iccen().set_bit()) })
    }

    /// Serve interrupts in vector table order again (ICCEN, SLAU445I 5.2, p. 282). Like [`Icc::enable`], it
    /// changes ICCEN with interrupts disabled, and is for the main loop only (SLAU445I 5.2.6.3, p. 288 and
    /// note "ICC Bypass", p. 291).
    #[inline]
    pub fn disable(&mut self) {
        critical_section::with(|_| unsafe { self.0.iccsc().clear_bits(|w| w.iccen().clear_bit()) })
    }

    /// The priority of the interrupt being served (ICMC), or `None` if no interrupt is (VSEFLG). In a nested
    /// handler, this is the priority of the innermost one (SLAU445I 5.2.4, p. 284 and SLAU445I Table 5-2,
    /// p. 293).
    #[inline]
    pub fn current_priority(&self) -> Option<Priority> {
        // ICMC and VSEFLG in ICCSC (SLAU445I Table 5-2, p. 293)
        let sc = self.0.iccsc().read();
        if sc.vseflg().bit_is_set() {
            return None;
        }
        Some(match sc.icmc().variant() {
            Icmc::Highest => Priority::Highest,
            Icmc::High => Priority::High,
            Icmc::Low => Priority::Low,
            Icmc::Lowest => Priority::Lowest,
        })
    }

    /// How many interrupt handlers are nested at the moment, 0 to 4 (MVSSP, ICCMVS bits 10-8, SLAU445I
    /// Table 5-3, p. 294).
    #[inline]
    pub fn nesting_depth(&self) -> u8 { self.0.iccmvs().read().mvssp().bits() }
}
