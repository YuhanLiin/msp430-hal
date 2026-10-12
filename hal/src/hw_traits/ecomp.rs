use crate::ecomp::{
    BufferSel, ComparatorDac, DacVRef, FilterStrength, Hysteresis, OutputPolarity, PowerMode,
};

/// Trait that links input and output types to keep business logic device independent.
/// Should be implemented as part of support for a new device
#[allow(non_camel_case_types)]
pub trait ECompInputs: ECompPeriph {
    type COMPx_0;
    type COMPx_1;
    type COMPx_2;
    type COMPx_3;
    type COMPx_Out;

    // The first two device-specific inputs are shared between pos and neg inputs
    type DeviceSpecific0;
    type DeviceSpecific1;

    // Device specific 2 and 3 are distinct between pos and neg inputs
    type DeviceSpecific2Pos;
    type DeviceSpecific2Neg;

    type DeviceSpecific3Pos;
    type DeviceSpecific3Neg;

    // The eCOMP can alternatively source from SAC units or a TIA, if they exist
    #[cfg(feature = "sac_l3")]
    /// The SAC module connected to the positive comparator input
    type SACp;
    #[cfg(feature = "sac")]
    /// The SAC module connected to the negative comparator input
    type SACn;
    #[cfg(feature = "tia")]
    type TIA;
}

#[allow(non_camel_case_types)]
pub trait ECompPeriph {
    fn cpxdacctl(enable: bool, vref: DacVRef, buf_mode: DacBufferMode, buf: BufferSel);
    fn set_buf1_val(buf: u8);
    fn set_buf2_val(buf: u8);
    fn set_dac_buffer_mode(mode: DacBufferMode);
    fn select_buffer(sel: BufferSel);
    fn cpxctl0(pos_in: u8, neg_in: u8);
    fn configure_comparator(
        pol: OutputPolarity,
        pwr: PowerMode,
        hstr: Hysteresis,
        fltr: FilterStrength,
    );
    fn value() -> bool;
    fn en_cpie();
    fn dis_cpie();
    fn en_cpiie();
    fn dis_cpiie();
    /// CPxINT: CPIFG in bit 0, CPIIFG in bit 1
    fn int_flags() -> u16;
    /// Clear the CPxINT flags in `mask`, which clear when written with 1
    fn clear_int_flags(mask: u16);
    /// Read CPxIV, which clears the highest-priority enabled flag
    fn iv() -> u16;
}

// Marker trait for an eCOMP DAC. Since the DAC has a typestate (hardware/software double buffer)
// we can't just say `type CompDac = ComparatorDac<COMP>`
pub trait CompDacPeriph<COMP: ECompPeriph> {}
impl<COMP: ECompInputs, MODE> CompDacPeriph<COMP> for ComparatorDac<'_, COMP, MODE> {}

/// Possible eCOMP DAC dual buffer modes
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
            #[inline(always)]
            fn set_buf1_val(buf: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacdata().modify(|_, w| w.cpdacbuf1().bits(buf));
                }
            }
            #[inline(always)]
            fn set_buf2_val(buf: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacdata().modify(|_, w| w.cpdacbuf2().bits(buf));
                }
            }
            #[inline(always)]
            fn select_buffer(sel: BufferSel) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl().modify(|_, w| w.cpdacsw().bit(sel.into()));
                }
            }
            #[inline(always)]
            fn set_dac_buffer_mode(mode: DacBufferMode) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl().modify(|_, w| w.cpdacbufs().bit(mode.into()));
                }
            }
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
                        .cpfltdly().bits(fltr as u8) 
                        .cpinv().bit(pol.into())
                    );
                }
            }
            #[inline(always)]
            fn value() -> bool {
                let comp = unsafe { $COMP::steal() };
                comp.$cpctl1().read().cpout().bit()
            }
            #[inline(always)]
            fn en_cpie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().set_bits(|w| w.cpie().set_bit())
                }
            }
            #[inline(always)]
            fn dis_cpie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().clear_bits(|w| w.cpie().clear_bit())
                }
            }
            #[inline(always)]
            fn en_cpiie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().set_bits(|w| w.cpiie().set_bit())
                }
            }
            #[inline(always)]
            fn dis_cpiie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1().clear_bits(|w| w.cpiie().clear_bit())
                }
            }
            #[inline(always)]
            fn int_flags() -> u16 {
                let comp = unsafe { $COMP::steal() };
                comp.$cpint().read().bits()
            }
            #[inline(always)]
            fn clear_int_flags(mask: u16) {
                let comp = unsafe { $COMP::steal() };
                comp.$cpint().write(|w| unsafe { w.bits(mask) });
            }
            #[inline(always)]
            fn iv() -> u16 {
                let comp = unsafe { $COMP::steal() };
                comp.$cpiv().read().bits()
            }
        }
    };
}
pub(crate) use impl_ecomp;
