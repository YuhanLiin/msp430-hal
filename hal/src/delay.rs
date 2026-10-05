//! Embedded hal delay implementation
//!
//! The delays count MCLK cycles in a loop, so they last at least as long as requested; interrupts
//! during a delay make it longer. A microsecond delay takes a multiplication besides, about 60 MCLK
//! cycles in all (80 with an FRAM wait state), which is 12 µs at 5 MHz and 5 µs at 16 MHz
//! (measured on an MSP430FR2476).

/// Delay provider struct
#[derive(Copy, Clone)]
pub struct SysDelay {
    /// Loop iterations per millisecond
    iters_per_ms: u16,
    /// Loop iterations per microsecond, in 1/65536ths, so a product's high word is whole iterations
    iters_per_us_q16: u32,
}

/// MCLK cycles per iteration of the delay loop: `SUB #1, Rn` takes 1 cycle and `JNZ` 2 (SLAU445I
/// 4.5.1.5, p. 154 to p. 155). `SUB #1, Rn` takes the 1 cycle of a register source (SLAU445I
/// Table 4-10, p. 155, Rn to Rm), as the constant generator supplies #1 with "No code memory
/// access required to retrieve the constant" (SLAU445I 4.3.4, p. 131). "All jump instructions
/// require one code word and take two CPU cycles to execute" (SLAU445I 4.5.1.5.3, p. 155).
const CYCLES_PER_ITER: u32 = 3;

impl SysDelay {
    /// Create a new delay object for an MCLK of `freq` Hz
    #[inline(always)]
    pub(crate) fn new(freq: u32) -> Self {
        // Round up, so delays don't fall short. The clock could be REFOCLK or VLOCLK (SELMS,
        // SLAU445I Table 3-8, p. 117), so be careful of small frequencies.
        let iters_per_ms = freq.div_ceil(1000 * CYCLES_PER_ITER).max(1) as u16;
        // At most 24000 * 65536 before the division, which fits (MCLK is at most 24 MHz: SLASEC4D
        // 5.3, p. 27)
        let iters_per_us_q16 = (freq.div_ceil(1000) << 16).div_ceil(1000 * CYCLES_PER_ITER);
        SysDelay { iters_per_ms, iters_per_us_q16 }
    }

    /// Spin for `iters` iterations of [`CYCLES_PER_ITER`] cycles
    #[inline(always)]
    fn spin(iters: u16) {
        if iters == 0 {
            return;
        }
        unsafe {
            core::arch::asm!(
                "1:",
                "sub #1, {n}",
                "jnz 1b",
                n = inout(reg) iters => _,
                options(nomem, nostack),
            );
        }
    }

    #[inline]
    fn ms(&self, mut ms: u32) {
        while ms > 0 {
            Self::spin(self.iters_per_ms);
            ms -= 1;
        }
    }

    /// Spin for `us` microseconds, which is below 1000. This takes a single multiplication, to
    /// keep the time the call itself takes short.
    #[inline]
    fn short_us(&self, us: u16) {
        // At most 999 * 524288 + 65535 for a 24 MHz MCLK, which fits (MCLK is at most 24 MHz:
        // SLASEC4D 5.3, p. 27). The wrapping operations leave out the overflow checks of debug
        // builds.
        let iters_q16 = (us as u32).wrapping_mul(self.iters_per_us_q16).wrapping_add(0xFFFF);
        Self::spin((iters_q16 >> 16) as u16);
    }

    #[inline]
    fn us(&self, mut us: u32) {
        // Whole milliseconds first, counted off without a division
        while us >= 1000 {
            Self::spin(self.iters_per_ms);
            us -= 1000;
        }
        self.short_us(us as u16);
    }
}

/// Busy-wait for at least `cycles` MCLK cycles, like TI's `__delay_cycles()`
#[inline]
pub(crate) fn delay_cycles(cycles: u32) {
    let mut iters = cycles.div_ceil(CYCLES_PER_ITER);
    // Tested at the end, so that a constant count that fits in one spin compiles to just that spin
    loop {
        let chunk = iters.min(u16::MAX as u32);
        SysDelay::spin(chunk as u16);
        iters -= chunk;
        if iters == 0 {
            break;
        }
    }
}

mod ehal1 {
    use super::*;
    use embedded_hal::delay::DelayNs;

    impl DelayNs for SysDelay {
        /// Pauses execution for at least `ns` nanoseconds, rounded up to whole microseconds. The call
        /// itself takes about 60 MCLK cycles (measured on an MSP430FR2476), see the
        /// [module documentation](crate::delay).
        #[inline]
        fn delay_ns(&mut self, ns: u32) {
            if ns <= 1000 {
                self.us(1);
            } else {
                self.us(ns.div_ceil(1000));
            }
        }

        /// Pauses execution for at least `us` microseconds. The call itself takes about 60 MCLK
        /// cycles (measured on an MSP430FR2476), see the [module documentation](crate::delay).
        #[inline]
        fn delay_us(&mut self, us: u32) { self.us(us) }

        /// Pauses execution for at least `ms` milliseconds (3 MCLK cycles per loop iteration, see
        /// `CYCLES_PER_ITER`: SLAU445I 4.5.1.5, p. 154 to p. 155).
        #[inline]
        fn delay_ms(&mut self, ms: u32) { self.ms(ms) }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::blocking::delay::{DelayMs, DelayUs};

    macro_rules! impl_delay {
        ($typ: ty) => {
            impl DelayMs<$typ> for SysDelay {
                #[inline]
                fn delay_ms(&mut self, ms: $typ) {
                    for _ in 0..ms {
                        SysDelay::spin(self.iters_per_ms);
                    }
                }
            }

            impl DelayUs<$typ> for SysDelay {
                /// The call itself takes about 60 MCLK cycles (measured on an MSP430FR2476), see
                /// the [module documentation](crate::delay).
                #[inline]
                fn delay_us(&mut self, us: $typ) {
                    if us > 0 {
                        self.us(us as u32);
                    }
                }
            }
        };
    }

    impl_delay!(u8);
    impl_delay!(u16);
    impl_delay!(u32);

    // A delay implementation for the default literal type to allow calls like `delay_ms(100)`
    // Negative durations are treated as zero.
    impl_delay!(i32);
}
