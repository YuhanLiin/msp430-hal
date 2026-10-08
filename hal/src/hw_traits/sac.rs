/// Trait representing a Smart Analog Combo (SAC) peripheral.
pub trait SacPeriph {
    /// Non-inverting opamp input pin, OAx+
    type PosInputPin;
    /// Inverting opamp input pin, OAx-
    type NegInputPin;
    /// Opamp output pin, OAxO
    type OutputPin;
    /// Write SACxOA: NSEL, PSEL, OAPM, NMUXEN, PMUXEN, SACEN and OAEN (SLAU445I Table 20-6, p. 532)
    fn configure_sacoa(psel: u8, nsel: NSel, pm: bool);
    /// Write SACxPGA: GAIN and MSEL (SLAU445I Table 20-7, p. 533)
    fn configure_sacpga(gain: u8, mode: MSel);
    /// Write SACxDAC: DACSREF, DACLSEL, DACDMAE, DACIE and DACEN (SLAU445I Table 20-8, p. 534)
    fn configure_dac(load_condition: u8, vref: bool, interrupts: bool);
    /// Write the DAC data, SACxDAT (SLAU445I Table 20-9, p. 535)
    fn set_dac_count(val: u16);
    /// Reads SACxIV, which clears DACIFG. 4 if the DAC loaded new data. (SLAU445I Table 20-10, p. 536, and
    /// SLAU445I Table 20-11, p. 537)
    fn dac_iv() -> u16;
}

// The sac module's input enums give the PSEL value of each source, so no need for a separate enum

// NSEL (SLAU445I Table 20-6, p. 532); 10b, device specific, is the paired OA
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
            type PosInputPin = Pin<$pos_port, $pos_pin, Alternate3<Input<Floating>>>;
            type NegInputPin = Pin<$neg_port, $neg_pin, Alternate3<Input<Floating>>>;
            type OutputPin   = Pin<$out_port, $out_pin, Alternate3<Input<Floating>>>;
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
            #[inline(always)]
            fn configure_sacpga(gain: u8, msel: MSel) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXpga().write(|w| w
                        .gain().bits(gain)
                        .msel().bits(msel as u8));
                }
            }
            // SACxDAC can only be modified while DACEN = 0, as it is after reset (SLAU445I 20.4.3, p. 534)
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
            // SACxDAT is written as a word: only word access is allowed (SLAU445I Table 20-9, p. 535)
            #[inline(always)]
            fn set_dac_count(val: u16) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXdat().write(|w| w.dacdata().bits(val));
                }
            }
            #[inline(always)]
            fn dac_iv() -> u16 {
                unsafe { $SAC::steal() }.$sacXiv().read().bits()
            }
        }
    };
}
pub(crate) use impl_sac_periph;
