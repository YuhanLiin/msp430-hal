use crate::ecomp::{
    BufferSel, ComparatorDac, ComparatorVector, DacVRef, FilterStrength, Hysteresis, OutputPolarity,
    PowerMode,
};

/// Trait that links input and output types to keep business logic device independent.
/// Should be implemented as part of support for a new device
#[allow(non_camel_case_types)]
pub trait ECompInputs: ECompPeriph {
    // The COMPx.0 to COMPx.3 input pins and the COMPxOUT pin (SLASEC4D Table 6-23, p. 78, to SLASEC4D
    // Table 6-26, p. 79; SLASEO7C Table 9-21, p. 63, and SLASEO7C Table 9-22, p. 63)
    type COMPx_0;
    type COMPx_1;
    type COMPx_2;
    type COMPx_3;
    type COMPx_Out;

    // CPPSEL and CPNSEL 010b to 101b are device specific (SLAU445I Table 18-2, p. 509)
    // The first two device-specific inputs are shared between pos and neg inputs (010b and 011b: SLASEC4D
    // Table 6-23, p. 78, SLASEC4D Table 6-24, p. 78, and SLASEO7C Table 9-21, p. 63)
    type DeviceSpecific0;
    type DeviceSpecific1;

    // Device specific 2 and 3 can differ between pos and neg inputs: 101b selects a different SAC output for
    // each on the MSP430FR2x5x (SLASEC4D Table 6-23, p. 78, and SLASEC4D Table 6-24, p. 78), while the
    // MSP430FR247x shares all of its inputs (SLASEO7C Table 9-21, p. 63)
    type DeviceSpecific2Pos;
    type DeviceSpecific2Neg;

    type DeviceSpecific3Pos;
    type DeviceSpecific3Neg;

    // The eCOMP can alternatively source from SAC units or a TIA, if they exist
    #[cfg(feature = "sac_l3")]
    /// The SAC module connected to the positive comparator input (CPPSEL = 101b: SLASEC4D Table 6-23, p. 78,
    /// and SLASEC4D Table 6-24, p. 78)
    type SACp;
    #[cfg(feature = "sac")]
    /// The SAC module connected to the negative comparator input (CPNSEL = 101b: SLASEC4D Table 6-23, p. 78,
    /// and SLASEC4D Table 6-24, p. 78)
    type SACn;
    #[cfg(feature = "tia")]
    type TIA;
}

#[allow(non_camel_case_types)]
pub trait ECompPeriph {
    /// Write CPDACEN, CPDACREFS, CPDACBUFS and CPDACSW in CPxDACCTL (SLAU445I Table 18-6, p. 512)
    fn cpxdacctl(enable: bool, vref: DacVRef, buf_mode: DacBufferMode, buf: BufferSel);
    /// Write CPDACBUF1, bits 5-0 of CPxDACDATA (SLAU445I Table 18-7, p. 513)
    fn set_buf1_val(buf: u8);
    /// Write CPDACBUF2, bits 13-8 of CPxDACDATA (SLAU445I Table 18-7, p. 513)
    fn set_buf2_val(buf: u8);
    /// Write CPDACBUFS in CPxDACCTL (SLAU445I Table 18-6, p. 512)
    fn set_dac_buffer_mode(mode: DacBufferMode);
    /// Write CPDACSW in CPxDACCTL (SLAU445I Table 18-6, p. 512)
    fn select_buffer(sel: BufferSel);
    /// Enable and select both inputs: CPPEN, CPPSEL, CPNEN and CPNSEL in CPxCTL0 (SLAU445I Table 18-2,
    /// p. 509)
    fn cpxctl0(pos_in: u8, neg_in: u8);
    /// Write CPHSEL, CPEN, CPMSEL, CPFLT, CPFLTDLY and CPINV in CPxCTL1 (SLAU445I Table 18-3, p. 510)
    fn configure_comparator(
        pol: OutputPolarity,
        pwr: PowerMode,
        hstr: Hysteresis,
        fltr: FilterStrength,
    );
    /// Read CPOUT in CPxCTL1 (SLAU445I Table 18-3, p. 511)
    fn value() -> bool;
    /// Set CPIE in CPxCTL1 (SLAU445I Table 18-3, p. 510)
    fn en_cpie();
    /// Clear CPIE in CPxCTL1 (SLAU445I Table 18-3, p. 510)
    fn dis_cpie();
    /// Set CPIIE in CPxCTL1 (SLAU445I Table 18-3, p. 510)
    fn en_cpiie();
    /// Clear CPIIE in CPxCTL1 (SLAU445I Table 18-3, p. 510)
    fn dis_cpiie();
    /// Read CPIFG in CPxINT (SLAU445I Table 18-4, p. 511)
    fn rising_flag() -> bool;
    /// Read CPIIFG in CPxINT (SLAU445I Table 18-4, p. 511)
    fn falling_flag() -> bool;
    /// Clear CPIFG and CPIIFG. They clear by writing 1 (SLAU445I Table 18-4, p. 511: "Write 1 to clear this
    /// bit", and measured on an MSP430FR2476: writing 0 leaves them set).
    fn clear_edge_flags();
    /// Read CPxIV, which clears the highest-priority enabled flag (SLAU445I Table 18-5, p. 512)
    fn iv() -> ComparatorVector;
}

// Marker trait for an eCOMP DAC. Since the DAC has a typestate (hardware/software double buffer)
// we can't just say `type CompDac = ComparatorDac<COMP>`
pub trait CompDacPeriph<COMP: ECompPeriph> {}
impl<COMP: ECompInputs, MODE> CompDacPeriph<COMP> for ComparatorDac<'_, COMP, MODE> {}

/// Possible eCOMP DAC dual buffer modes (CPDACBUFS: SLAU445I Table 18-6, p. 512; SLAU445I 18.2.4, p. 506)
pub enum DacBufferMode {
    /// In hardware mode the DAC count is determined by either CPDACBUF1 or CPDACBUF2,
    /// based on the output value of the comparator
    Hardware,
    /// In software mode the DAC count can be switched between CPDACBUF1 and CPDACBUF2 by software
    Software,
}
impl From<DacBufferMode> for bool {
    fn from(value: DacBufferMode) -> Self {
        match value {
            DacBufferMode::Hardware => false,
            DacBufferMode::Software => true,
        }
    }
}

macro_rules! impl_ecomp {
    ($COMP: ident,
        $cpctl0: ident, $cpctl1: ident,
        $cpdacctl: ident, $cpdacdata: ident,
        $cpint: ident, $cpiv: ident ) => {
        impl ECompPeriph for $COMP {
            // CPxDACCTL: CPDACEN, CPDACREFS, CPDACBUFS and CPDACSW (SLAU445I Table 18-6, p. 512)
            #[inline(always)]
            fn cpxdacctl(enable: bool, vref: DacVRef, buf_mode: DacBufferMode, buf: BufferSel) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl().modify(|_, w| w
                        .cpdacen().bit(enable)
                        .cpdacrefs().bit(vref.into())
                        .cpdacbufs().bit(buf_mode.into())
                        .cpdacsw().bit(buf.into())
                    );
                }
            }
            // CPxDACDATA: CPDACBUF1 in bits 5-0, CPDACBUF2 in bits 13-8 (SLAU445I Table 18-7, p. 513)
            #[inline(always)]
            fn set_buf1_val(buf: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacdata().modify(|_, w| w.cpdacbuf1().bits(buf));
                }
            }
            // CPDACBUF2, bits 13-8 of CPxDACDATA (SLAU445I Table 18-7, p. 513)
            #[inline(always)]
            fn set_buf2_val(buf: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacdata().modify(|_, w| w.cpdacbuf2().bits(buf));
                }
            }
            // CPDACSW, bit 0 of CPxDACCTL: 0b selects CPDACBUF1, 1b CPDACBUF2 (SLAU445I Table 18-6, p. 512)
            #[inline(always)]
            fn select_buffer(sel: BufferSel) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl().modify(|_, w| w.cpdacsw().bit(sel.into()));
                }
            }
            // CPDACBUFS, bit 1 of CPxDACCTL: 0b lets the comparator output select the buffer, 1b CPDACSW
            // (SLAU445I Table 18-6, p. 512)
            #[inline(always)]
            fn set_dac_buffer_mode(mode: DacBufferMode) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl().modify(|_, w| w.cpdacbufs().bit(mode.into()));
                }
            }
            // CPxCTL0: CPPEN, CPPSEL, CPNEN and CPNSEL (SLAU445I Table 18-2, p. 509)
            #[inline(always)]
            fn cpxctl0(pos_in: u8, neg_in: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpctl0().modify(|_, w| w
                        .cppen().set_bit()
                        .cppsel().bits(pos_in.into())
                        .cpnen().set_bit()
                        .cpnsel().bits(neg_in.into())
                    );
                }
            }
            // CPxCTL1: CPHSEL, CPEN, CPMSEL, CPFLT, CPFLTDLY and CPINV (SLAU445I Table 18-3, p. 510)
            #[inline(always)]
            fn configure_comparator(
                pol: OutputPolarity,
                pwr: PowerMode,
                hstr: Hysteresis,
                fltr: FilterStrength,
            ) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpctl1().modify(|_, w| w
                        .cphsel().bits(hstr as u8)
                        .cpen().set_bit()
                        .cpmsel().bit(pwr.into())
                        .cpflt().bit(fltr != FilterStrength::Off)
                        // Note: fltr could be 3 bits, will be truncated to 00 but only if filter is off anyway
                        // (CPFLTDLY is bits 7-6: SLAU445I Table 18-3, p. 510)
                        .cpfltdly().bits(fltr as u8) 
                        .cpinv().bit(pol.into())
                    );
                }
            }
            // CPOUT (SLAU445I Table 18-3, p. 511)
            #[inline(always)]
            fn value() -> bool {
                let comp = unsafe { $COMP::steal() };
                comp.$cpctl1().read().cpout().bit()
            }
            // CPIE, bit 14 of CPxCTL1 (SLAU445I Table 18-3, p. 510)
            #[inline(always)]
            fn en_cpie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().set_bits(|w| w.cpie().set_bit())
                }
            }
            // CPIE, bit 14 of CPxCTL1 (SLAU445I Table 18-3, p. 510)
            #[inline(always)]
            fn dis_cpie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().clear_bits(|w| w.cpie().clear_bit())
                }
            }
            // CPIIE, bit 15 of CPxCTL1 (SLAU445I Table 18-3, p. 510)
            #[inline(always)]
            fn en_cpiie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().set_bits(|w| w.cpiie().set_bit())
                }
            }
            // CPIIE, bit 15 of CPxCTL1 (SLAU445I Table 18-3, p. 510)
            #[inline(always)]
            fn dis_cpiie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().clear_bits(|w| w.cpiie().clear_bit())
                }
            }
            // CPIFG in CPxINT (SLAU445I Table 18-4, p. 511)
            #[inline(always)]
            fn rising_flag() -> bool {
                let comp = unsafe { $COMP::steal() };
                comp.$cpint().read().cpifg().bit()
            }
            // CPIIFG in CPxINT (SLAU445I Table 18-4, p. 511)
            #[inline(always)]
            fn falling_flag() -> bool {
                let comp = unsafe { $COMP::steal() };
                comp.$cpint().read().cpiifg().bit()
            }
            // CPIIFG and CPIFG clear when written with 1 (SLAU445I Table 18-4, p. 511)
            #[inline(always)]
            fn clear_edge_flags() {
                let comp = unsafe { $COMP::steal() };
                comp.$cpint().write(|w| w.cpifg().set_bit().cpiifg().set_bit());
            }
            // CPxIV (SLAU445I Table 18-5, p. 512)
            #[inline(always)]
            fn iv() -> ComparatorVector {
                let comp = unsafe { $COMP::steal() };
                let r = comp.$cpiv().read();
                let iv = r.cpiv();
                if iv.is_cpifg() { ComparatorVector::RisingEdge }
                else if iv.is_cpiifg() { ComparatorVector::FallingEdge }
                else { ComparatorVector::None }
            }
        }
    };
}
pub(crate) use impl_ecomp;
