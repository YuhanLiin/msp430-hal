/// Trait representing a Smart Analog Combo (SAC) peripheral.
pub trait SacPeriph {
    /// Non-inverting opamp input pin, OAx+ (PSEL = 00: SLASEC4D Table 6-27, p. 79, to SLASEC4D Table 6-30,
    /// p. 80)
    type PosInputPin;
    /// Inverting opamp input pin, OAx- (NSEL = 00: SLASEC4D Table 6-27, p. 79, to SLASEC4D Table 6-30, p. 80)
    type NegInputPin;
    /// Opamp output pin, OAxO (SLASEC4D Table 6-63, p. 96, and SLASEC4D Table 6-65, p. 100)
    type OutputPin;
    /// Write SACxOA: NSEL, PSEL, OAPM, NMUXEN, PMUXEN, SACEN and OAEN (SLAU445I Table 20-6, p. 532)
    fn configure_sacoa(psel: u8, nsel: NSel, pm: bool);
    /// Write SACxPGA: GAIN and MSEL (SLAU445I Table 20-7, p. 533)
    fn configure_sacpga(gain: u8, mode: MSel);
    /// Write SACxDAC: DACSREF, DACLSEL, DACDMAE, DACIE and DACEN (SLAU445I Table 20-8, p. 534)
    fn configure_dac(load_condition: u8, vref: bool, interrupts: bool);
    /// Write the DAC data, SACxDAT (SLAU445I Table 20-9, p. 535)
    fn set_dac_count(val: u16);
    /// Reads SACxIV, which clears DACIFG: whether it reported DACIFG, set when the DAC loaded new data
    /// (SLAU445I Table 20-10, p. 536, and SLAU445I Table 20-11, p. 537)
    fn dac_iv_dacifg() -> bool;
}

// The sac module's input enums give the PSEL value of each source, so no need for a separate enum

// NSEL (SLAU445I Table 20-6, p. 532); 10b, device specific, is the paired OA (SLASEC4D Table 6-27, p. 79, to
// SLASEC4D Table 6-30, p. 80)
#[derive(Debug, Copy, Clone)]
pub enum NSel {
    ExtPinMinus = 0b00,
    Feedback    = 0b01,
    PairedOpamp = 0b10,
}

// MSEL (SLAU445I Table 20-7, p. 533)
#[derive(Debug, Copy, Clone)]
pub enum MSel {
    Inverting    = 0b00,
    Follower     = 0b01,
    NonInverting = 0b10,
    Cascade      = 0b11,
}

macro_rules! impl_sac_periph {
    ($SAC: ident,
        $pos_port: ident, $pos_pin: ident, // Positive input
        $neg_port: ident, $neg_pin: ident, // Negative input
        $out_port: ident, $out_pin: ident, // Output 
        $sacXoa: ident, $sacXpga: ident, $sacXdac: ident, $sacXdat: ident, $sacXiv: ident) => {
        impl SacPeriph for $SAC {
            // The OA pins are their port's tertiary module function, PxSELx = 11 (SLAU445I Table 8-3, p. 314;
            // SLASEC4D Table 6-63, p. 96, and SLASEC4D Table 6-65, p. 100)
            type PosInputPin = Pin<$pos_port, $pos_pin, Alternate3<Input<Floating>>>;
            type NegInputPin = Pin<$neg_port, $neg_pin, Alternate3<Input<Floating>>>;
            type OutputPin   = Pin<$out_port, $out_pin, Alternate3<Input<Floating>>>;
            // SACxOA: NSEL, PSEL, OAPM, NMUXEN, PMUXEN, SACEN and OAEN (SLAU445I Table 20-6, p. 532)
            #[inline(always)]
            fn configure_sacoa(psel: u8, nsel: NSel, pm: bool) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXoa().write(|w| w
                        .nsel().bits(nsel as u8)
                        .psel().bits(psel)
                        .oapm().bit(pm)
                        .nmuxen().set_bit()
                        .pmuxen().set_bit()
                        .sacen().set_bit()
                        .oaen().set_bit()
                    );
                }
            }
            // SACxPGA: GAIN and MSEL (SLAU445I Table 20-7, p. 533)
            #[inline(always)]
            fn configure_sacpga(gain: u8, msel: MSel) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXpga().write(|w| w
                        .gain().bits(gain)
                        .msel().bits(msel as u8));
                }
            }
            // SACxDAC: DACSREF, DACLSEL, DACDMAE, DACIE and DACEN (SLAU445I Table 20-8, p. 534). "This
            // register can be modified only when DACEN = 0" (SLAU445I 20.4.3, p. 534), as it is after reset
            // (SLAU445I Table 20-5, p. 531)
            #[inline(always)]
            fn configure_dac(lsel: u8, vref: bool, interrupts: bool) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXdac().write(|w| w
                        .dacsref().bit(vref)
                        .daclsel().bits(lsel)
                        .dacdmae().clear_bit()
                        .dacie().bit(interrupts)
                        .dacen().set_bit()
                    );
                }
            }
            // SACxDAT, written as a word: "Only word access to the SACxDAT register is allowed" (SLAU445I
            // Table 20-9 note 1, p. 535)
            #[inline(always)]
            fn set_dac_count(val: u16) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXdat().write(|w| w.dacdata().bits(val));
                }
            }
            // SACxIV (SLAU445I Table 20-11, p. 537)
            #[inline(always)]
            fn dac_iv_dacifg() -> bool {
                unsafe { $SAC::steal() }.$sacXiv().read().saciv().is_dacifg()
            }
        }
    };
}
pub(crate) use impl_sac_periph;
