use super::Steal;

// The port registers: SLAU445I Table 8-4, p. 319 to p. 331
pub trait GpioPeriph: Steal {
    // PxIN, read only (SLAU445I Table 8-9, p. 334)
    fn pxin_rd(&self) -> u8;

    // PxOUT (SLAU445I Table 8-10, p. 334)
    fn pxout_rd(&self) -> u8;
    fn pxout_wr(&self, bits: u8);
    fn pxout_set(&self, bits: u8);
    fn pxout_clear(&self, bits: u8);
    fn pxout_toggle(&self, bits: u8);

    // PxDIR (SLAU445I Table 8-11, p. 334)
    fn pxdir_rd(&self) -> u8;
    fn pxdir_wr(&self, bits: u8);
    fn pxdir_set(&self, bits: u8);
    fn pxdir_clear(&self, bits: u8);

    // PxREN (SLAU445I Table 8-12, p. 335)
    fn pxren_rd(&self) -> u8;
    fn pxren_wr(&self, bits: u8);
    fn pxren_set(&self, bits: u8);
    fn pxren_clear(&self, bits: u8);

    // PxSELC, which "Always reads as 0" (SLAU445I Table 8-15, p. 336), so it is only written
    fn pxselc_wr(&self, bits: u8);

    // PxSEL0 (SLAU445I Table 8-13, p. 335)
    fn pxsel0_rd(&self) -> u8;
    fn pxsel0_wr(&self, bits: u8);
    fn pxsel0_set(&self, bits: u8);
    fn pxsel0_clear(&self, bits: u8);

    // PxSEL1 (SLAU445I Table 8-14, p. 335)
    fn pxsel1_rd(&self) -> u8;
    fn pxsel1_wr(&self, bits: u8);
    fn pxsel1_set(&self, bits: u8);
    fn pxsel1_clear(&self, bits: u8);

    // ADCPCTL0 to ADCPCTL7 are bits 0 to 7 of SYSCFG2 (SLAU445I Table 1-31, p. 82; SLASE59F Table 6-17,
    // p. 55; SLASEE4C Table 6-15, p. 58)
    #[cfg(feature = "adcpctl")]
    fn adcpctl_set(&self, mask: u16) {
        unsafe { crate::_pac::Sys::steal().syscfg2().set_bits(|w| w.bits(mask)) };
    }
    #[cfg(feature = "adcpctl")]
    fn adcpctl_clr(&self, mask: u16) {
        unsafe { crate::_pac::Sys::steal().syscfg2().clear_bits(|w| w.bits(mask)) };
    }
}

// Ports with interrupts also have PxIES, PxIE and PxIFG (SLAU445I 8.2.6, p. 314) and a word-accessible
// PxIV (SLAU445I 8.2.6, p. 315)
pub trait IntrPeriph: GpioPeriph {
    // PxIES (SLAU445I Table 8-16, p. 336)
    fn pxies_rd(&self) -> u8;
    fn pxies_wr(&self, bits: u8);
    fn pxies_set(&self, bits: u8);
    fn pxies_clear(&self, bits: u8);

    // PxIE (SLAU445I Table 8-17, p. 336)
    fn pxie_rd(&self) -> u8;
    fn pxie_wr(&self, bits: u8);
    fn pxie_set(&self, bits: u8);
    fn pxie_clear(&self, bits: u8);

    // PxIFG (SLAU445I Table 8-18, p. 337)
    fn pxifg_rd(&self) -> u8;
    fn pxifg_wr(&self, bits: u8);
    fn pxifg_set(&self, bits: u8);
    fn pxifg_clear(&self, bits: u8);

    // PxIV, a 16-bit register (SLAU445I Tables 8-5 to 8-8, p. 332 to p. 333). Reading it clears the flag it
    // reports (SLAU445I 8.2.6, p. 315).
    fn pxiv_rd(&self) -> crate::gpio::GpioVector;
}

// Read, write, set and clear the bits of one 8-bit port register (SLAU445I Table 8-4, p. 319 to p. 331)
macro_rules! reg_methods {
    ($reg:ident, $rd:ident, $wr:ident, $set:ident, $clear:ident) => {
        // One port register of SLAU445I Tables 8-9 to 8-18, p. 334 to p. 337
        #[inline(always)]
        fn $rd(&self) -> u8 { self.$reg().read().bits() }

        #[inline(always)]
        fn $wr(&self, bits: u8) { self.$reg().write(|w| unsafe { w.bits(bits) }); }

        #[inline(always)]
        fn $set(&self, bits: u8) { unsafe { self.$reg().set_bits(|w| w.bits(bits)) } }

        #[inline(always)]
        fn $clear(&self, bits: u8) { unsafe { self.$reg().clear_bits(|w| w.bits(bits)) } }
    };
}
pub(crate) use reg_methods;

macro_rules! gpio_impl {
    ($px:ident: $Px:ident =>
     $pxin:ident, $pxout:ident, $pxdir:ident, $pxren:ident, $pxselc:ident, $pxsel0:ident, $pxsel1:ident
     $(, [$pxies:ident, $pxie:ident, $pxifg:ident, $pxiv:ident])?
    ) => {
        mod $px {
            use crate::{pac, gpio::*, hw_traits::{Steal, gpio::*}};

            impl Steal for pac::$Px {
                #[inline(always)]
                unsafe fn steal() -> Self {
                    $Px::steal()
                }
            }

            impl GpioPeriph for pac::$Px {
                // PxIN (SLAU445I Table 8-9, p. 334)
                #[inline(always)]
                fn pxin_rd(&self) -> u8 {
                    self.$pxin().read().bits()
                }

                // PxSELC (SLAU445I Table 8-15, p. 336)
                #[inline(always)]
                fn pxselc_wr(&self, bits: u8) {
                    self.$pxselc().write(|w| unsafe { w.bits(bits) });
                }

                // PxOUT (SLAU445I Table 8-10, p. 334)
                #[inline(always)]
                fn pxout_toggle(&self, bits: u8) {
                    unsafe { self.$pxout().toggle_bits(|w| w.bits(bits)) };
                }

                // PxOUT, PxDIR, PxREN, PxSEL0 and PxSEL1 (SLAU445I Tables 8-10 to 8-14, p. 334 to p. 335)
                reg_methods!($pxout, pxout_rd, pxout_wr, pxout_set, pxout_clear);
                reg_methods!($pxdir, pxdir_rd, pxdir_wr, pxdir_set, pxdir_clear);
                reg_methods!($pxren, pxren_rd, pxren_wr, pxren_set, pxren_clear);
                reg_methods!($pxsel0, pxsel0_rd, pxsel0_wr, pxsel0_set, pxsel0_clear);
                reg_methods!($pxsel1, pxsel1_rd, pxsel1_wr, pxsel1_set, pxsel1_clear);
            }

            $(
                impl IntrPeriph for pac::$Px {
                    // PxIES, PxIE and PxIFG (SLAU445I Tables 8-16 to 8-18, p. 336 to p. 337)
                    reg_methods!($pxies, pxies_rd, pxies_wr, pxies_set, pxies_clear);
                    reg_methods!($pxie, pxie_rd, pxie_wr, pxie_set, pxie_clear);
                    reg_methods!($pxifg, pxifg_rd, pxifg_wr, pxifg_set, pxifg_clear);

                    // PxIV (SLAU445I Tables 8-5 to 8-8, p. 332 to p. 333)
                    #[inline(always)]
                    fn pxiv_rd(&self) -> GpioVector {
                        let r = self.$pxiv().read();
                        let iv = r.$pxiv();
                        if iv.is_ifg0() { GpioVector::Pin0Isr }
                        else if iv.is_ifg1() { GpioVector::Pin1Isr }
                        else if iv.is_ifg2() { GpioVector::Pin2Isr }
                        else if iv.is_ifg3() { GpioVector::Pin3Isr }
                        else if iv.is_ifg4() { GpioVector::Pin4Isr }
                        else if iv.is_ifg5() { GpioVector::Pin5Isr }
                        else if iv.is_ifg6() { GpioVector::Pin6Isr }
                        else if iv.is_ifg7() { GpioVector::Pin7Isr }
                        else { GpioVector::NoIsr }
                    }
                }
            )?
        }
    };
}
pub(crate) use gpio_impl;
