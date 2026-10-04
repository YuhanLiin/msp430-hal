pub use msp430fr247x as pac;

/// PAC with standardised peripheral names. For the 247x this is just the PAC.
pub use msp430fr247x as _pac;
/*         GPIO          */
pub mod gpio {
    // Make PAC GPIO avilable as a re-export
    pub use crate::pac::{P1, P2, P3, P4, P5, P6};

    use crate::gpio::*;
    use crate::hw_traits::gpio::gpio_impl;

    // Define alternate pin transitions
    //
    // Alternate1, 2 and 3 are PxSELx (PxSEL1/PxSEL0) = 01, 10 and 11, the primary, secondary and
    // tertiary module functions (SLAU445I Table 8-3, p. 314). Each impl is a row of the port's pin
    // function table (SLASEO7C Tables 9-23 to 9-28, p. 65 to p. 70). PxSEL doesn't set the direction
    // (SLAU445I 8.2.5, p. 314), so a function the table lists with one PxDIR value is only given to
    // `Input` (PxDIR = 0) or `Output` (PxDIR = 1) pins. A timer pin's direction picks the capture input
    // (CCIxA, in) or the compare output (out). An eUSCI or TA2/TA3 signal only reaches the pin in its
    // mapping, set by USCIB0RMP (SYSCFG2, SLAU445I Table 1-31, p. 82) or USCIA0RMP, USCIB1RMP, TA2RMP
    // and TA3RMP (SYSCFG3, SLAU445I Table 1-32, p. 83): "Only one selected port is valid at any time"
    // (SLASEO7C Table 9-11 notes 1 and 2, p. 54; SLASEO7C Table 9-16 notes 1 and 2, p. 60).

    // P1 alternate 1, P1SELx = 01 (SLASEO7C Table 9-23, p. 65): P1.0 UCB0STE, P1.1 UCB0CLK,
    // P1.2 UCB0SIMO/UCB0SDA, P1.3 UCB0SOMI/UCB0SCL (USCIB0RMP = 0), P1.4 UCA0TXD/UCA0SIMO,
    // P1.5 UCA0RXD/UCA0SOMI, P1.6 UCA0CLK, P1.7 UCA0STE (USCIA0RMP = 0)
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2, P1SELx = 10 (SLASEO7C Table 9-23, p. 65)
    impl<PULL> ToAlternate2 for Pin<P1, Pin0, Input<PULL>> {} // 10: TA0CLK, in (P1DIR.0 = 0)
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}         // 10: TA0.CCI1A in / TA0.1 out
    impl<DIR>  ToAlternate2 for Pin<P1, Pin2, DIR> {}         // 10: TA0.CCI2A in / TA0.2 out
    impl ToAlternate2       for Pin<P1, Pin3, Output> {}      // 10: MCLK, out (P1DIR.3 = 1)
    impl<DIR>  ToAlternate2 for Pin<P1, Pin4, DIR> {}         // 10: TA1.CCI2A in / TA1.2 out
    impl<DIR>  ToAlternate2 for Pin<P1, Pin5, DIR> {}         // 10: TA1.CCI1A in / TA1.1 out
    impl<PULL> ToAlternate2 for Pin<P1, Pin6, Input<PULL>> {} // 10: TA1CLK, in (P1DIR.6 = 0)
    impl ToAlternate2       for Pin<P1, Pin7, Output> {}      // 10: SMCLK, out (P1DIR.7 = 1)
    // P1 alternate 3, P1SELx = 11 (SLASEO7C Table 9-23, p. 65): analog inputs A0 to A7 on P1.0 to
    // P1.7, with Veref+ on P1.0, COMP0.0 on P1.1, Veref- on P1.2 and VREF+ on P1.4
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1, P2SELx = 01 (SLASEO7C Table 9-24, p. 66). P2.2 has no 01 function.
    impl<DIR> ToAlternate1 for Pin<P2, Pin0, DIR> {} // 01: XOUT
    impl<DIR> ToAlternate1 for Pin<P2, Pin1, DIR> {} // 01: XIN
    impl<DIR> ToAlternate1 for Pin<P2, Pin3, DIR> {} // 01: TA2.CCI0A in / TA2.0 out, TA2RMP = 0
    impl<DIR> ToAlternate1 for Pin<P2, Pin4, DIR> {} // 01: UCA1CLK
    impl<DIR> ToAlternate1 for Pin<P2, Pin5, DIR> {} // 01: UCA1RXD/UCA1SOMI
    impl<DIR> ToAlternate1 for Pin<P2, Pin6, DIR> {} // 01: UCA1TXD/UCA1SIMO
    impl<DIR> ToAlternate1 for Pin<P2, Pin7, DIR> {} // 01: UCB1STE, USCIB1RMP = 0
    // P2 alternate 2, P2SELx = 10 (SLASEO7C Table 9-24, p. 66)
    impl ToAlternate2 for Pin<P2, Pin2, Output> {} // 10: ACLK, out (P2DIR.2 = 1)
    // P2 alternate 3, P2SELx = 11 (SLASEO7C Table 9-24, p. 66)
    impl<DIR> ToAlternate3 for Pin<P2, Pin2, DIR> {} // 11: COMP0.1

    // P3 alternate 1, P3SELx = 01 (SLASEO7C Table 9-25, p. 67)
    impl<DIR>  ToAlternate1 for Pin<P3, Pin0, DIR> {}         // 01: TA2.CCI2A in / TA2.2 out, TA2RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P3, Pin1, DIR> {}         // 01: UCA1STE
    impl<DIR>  ToAlternate1 for Pin<P3, Pin2, DIR> {}         // 01: UCB1SIMO/UCB1SDA, USCIB1RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P3, Pin3, DIR> {}         // 01: TA2.CCI1A in / TA2.1 out, TA2RMP = 0
    impl<PULL> ToAlternate1 for Pin<P3, Pin4, Input<PULL>> {} // 01: TA2CLK, in (P3DIR.4 = 0), TA2RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P3, Pin5, DIR> {}         // 01: UCB1CLK, USCIB1RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P3, Pin6, DIR> {}         // 01: UCB1SOMI/UCB1SCL, USCIB1RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P3, Pin7, DIR> {}         // 01: TA3.CCI2A in / TA3.2 out, TA3RMP = 0
    // P3 alternate 2, P3SELx = 10 (SLASEO7C Table 9-25, p. 67)
    impl ToAlternate2       for Pin<P3, Pin4, Output> {}      // 10: COMP0OUT, out (P3DIR.4 = 1)
    impl<PULL> ToAlternate2 for Pin<P3, Pin5, Input<PULL>> {} // 10: TB0TRG, in (P3DIR.5 = 0)


    // P4 alternate 1, P4SELx = 01 (SLASEO7C Table 9-26, p. 68)
    impl<DIR>  ToAlternate1 for Pin<P4, Pin0, DIR> {}         // 01: TA3.CCI1A in / TA3.1 out, TA3RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P4, Pin1, DIR> {}         // 01: TA3.CCI0A in / TA3.0 out, TA3RMP = 0
    impl<PULL> ToAlternate1 for Pin<P4, Pin2, Input<PULL>> {} // 01: TA3CLK, in (P4DIR.2 = 0), TA3RMP = 0
    impl<DIR>  ToAlternate1 for Pin<P4, Pin3, DIR> {}         // 01: UCB1SOMI/UCB1SCL, USCIB1RMP = 1
    impl<DIR>  ToAlternate1 for Pin<P4, Pin4, DIR> {}         // 01: UCB1SIMO/UCB1SDA, USCIB1RMP = 1
    impl<DIR>  ToAlternate1 for Pin<P4, Pin5, DIR> {}         // 01: UCB0SOMI/UCB0SCL, USCIB0RMP = 1
    impl<DIR>  ToAlternate1 for Pin<P4, Pin6, DIR> {}         // 01: UCB0SIMO/UCB0SDA, USCIB0RMP = 1
    impl<DIR>  ToAlternate1 for Pin<P4, Pin7, DIR> {}         // 01: UCA0STE, USCIA0RMP = 1
    // P4 alternate 2, P4SELx = 10 (SLASEO7C Table 9-26, p. 68)
    impl<DIR> ToAlternate2 for Pin<P4, Pin3, DIR> {} // 10: TB0.CCI5A in / TB0.5 out
    impl<DIR> ToAlternate2 for Pin<P4, Pin4, DIR> {} // 10: TB0.CCI6A in / TB0.6 out
    impl<DIR> ToAlternate2 for Pin<P4, Pin5, DIR> {} // 10: TA3.CCI2A in / TA3.2 out, TA3RMP = 1
    impl<DIR> ToAlternate2 for Pin<P4, Pin6, DIR> {} // 10: TA3.CCI1A in / TA3.1 out, TA3RMP = 1
    impl<DIR> ToAlternate2 for Pin<P4, Pin7, DIR> {} // 10: TB0.CCI1A in / TB0.1 out
    // P4 alternate 3, P4SELx = 11 (SLASEO7C Table 9-26, p. 68)
    impl<DIR> ToAlternate3 for Pin<P4, Pin3, DIR> {} // 11: A8
    impl<DIR> ToAlternate3 for Pin<P4, Pin4, DIR> {} // 11: A9

    // P5 alternate 1, P5SELx = 01 (SLASEO7C Table 9-27, p. 69): P5.0 UCA0CLK, P5.1 UCA0RXD/UCA0SOMI,
    // P5.2 UCA0TXD/UCA0SIMO (USCIA0RMP = 1), P5.3 UCB1CLK, P5.4 UCB1STE (USCIB1RMP = 1), P5.5 UCB0CLK,
    // P5.6 UCB0STE (USCIB0RMP = 1), P5.7 TA2.CCI1A in / TA2.1 out (TA2RMP = 1)
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P5, PIN, DIR> {}
    // P5 alternate 2, P5SELx = 10 (SLASEO7C Table 9-27, p. 69, whose P5.3 and P5.4 values revision C
    // corrected: SLASEO7C 5, p. 5)
    impl<DIR>  ToAlternate2 for Pin<P5, Pin0, DIR> {}         // 10: TB0.CCI2A in / TB0.2 out
    impl<DIR>  ToAlternate2 for Pin<P5, Pin1, DIR> {}         // 10: TB0.CCI3A in / TB0.3 out
    impl<DIR>  ToAlternate2 for Pin<P5, Pin2, DIR> {}         // 10: TB0.CCI4A in / TB0.4 out
    impl<DIR>  ToAlternate2 for Pin<P5, Pin3, DIR> {}         // 10: TA3.CCI0A in / TA3.0 out, TA3RMP = 1
    impl<PULL> ToAlternate2 for Pin<P5, Pin4, Input<PULL>> {} // 10: TA3CLK, in (P5DIR.4 = 0), TA3RMP = 1
    impl<PULL> ToAlternate2 for Pin<P5, Pin5, Input<PULL>> {} // 10: TA2CLK, in (P5DIR.5 = 0), TA2RMP = 1
    impl<DIR>  ToAlternate2 for Pin<P5, Pin6, DIR> {}         // 10: TA2.CCI0A in / TA2.0 out, TA2RMP = 1
    // P5 alternate 3, P5SELx = 11 (SLASEO7C Table 9-27, p. 69)
    impl<DIR> ToAlternate3 for Pin<P5, Pin3, DIR> {} // 11: A10
    impl<DIR> ToAlternate3 for Pin<P5, Pin4, DIR> {} // 11: A11
    impl<DIR> ToAlternate3 for Pin<P5, Pin7, DIR> {} // 11: COMP0.2

    // P6 alternate 1, P6SELx = 01 (SLASEO7C Table 9-28, p. 70)
    impl<DIR>  ToAlternate1 for Pin<P6, Pin0, DIR> {}         // 01: TA2.CCI2A in / TA2.2 out, TA2RMP = 1
    impl<PULL> ToAlternate1 for Pin<P6, Pin1, Input<PULL>> {} // 01: TB0CLK, in (P6DIR.1 = 0)
    impl<DIR>  ToAlternate1 for Pin<P6, Pin2, DIR> {}         // 01: TB0.CCI0A in / TB0.0 out
    // P6 alternate 3, P6SELx = 11 (SLASEO7C Table 9-28, p. 70)
    impl<DIR> ToAlternate3 for Pin<P6, Pin0, DIR> {} // 11: COMP0.3

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable. Every port has
    // PxSELC and the interrupt registers (SLASEO7C Tables 9-40 to 9-42, p. 75 to p. 77) and its own
    // interrupt vector (SLASEO7C Table 9-2, p. 47).
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
    gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
    gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1, [p5ies, p5ie, p5ifg, p5iv]);
    gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1, [p6ies, p6ie, p6ifg, p6iv]);

    // Pins per port (SLASEO7C 9.10.3, p. 51: "P1, P3, P4, and P5 implement 8 bits each. P2 implements
    // 6 bits excluding the I/Os multiplexed with XIN and XOUT. P6 implements 3 bits."). The pins a port
    // lacks are always the top ones: P6 has P6.0 to P6.2 (SLASEO7C Table 9-28, p. 70).
    impl_port_pins!(P1, 8);
    impl_port_pins!(P2, 8);
    impl_port_pins!(P3, 8);
    impl_port_pins!(P4, 8);
    impl_port_pins!(P5, 8);
    impl_port_pins!(P6, 3);
}

/* ADC */
mod adc {
    use crate::{adc::*, gpio::*};

    impl_adc_channel_pin!(P1, Pin0, Alternate3 => 0);
    impl_adc_channel_pin!(P1, Pin1, Alternate3 => 1);
    impl_adc_channel_pin!(P1, Pin2, Alternate3 => 2);
    impl_adc_channel_pin!(P1, Pin3, Alternate3 => 3);
    impl_adc_channel_pin!(P1, Pin4, Alternate3 => 4);
    impl_adc_channel_pin!(P1, Pin5, Alternate3 => 5);
    impl_adc_channel_pin!(P1, Pin6, Alternate3 => 6);
    impl_adc_channel_pin!(P1, Pin7, Alternate3 => 7);
    impl_adc_channel_pin!(P4, Pin3, Alternate3 => 8);
    impl_adc_channel_pin!(P4, Pin4, Alternate3 => 9);
    impl_adc_channel_pin!(P5, Pin3, Alternate3 => 10);
    impl_adc_channel_pin!(P5, Pin4, Alternate3 => 11);
}

/* Backup Memory */
/// Size of the Backup Memory segment on this device, in bytes (SLASEO7C 9.10.10, p. 61: "This device
/// provides up to 32 bytes that are retained during LPM3.5"; BAKMEM0 to BAKMEM15:
/// SLASEO7C Table 9-54, p. 81)
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::{CapturePeriph, NoCapturePin}, gpio::*, pac::*, pin_mapping::*};

    // The capture input A (CCIxA) pin of each capture/compare register, from the timer signal
    // connections (SLASEO7C Tables 9-12 to 9-16, p. 55 to p. 60) and the pin function tables
    // (SLASEO7C Tables 9-23 to 9-28, p. 65 to p. 70). `()` is a register without a CCIxA pin, or one the
    // timer doesn't have: TA0 to TA3 have CCR0 to CCR2 (SLASEO7C 9.10.8, p. 55), TB0 has CCR0 to CCR6
    // (SLASEO7C Table 9-15, p. 59).

    // TA0 (SLASEO7C Table 9-12, p. 55, titled Timer0_A0; pins: SLASEO7C Table 9-23, p. 65): CCI0A is
    // ACLK, internal
    impl CapturePeriph for Ta0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin1, Alternate2<Input<Floating>>>; // P1.1 TA0.CCI1A, P1SELx = 10
        type Gpio2 = Pin<P1, Pin2, Alternate2<Input<Floating>>>; // P1.2 TA0.CCI2A, P1SELx = 10
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TA1 (SLASEO7C Table 9-13, p. 56, titled Timer0_A1; pins: SLASEO7C Table 9-23, p. 65): CCI0A isn't
    // connected (N/A), so input A of capture pin 0 is `NoCapturePin`, which can't be selected
    impl CapturePeriph for Ta1 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = Pin<P1, Pin5, Alternate2<Input<Floating>>>; // P1.5 TA1.CCI1A, P1SELx = 10
        type Gpio2 = Pin<P1, Pin4, Alternate2<Input<Floating>>>; // P1.4 TA1.CCI2A, P1SELx = 10
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TA2, default pins, TA2RMP = 0 (SLASEO7C Table 9-14, p. 58; SLASEO7C Table 9-16, p. 60; pins:
    // SLASEO7C Table 9-24, p. 66; SLASEO7C Table 9-25, p. 67)
    impl CapturePeriph<DefaultMapping> for Ta2 {
        type Gpio0 = Pin<P2, Pin3, Alternate1<Input<Floating>>>; // P2.3 TA2.CCI0A, P2SELx = 01, TA2RMP = 0
        type Gpio1 = Pin<P3, Pin3, Alternate1<Input<Floating>>>; // P3.3 TA2.CCI1A, P3SELx = 01, TA2RMP = 0
        type Gpio2 = Pin<P3, Pin0, Alternate1<Input<Floating>>>; // P3.0 TA2.CCI2A, P3SELx = 01, TA2RMP = 0
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
    // TA2, remapped pins, TA2RMP = 1 (SLASEO7C Table 9-16, p. 60; pins: SLASEO7C Table 9-27, p. 69;
    // SLASEO7C Table 9-28, p. 70)
    impl CapturePeriph<RemappedMapping> for Ta2 {
        type Gpio0 = Pin<P5, Pin6, Alternate2<Input<Floating>>>; // P5.6 TA2.CCI0A, P5SELx = 10, TA2RMP = 1
        type Gpio1 = Pin<P5, Pin7, Alternate1<Input<Floating>>>; // P5.7 TA2.CCI1A, P5SELx = 01, TA2RMP = 1
        type Gpio2 = Pin<P6, Pin0, Alternate1<Input<Floating>>>; // P6.0 TA2.CCI2A, P6SELx = 01, TA2RMP = 1
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TA3, default pins, TA3RMP = 0 (SLASEO7C Table 9-14, p. 58; SLASEO7C Table 9-16, p. 60; pins:
    // SLASEO7C Table 9-25, p. 67; SLASEO7C Table 9-26, p. 68)
    impl CapturePeriph<DefaultMapping> for Ta3 {
        type Gpio0 = Pin<P4, Pin1, Alternate1<Input<Floating>>>; // P4.1 TA3.CCI0A, P4SELx = 01, TA3RMP = 0
        type Gpio1 = Pin<P4, Pin0, Alternate1<Input<Floating>>>; // P4.0 TA3.CCI1A, P4SELx = 01, TA3RMP = 0
        type Gpio2 = Pin<P3, Pin7, Alternate1<Input<Floating>>>; // P3.7 TA3.CCI2A, P3SELx = 01, TA3RMP = 0
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
    // TA3, remapped pins, TA3RMP = 1 (SLASEO7C Table 9-16, p. 60; pins: SLASEO7C Table 9-26, p. 68;
    // SLASEO7C Table 9-27, p. 69)
    impl CapturePeriph<RemappedMapping> for Ta3 {
        type Gpio0 = Pin<P5, Pin3, Alternate2<Input<Floating>>>; // P5.3 TA3.CCI0A, P5SELx = 10, TA3RMP = 1
        type Gpio1 = Pin<P4, Pin6, Alternate2<Input<Floating>>>; // P4.6 TA3.CCI1A, P4SELx = 10, TA3RMP = 1
        type Gpio2 = Pin<P4, Pin5, Alternate2<Input<Floating>>>; // P4.5 TA3.CCI2A, P4SELx = 10, TA3RMP = 1
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TB0 (SLASEO7C Table 9-15, p. 59, which names the CCR3 to CCR6 inputs CCI1A;
    // SLASEO7C Table 9-26, p. 68, SLASEO7C Table 9-27, p. 69 and SLASEO7C Table 7-2, p. 18 give CCI3A
    // to CCI6A; pins also: SLASEO7C Table 9-28, p. 70)
    impl CapturePeriph for Tb0 {
        type Gpio0 = Pin<P6, Pin2, Alternate1<Input<Floating>>>; // P6.2 TB0.CCI0A, P6SELx = 01
        type Gpio1 = Pin<P4, Pin7, Alternate2<Input<Floating>>>; // P4.7 TB0.CCI1A, P4SELx = 10
        type Gpio2 = Pin<P5, Pin0, Alternate2<Input<Floating>>>; // P5.0 TB0.CCI2A, P5SELx = 10
        type Gpio3 = Pin<P5, Pin1, Alternate2<Input<Floating>>>; // P5.1 TB0.CCI3A, P5SELx = 10
        type Gpio4 = Pin<P5, Pin2, Alternate2<Input<Floating>>>; // P5.2 TB0.CCI4A, P5SELx = 10
        type Gpio5 = Pin<P4, Pin3, Alternate2<Input<Floating>>>; // P4.3 TB0.CCI5A, P4SELx = 10
        type Gpio6 = Pin<P4, Pin4, Alternate2<Input<Floating>>>; // P4.4 TB0.CCI6A, P4SELx = 10
    }
}

/* Clocks */
/// MODCLK frequency
pub const MODCLK_FREQ_HZ: u32 = 5_000_000;

/* eCOMP */
pub mod ecomp {
    use core::convert::Infallible;

    use crate::hw_traits::ecomp::*;
    use crate::pac::EComp0;
    use crate::{ecomp::*, gpio::*};

    // eCOMP0 pins and channels (SLASEO7C Tables 9-21 and 9-22, p. 63; pin functions:
    // SLASEO7C Tables 9-23 to 9-28, p. 65 to p. 70). CPPSEL and CPNSEL 000b and 001b are external
    // inputs, 010b to 101b are device specific and 110b is the DAC (SLAU445I Table 18-2, p. 509). Here
    // 010b is the low-power 1.2 V reference, 011b and 100b are COMP0.2 and COMP0.3, and 101b is N/A,
    // so the device-specific inputs other than the reference are unused.
    impl ECompInputs for EComp0 {
        type COMPx_0   = Pin<P1, Pin1, Alternate3<Input<Floating>>>; // P1.1 COMP0.0, P1SELx = 11
        type COMPx_1   = Pin<P2, Pin2, Alternate3<Input<Floating>>>; // P2.2 COMP0.1, P2SELx = 11
        type COMPx_2   = Pin<P5, Pin7, Alternate3<Input<Floating>>>; // P5.7 COMP0.2, P5SELx = 11
        type COMPx_3   = Pin<P6, Pin0, Alternate3<Input<Floating>>>; // P6.0 COMP0.3, P6SELx = 11
        type COMPx_Out = Pin<P3, Pin4, Alternate2<Output>>;          // P3.4 COMP0OUT, P3SELx = 10, output

        type DeviceSpecific0    = (); // Internal 1.2V reference (010b). No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        type DeviceSpecific3Pos = Infallible; // Not used
        type DeviceSpecific3Neg = Infallible; // Not used
    }

    /// List of possible inputs to the positive input of an eCOMP comparator
    /// (CPPSEL, SLASEO7C Table 9-21, p. 63). The DAC option takes a reference to ensure it has been
    /// configured.
    #[allow(non_camel_case_types)]
    pub enum PositiveInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.1 for COMP0 (000b)
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P2.2 for COMP0 (001b)
        COMPx_1(COMP::COMPx_1),
        /// COMPx.2. P5.7 for COMP0 (011b)
        COMPx_2(COMP::COMPx_2),
        /// COMPx.3. P6.0 for COMP0 (100b)
        COMPx_3(COMP::COMPx_3),
        /// Internal 1.2V reference (010b), the low-power 1.2 V reference
        _1V2,
        /// This eCOMP's internal 6-bit DAC (110b)
        ///
        /// Requires a reference to ensure that it has been configured.
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> PositiveInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cppsel(&self) -> u8 {
            // CPPSEL (SLASEO7C Table 9-21, p. 63): the 1.2 V reference takes 010b, so COMP0.2 and COMP0.3
            // are 011b and 100b
            match self {
                PositiveInput::COMPx_0(_) => 0b000,
                PositiveInput::COMPx_1(_) => 0b001,
                PositiveInput::COMPx_2(_) => 0b011,
                PositiveInput::COMPx_3(_) => 0b100,
                PositiveInput::_1V2       => 0b010,
                PositiveInput::Dac(_)     => 0b110,
            }
        }
    }

    /// List of possible inputs to the negative input of an eCOMP comparator
    /// (CPNSEL, SLASEO7C Table 9-21, p. 63). The DAC option takes a reference to ensure it has been
    /// configured.
    #[allow(non_camel_case_types)]
    pub enum NegativeInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.1 for COMP0 (000b)
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P2.2 for COMP0 (001b)
        COMPx_1(COMP::COMPx_1),
        /// COMPx.2. P5.7 for COMP0 (011b)
        COMPx_2(COMP::COMPx_2),
        /// COMPx.3. P6.0 for COMP0 (100b)
        COMPx_3(COMP::COMPx_3),
        /// Internal 1.2V reference (010b), the low-power 1.2 V reference
        _1V2,
        /// This eCOMP's internal 6-bit DAC (110b)
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> NegativeInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cpnsel(&self) -> u8 {
            // CPNSEL, the same channels as CPPSEL (SLASEO7C Table 9-21, p. 63)
            match self {
                NegativeInput::COMPx_0(_) => 0b000,
                NegativeInput::COMPx_1(_) => 0b001,
                NegativeInput::COMPx_2(_) => 0b011,
                NegativeInput::COMPx_3(_) => 0b100,
                NegativeInput::_1V2       => 0b010,
                NegativeInput::Dac(_)     => 0b110,
            }
        }
    }

    // eCOMP0 registers (SLASEO7C Table 9-56, p. 82). That table leaves out CP0DACDATA, which is at offset
    // 12h (SLAU445I Table 18-1, p. 508).
    impl_ecomp!(EComp0, cp0ctl0, cp0ctl1, cp0dacctl, cp0dacdata, cp0int, cp0iv);
}

/* eUSCI */
mod eusci {
    use crate::{
        hw_traits::{eusci::*, Steal},
        pac::*,
    };

    // eUSCI_A0, eUSCI_A1, eUSCI_B0 and eUSCI_B1 (SLASEO7C 9.10.7, p. 54; SLASEO7C Table 9-11, p. 54;
    // registers: SLASEO7C Tables 9-50 to 9-53, p. 79 to p. 81)
    eusci_steal_impl!(EUsciA0);
    eusci_steal_impl!(EUsciA1);
    eusci_steal_impl!(EUsciB0);
    eusci_steal_impl!(EUsciB1);
}

/* I2C */
mod i2c {
    use crate::{
        gpio::*,
        hw_traits::eusci::*,
        i2c::{impl_i2c_pin, I2cUsci},
        pac::*,
        pin_mapping::*,
    };

    // "The eUSCI_B module supports either SPI or I2C communications" (SLASEO7C 9.10.7, p. 54); registers:
    // SLASEO7C Tables 9-52 and 9-53, p. 80 to p. 81.
    // eUSCI_B0 registers: SLASEO7C Table 9-52, p. 80; in I2C mode: SLAU445I Table 24-3, p. 648
    eusci_i2c_impl!(
        EUsciB0,
        ucb0ctlw0,
        ucb0ctlw1,
        ucb0brw,
        ucb0statw,
        ucb0tbcnt,
        ucb0rxbuf,
        ucb0txbuf,
        ucb0i2coa0,
        ucb0i2coa1,
        ucb0i2coa2,
        ucb0i2coa3,
        ucb0addrx,
        ucb0addmask,
        ucb0i2csa,
        ucb0ie,
        ucb0ifg,
        ucb0iv,
        crate::pac::e_usci_b0::ucb0ifg::R,
    );
    // eUSCI_B1 registers: SLASEO7C Table 9-53, p. 80 to p. 81; in I2C mode: SLAU445I Table 24-3, p. 648
    eusci_i2c_impl!(
        EUsciB1,
        ucb1ctlw0,
        ucb1ctlw1,
        ucb1brw,
        ucb1statw,
        ucb1tbcnt,
        ucb1rxbuf,
        ucb1txbuf,
        ucb1i2coa0,
        ucb1i2coa1,
        ucb1i2coa2,
        ucb1i2coa3,
        ucb1addrx,
        ucb1addmask,
        ucb1i2csa,
        ucb1ie,
        ucb1ifg,
        ucb1iv,
        crate::pac::e_usci_b1::ucb1ifg::R,
    );
    // I2C pins (SLASEO7C Table 9-11, p. 54), all PxSELx = 01, which `impl_i2c_pin!` takes as Alternate1
    // (SLASEO7C Table 9-23, p. 65; SLASEO7C Tables 9-25 to 9-27, p. 67 to p. 69). The default pins need
    // USCIBxRMP = 0, the remapped ones USCIBxRMP = 1. UCLKI, the external clock that UCSSELx = 00b
    // selects (SLAU445I Table 24-4, p. 649), is the "Externally provided clock on the eUSCI_B SPI clock
    // input pin", UCBxCLK (SLAU445I Figure 24-1, p. 628).

    /// I2C SCL pin for eUSCI B0 (default mapping)
    ///
    /// P1.3 in its UCB0SOMI/UCB0SCL function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0SCLPinDefault;
    impl_i2c_pin!(UsciB0SCLPinDefault, P1, Pin3);

    /// I2C SCL pin for eUSCI B0 (remapped mapping)
    ///
    /// P4.5 in its UCB0SOMI/UCB0SCL function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0SCLPinRemapped;
    impl_i2c_pin!(UsciB0SCLPinRemapped, P4, Pin5);

    /// I2C SDA pin for eUSCI B0 (default mapping)
    ///
    /// P1.2 in its UCB0SIMO/UCB0SDA function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0SDAPinDefault;
    impl_i2c_pin!(UsciB0SDAPinDefault, P1, Pin2);

    /// I2C SDA pin for eUSCI B0 (remapped mapping)
    ///
    /// P4.6 in its UCB0SIMO/UCB0SDA function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0SDAPinRemapped;
    impl_i2c_pin!(UsciB0SDAPinRemapped, P4, Pin6);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (default mapping)
    ///
    /// P1.1 in its UCB0CLK function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0UCLKIPinDefault;
    impl_i2c_pin!(UsciB0UCLKIPinDefault, P1, Pin1);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (remapped mapping)
    ///
    /// P5.5 in its UCB0CLK function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0UCLKIPinRemapped;
    impl_i2c_pin!(UsciB0UCLKIPinRemapped, P5, Pin5);

    /// I2C SCL pin for eUSCI B1 (default mapping)
    ///
    /// P3.6 in its UCB1SOMI/UCB1SCL function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1SCLPinDefault;
    impl_i2c_pin!(UsciB1SCLPinDefault, P3, Pin6);

    /// I2C SCL pin for eUSCI B1 (remapped mapping)
    ///
    /// P4.3 in its UCB1SOMI/UCB1SCL function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1SCLPinRemapped;
    impl_i2c_pin!(UsciB1SCLPinRemapped, P4, Pin3);

    /// I2C SDA pin for eUSCI B1 (default mapping)
    ///
    /// P3.2 in its UCB1SIMO/UCB1SDA function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1SDAPinDefault;
    impl_i2c_pin!(UsciB1SDAPinDefault, P3, Pin2);

    /// I2C SDA pin for eUSCI B1 (remapped mapping)
    ///
    /// P4.4 in its UCB1SIMO/UCB1SDA function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1SDAPinRemapped;
    impl_i2c_pin!(UsciB1SDAPinRemapped, P4, Pin4);

    /// UCLKI pin for eUSCI B1. Used as an external clock source. (default mapping)
    ///
    /// P3.5 in its UCB1CLK function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1UCLKIPinDefault;
    impl_i2c_pin!(UsciB1UCLKIPinDefault, P3, Pin5);

    /// UCLKI pin for eUSCI B1. Used as an external clock source. (remapped mapping)
    ///
    /// P5.3 in its UCB1CLK function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1UCLKIPinRemapped;
    impl_i2c_pin!(UsciB1UCLKIPinRemapped, P5, Pin3);

    impl I2cUsci<DefaultMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinDefault;
        type DataPin = UsciB0SDAPinDefault;
        type ExternalClockPin = UsciB0UCLKIPinDefault;

        fn configure_pin_mapping() {
            // USCIB0RMP, SYSCFG2 bit 11, = 0: default pins (SLAU445I Table 1-31, p. 82)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().clear_bits(|w| w.uscib0rmp().clear_bit()) };
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinRemapped;
        type DataPin = UsciB0SDAPinRemapped;
        type ExternalClockPin = UsciB0UCLKIPinRemapped;

        fn configure_pin_mapping() {
            // USCIB0RMP, SYSCFG2 bit 11, = 1: remapped pins (SLAU445I Table 1-31, p. 82)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().set_bits(|w| w.uscib0rmp().set_bit()) };
        }
    }

    impl I2cUsci<DefaultMapping> for EUsciB1 {
        type ClockPin = UsciB1SCLPinDefault;
        type DataPin = UsciB1SDAPinDefault;
        type ExternalClockPin = UsciB1UCLKIPinDefault;

        fn configure_pin_mapping() {
            // USCIB1RMP, SYSCFG3 bit 4, = 0: default pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.uscib1rmp().clear_bit()) };
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB1 {
        type ClockPin = UsciB1SCLPinRemapped;
        type DataPin = UsciB1SDAPinRemapped;
        type ExternalClockPin = UsciB1UCLKIPinRemapped;

        fn configure_pin_mapping() {
            // USCIB1RMP, SYSCFG3 bit 4, = 1: remapped pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.uscib1rmp().set_bit()) };
        }
    }
}

/* Information Memory */
/// Size of the Information Memory segment on this device, in bytes
pub const INFO_MEM_SIZE: usize = 512;

/* PWM */
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // TA0
    impl PwmPeriph<CCR1> for Ta0 {
        type Gpio = Pin<P1, Pin1, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Ta0 {
        type Gpio = Pin<P1, Pin2, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TA1
    impl PwmPeriph<CCR1> for Ta1 {
        type Gpio = Pin<P1, Pin5, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Ta1 {
        type Gpio = Pin<P1, Pin4, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TA2
    impl PwmPeriph<CCR0> for Ta2 {
        type Gpio = Pin<P2, Pin3, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR1> for Ta2 {
        type Gpio = Pin<P3, Pin3, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Ta2 {
        type Gpio = Pin<P3, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }

    // TA3
    impl PwmPeriph<CCR0> for Ta3 {
        type Gpio = Pin<P4, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR1> for Ta3 {
        type Gpio = Pin<P4, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Ta3 {
        type Gpio = Pin<P3, Pin7, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TB0
    impl PwmPeriph<CCR0> for Tb0 {
        type Gpio = Pin<P6, Pin2, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR1> for Tb0 {
        type Gpio = Pin<P4, Pin7, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Tb0 {
        type Gpio = Pin<P5, Pin0, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR3> for Tb0 {
        type Gpio = Pin<P5, Pin1, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR4> for Tb0 {
        type Gpio = Pin<P5, Pin2, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR5> for Tb0 {
        type Gpio = Pin<P4, Pin3, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR6> for Tb0 {
        type Gpio = Pin<P4, Pin4, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, serial::*};

    // "The eUSCI_A module supports either UART or SPI communications" (SLASEO7C 9.10.7, p. 54);
    // registers: SLASEO7C Tables 9-50 and 9-51, p. 79 to p. 80.
    // eUSCI_A0 registers: SLASEO7C Table 9-50, p. 79 to p. 80; in UART mode: SLAU445I Table 22-7, p. 592
    eusci_uart_impl!(
        EUsciA0,
        uca0ctlw0,
        uca0ctlw1,
        uca0brw,
        uca0mctlw,
        uca0statw,
        uca0rxbuf,
        uca0txbuf,
        uca0abctl,
        uca0ie,
        uca0ifg,
        uca0iv,
        crate::pac::e_usci_a0::uca0statw::R
    );

    // eUSCI_A1 registers: SLASEO7C Table 9-51, p. 80; in UART mode: SLAU445I Table 22-7, p. 592
    eusci_uart_impl!(
        EUsciA1,
        uca1ctlw0,
        uca1ctlw1,
        uca1brw,
        uca1mctlw,
        uca1statw,
        uca1rxbuf,
        uca1txbuf,
        uca1abctl,
        uca1ie,
        uca1ifg,
        uca1iv,
        crate::pac::e_usci_a1::uca1statw::R
    );

    impl SerialUsci<DefaultMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinDefault;
        type TxPin = UsciA0TxPinDefault;
        type RxPin = UsciA0RxPinDefault;

        fn configure_pin_mapping() {
            // USCIA0RMP, SYSCFG3 bit 0, = 0: default pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.uscia0rmp().clear_bit()) };
        }
    }
    impl SerialUsci<RemappedMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinRemapped;
        type TxPin = UsciA0TxPinRemapped;
        type RxPin = UsciA0RxPinRemapped;

        fn configure_pin_mapping() {
            // USCIA0RMP, SYSCFG3 bit 0, = 1: remapped pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.uscia0rmp().set_bit()) };
        }
    }

    // eUSCI_A1 has one set of pins and no remap bit (SLASEO7C Table 9-11, p. 54)
    impl SerialUsci for EUsciA1 {
        type ClockPin = UsciA1ClockPin;
        type TxPin = UsciA1TxPin;
        type RxPin = UsciA1RxPin;
    }

    // UART pins (SLASEO7C Table 9-11, p. 54), all PxSELx = 01, which `impl_serial_pin!` takes as
    // Alternate1 (SLASEO7C Table 9-23, p. 65; SLASEO7C Table 9-24, p. 66; SLASEO7C Table 9-27, p. 69).
    // "Only one selected port is valid at any time" (SLASEO7C Table 9-11 notes 1 and 2, p. 54). UCLK is
    // the external clock that UCSSELx = 00b selects (SLAU445I Table 22-8, p. 593); the UCAxCLK pins are
    // listed for SPI only (SLASEO7C Table 9-11, p. 54).

    /// UCLK pin for E_USCI_A0 (default mapping)
    ///
    /// P1.6 in its UCA0CLK function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0ClockPinDefault;
    // Default pin mapping for the eUSCI_A0 clock signal.
    // Active when the USCIA0RMP remap bit in SYSCFG3 is cleared (SLAU445I Table 1-32, p. 83).
    impl_serial_pin!(UsciA0ClockPinDefault, P1, Pin6);

    /// UCLK pin for E_USCI_A0 (remapped mapping)
    ///
    /// P5.0 in its UCA0CLK function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0ClockPinRemapped;
    // Alternate pin mapping selected when the USCIA0RMP remap bit is set.
    // Only one mapping (default or remapped) is active at a time (SLASEO7C Table 9-11 note 2, p. 54).
    impl_serial_pin!(UsciA0ClockPinRemapped, P5, Pin0);

    /// Tx pin for E_USCI_A0 (default mapping)
    ///
    /// P1.4 in its UCA0TXD/UCA0SIMO function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0TxPinDefault;
    // Default transmit pin mapping.
    // Used when the USCIA0RMP remap bit in SYSCFG3 is cleared (SLAU445I Table 1-32, p. 83).
    impl_serial_pin!(UsciA0TxPinDefault, P1, Pin4);

    /// Tx pin for E_USCI_A0 (remapped mapping)
    ///
    /// P5.2 in its UCA0TXD/UCA0SIMO function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0TxPinRemapped;
    // Alternate transmit pin mapping selected via the USCIA0RMP remap bit.
    // Only one mapping (default or remapped) is active at a time (SLASEO7C Table 9-11 note 2, p. 54).
    impl_serial_pin!(UsciA0TxPinRemapped, P5, Pin2);

    /// Rx pin for E_USCI_A0 (default mapping)
    ///
    /// P1.5 in its UCA0RXD/UCA0SOMI function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0RxPinDefault;
    // Default receive pin mapping.
    // Active when the USCIA0RMP remap bit in SYSCFG3 is cleared (SLAU445I Table 1-32, p. 83).
    impl_serial_pin!(UsciA0RxPinDefault, P1, Pin5);

    /// Rx pin for E_USCI_A0 (remapped mapping)
    ///
    /// P5.1 in its UCA0RXD/UCA0SOMI function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0RxPinRemapped;
    // Alternate receive pin mapping selected when the USCIA0RMP remap bit is set.
    // Only one mapping (default or remapped) is active at a time (SLASEO7C Table 9-11 note 2, p. 54).
    impl_serial_pin!(UsciA0RxPinRemapped, P5, Pin1);

    /// UCLK pin for E_USCI_A1
    ///
    /// P2.4 in its UCA1CLK function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66).
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P2, Pin4);

    /// Tx pin for E_USCI_A1
    ///
    /// P2.6 in its UCA1TXD/UCA1SIMO function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66).
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P2, Pin6);

    /// Rx pin for E_USCI_A1
    ///
    /// P2.5 in its UCA1RXD/UCA1SOMI function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66).
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P2, Pin5);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, spi::*};

    // eUSCI_A and eUSCI_B both support SPI (SLASEO7C 9.10.7, p. 54); registers:
    // SLASEO7C Tables 9-50 to 9-53, p. 79 to p. 81.
    // eUSCI_A0 registers: SLASEO7C Table 9-50, p. 79 to p. 80; in SPI mode: SLAU445I Table 23-2, p. 612
    eusci_spi_impl!(
        EUsciA0,
        uca0ctlw0_spi,
        uca0brw,
        uca0statw_spi,
        uca0rxbuf,
        uca0txbuf,
        uca0ie_spi,
        uca0ifg_spi,
        uca0iv,
        crate::pac::e_usci_a0::uca0statw_spi::R
    );
    // eUSCI_A1 registers: SLASEO7C Table 9-51, p. 80; in SPI mode: SLAU445I Table 23-2, p. 612
    eusci_spi_impl!(
        EUsciA1,
        uca1ctlw0_spi,
        uca1brw,
        uca1statw_spi,
        uca1rxbuf,
        uca1txbuf,
        uca1ie_spi,
        uca1ifg_spi,
        uca1iv,
        crate::pac::e_usci_a1::uca1statw_spi::R
    );
    // eUSCI_B0 registers: SLASEO7C Table 9-52, p. 80; in SPI mode: SLAU445I Table 23-11, p. 619
    eusci_spi_impl!(
        EUsciB0,
        ucb0ctlw0_spi,
        ucb0brw,
        ucb0statw_spi,
        ucb0rxbuf,
        ucb0txbuf,
        ucb0ie_spi,
        ucb0ifg_spi,
        ucb0iv,
        crate::pac::e_usci_b0::ucb0statw_spi::R
    );
    // eUSCI_B1 registers: SLASEO7C Table 9-53, p. 80 to p. 81; in SPI mode: SLAU445I Table 23-11, p. 619
    eusci_spi_impl!(
        EUsciB1,
        ucb1ctlw0_spi,
        ucb1brw,
        ucb1statw_spi,
        ucb1rxbuf,
        ucb1txbuf,
        ucb1ie_spi,
        ucb1ifg_spi,
        ucb1iv,
        crate::pac::e_usci_b1::ucb1statw_spi::R
    );

    impl SpiUsci<DefaultMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinDefault;
        type MOSI = UsciA0MOSIPinDefault;
        type SCLK = UsciA0SCLKPinDefault;
        type STE = UsciA0STEPinDefault;

        fn configure_pin_mapping() {
            // USCIA0RMP, SYSCFG3 bit 0, = 0: default pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.uscia0rmp().clear_bit()) };
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinRemapped;
        type MOSI = UsciA0MOSIPinRemapped;
        type SCLK = UsciA0SCLKPinRemapped;
        type STE = UsciA0STEPinRemapped;

        fn configure_pin_mapping() {
            // USCIA0RMP, SYSCFG3 bit 0, = 1: remapped pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.uscia0rmp().set_bit()) };
        }
    }

    // eUSCI_A1 has one set of pins and no remap bit (SLASEO7C Table 9-11, p. 54)
    impl SpiUsci for EUsciA1 {
        type MISO = UsciA1MISOPin;
        type MOSI = UsciA1MOSIPin;
        type SCLK = UsciA1SCLKPin;
        type STE = UsciA1STEPin;
    }

    impl SpiUsci<DefaultMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinDefault;
        type MOSI = UsciB0MOSIPinDefault;
        type SCLK = UsciB0SCLKPinDefault;
        type STE = UsciB0STEPinDefault;

        fn configure_pin_mapping() {
            // USCIB0RMP, SYSCFG2 bit 11, = 0: default pins (SLAU445I Table 1-31, p. 82)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().clear_bits(|w| w.uscib0rmp().clear_bit()) };
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinRemapped;
        type MOSI = UsciB0MOSIPinRemapped;
        type SCLK = UsciB0SCLKPinRemapped;
        type STE = UsciB0STEPinRemapped;

        fn configure_pin_mapping() {
            // USCIB0RMP, SYSCFG2 bit 11, = 1: remapped pins (SLAU445I Table 1-31, p. 82)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().set_bits(|w| w.uscib0rmp().set_bit()) };
        }
    }

    impl SpiUsci<DefaultMapping> for EUsciB1 {
        type MISO = UsciB1MISOPinDefault;
        type MOSI = UsciB1MOSIPinDefault;
        type SCLK = UsciB1SCLKPinDefault;
        type STE = UsciB1STEPinDefault;

        fn configure_pin_mapping() {
            // USCIB1RMP, SYSCFG3 bit 4, = 0: default pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.uscib1rmp().clear_bit()) };
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciB1 {
        type MISO = UsciB1MISOPinRemapped;
        type MOSI = UsciB1MOSIPinRemapped;
        type SCLK = UsciB1SCLKPinRemapped;
        type STE = UsciB1STEPinRemapped;

        fn configure_pin_mapping() {
            // USCIB1RMP, SYSCFG3 bit 4, = 1: remapped pins (SLAU445I Table 1-32, p. 83)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.uscib1rmp().set_bit()) };
        }
    }

    // SPI pins (SLASEO7C Table 9-11, p. 54): MISO is SOMI, MOSI is SIMO. All are PxSELx = 01, which
    // `impl_spi_pin!` takes as Alternate1 (SLASEO7C Tables 9-23 to 9-27, p. 65 to p. 69). The default
    // pins need the remap bit (USCIA0RMP, USCIB0RMP, USCIB1RMP) at 0, the remapped ones at 1.

    /// SPI MISO pin for eUSCI A0 (P1.5) (default mapping)
    ///
    /// P1.5 in its UCA0RXD/UCA0SOMI function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0MISOPinDefault;
    impl_spi_pin!(UsciA0MISOPinDefault, P1, Pin5);

    /// SPI MISO pin for eUSCI A0 (P5.1) (remapped mapping)
    ///
    /// P5.1 in its UCA0RXD/UCA0SOMI function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0MISOPinRemapped;
    impl_spi_pin!(UsciA0MISOPinRemapped, P5, Pin1);

    /// SPI MOSI pin for eUSCI A0 (P1.4) (default mapping)
    ///
    /// P1.4 in its UCA0TXD/UCA0SIMO function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0MOSIPinDefault;
    impl_spi_pin!(UsciA0MOSIPinDefault, P1, Pin4);

    /// SPI MOSI pin for eUSCI A0 (P5.2) (remapped mapping)
    ///
    /// P5.2 in its UCA0TXD/UCA0SIMO function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0MOSIPinRemapped;
    impl_spi_pin!(UsciA0MOSIPinRemapped, P5, Pin2);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (default mapping)
    ///
    /// P1.6 in its UCA0CLK function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0SCLKPinDefault;
    impl_spi_pin!(UsciA0SCLKPinDefault, P1, Pin6);

    /// SPI SCLK pin for eUSCI A0 (P5.0) (remapped mapping)
    ///
    /// P5.0 in its UCA0CLK function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0SCLKPinRemapped;
    impl_spi_pin!(UsciA0SCLKPinRemapped, P5, Pin0);

    /// SPI STE pin for eUSCI A0 (P1.7) (default mapping)
    ///
    /// P1.7 in its UCA0STE function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIA0RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0STEPinDefault;
    impl_spi_pin!(UsciA0STEPinDefault, P1, Pin7);

    /// SPI STE pin for eUSCI A0 (P4.7) (remapped mapping)
    ///
    /// P4.7 in its UCA0STE function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIA0RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciA0STEPinRemapped;
    impl_spi_pin!(UsciA0STEPinRemapped, P4, Pin7);

    /// SPI MISO pin for eUSCI A1 (P2.5)
    ///
    /// P2.5 in its UCA1RXD/UCA1SOMI function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66).
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P2, Pin5);

    /// SPI MOSI pin for eUSCI A1 (P2.6)
    ///
    /// P2.6 in its UCA1TXD/UCA1SIMO function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66).
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P2, Pin6);

    /// SPI SCLK pin for eUSCI A1 (P2.4)
    ///
    /// P2.4 in its UCA1CLK function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66).
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P2, Pin4);
    /// SPI STE pin for eUSCI A1 (P3.1)
    ///
    /// P3.1 in its UCA1STE function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67).
    pub struct UsciA1STEPin;
    impl_spi_pin!(UsciA1STEPin, P3, Pin1);

    /// SPI MISO pin for eUSCI B0 (P1.3) (default mapping)
    ///
    /// P1.3 in its UCB0SOMI/UCB0SCL function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0MISOPinDefault;
    impl_spi_pin!(UsciB0MISOPinDefault, P1, Pin3);

    /// SPI MISO pin for eUSCI B0 (P4.5) (remapped mapping)
    ///
    /// P4.5 in its UCB0SOMI/UCB0SCL function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0MISOPinRemapped;
    impl_spi_pin!(UsciB0MISOPinRemapped, P4, Pin5);

    /// SPI MOSI pin for eUSCI B0 (P1.2) (default mapping)
    ///
    /// P1.2 in its UCB0SIMO/UCB0SDA function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0MOSIPinDefault;
    impl_spi_pin!(UsciB0MOSIPinDefault, P1, Pin2);

    /// SPI MOSI pin for eUSCI B0 (P4.6) (remapped mapping)
    ///
    /// P4.6 in its UCB0SIMO/UCB0SDA function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0MOSIPinRemapped;
    impl_spi_pin!(UsciB0MOSIPinRemapped, P4, Pin6);

    /// SPI SCLK pin for eUSCI B0 (P1.1) (default mapping)
    ///
    /// P1.1 in its UCB0CLK function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0SCLKPinDefault;
    impl_spi_pin!(UsciB0SCLKPinDefault, P1, Pin1);

    /// SPI SCLK pin for eUSCI B0 (P5.5) (remapped mapping)
    ///
    /// P5.5 in its UCB0CLK function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0SCLKPinRemapped;
    impl_spi_pin!(UsciB0SCLKPinRemapped, P5, Pin5);

    /// SPI STE pin for eUSCI B0 (P1.0) (default mapping)
    ///
    /// P1.0 in its UCB0STE function, P1SELx = 01 (SLASEO7C Table 9-23, p. 65),
    /// used while USCIB0RMP = 0 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0STEPinDefault;
    impl_spi_pin!(UsciB0STEPinDefault, P1, Pin0);

    /// SPI STE pin for eUSCI B0 (P5.6) (remapped mapping)
    ///
    /// P5.6 in its UCB0STE function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIB0RMP = 1 (SLAU445I Table 1-31, p. 82).
    pub struct UsciB0STEPinRemapped;
    impl_spi_pin!(UsciB0STEPinRemapped, P5, Pin6);

    /// SPI MISO pin for eUSCI B1 (P3.6) (default mapping)
    ///
    /// P3.6 in its UCB1SOMI/UCB1SCL function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1MISOPinDefault;
    impl_spi_pin!(UsciB1MISOPinDefault, P3, Pin6);

    /// SPI MISO pin for eUSCI B1 (P4.3) (remapped mapping)
    ///
    /// P4.3 in its UCB1SOMI/UCB1SCL function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1MISOPinRemapped;
    impl_spi_pin!(UsciB1MISOPinRemapped, P4, Pin3);

    /// SPI MOSI pin for eUSCI B1 (P3.2) (default mapping)
    ///
    /// P3.2 in its UCB1SIMO/UCB1SDA function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1MOSIPinDefault;
    impl_spi_pin!(UsciB1MOSIPinDefault, P3, Pin2);

    /// SPI MOSI pin for eUSCI B1 (P4.4) (remapped mapping)
    ///
    /// P4.4 in its UCB1SIMO/UCB1SDA function, P4SELx = 01 (SLASEO7C Table 9-26, p. 68),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1MOSIPinRemapped;
    impl_spi_pin!(UsciB1MOSIPinRemapped, P4, Pin4);

    /// SPI SCLK pin for eUSCI B1 (P3.5) (default mapping)
    ///
    /// P3.5 in its UCB1CLK function, P3SELx = 01 (SLASEO7C Table 9-25, p. 67),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1SCLKPinDefault;
    impl_spi_pin!(UsciB1SCLKPinDefault, P3, Pin5);

    /// SPI SCLK pin for eUSCI B1 (P5.3) (remapped mapping)
    ///
    /// P5.3 in its UCB1CLK function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1SCLKPinRemapped;
    impl_spi_pin!(UsciB1SCLKPinRemapped, P5, Pin3);

    /// SPI STE pin for eUSCI B1 (P2.7) (default mapping)
    ///
    /// P2.7 in its UCB1STE function, P2SELx = 01 (SLASEO7C Table 9-24, p. 66),
    /// used while USCIB1RMP = 0 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1STEPinDefault;
    impl_spi_pin!(UsciB1STEPinDefault, P2, Pin7);

    /// SPI STE pin for eUSCI B1 (P5.4) (remapped mapping)
    ///
    /// P5.4 in its UCB1STE function, P5SELx = 01 (SLASEO7C Table 9-27, p. 69),
    /// used while USCIB1RMP = 1 (SLAU445I Table 1-32, p. 83).
    pub struct UsciB1STEPinRemapped;
    impl_spi_pin!(UsciB1STEPinRemapped, P5, Pin4);
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_a::*, timer_b::*, Steal},
        pac::*,
        pin_mapping::*,
        timer::*,
    };

    // TA0 to TA3 have three capture/compare registers each (SLASEO7C 9.10.8, p. 55), TB0 seven
    // (SLASEO7C Table 9-15, p. 59); registers: SLASEO7C Tables 9-44 to 9-48, p. 77 to p. 79.
    // TA0 registers: SLASEO7C Table 9-44, p. 77; Timer_A registers: SLAU445I Table 13-3, p. 383
    timer_a_impl!(
        Ta0,
        ta0,
        ta0ctl,
        ta0ex0,
        ta0iv,
        ta0r,
        taclr,
        taifg,
        taidex,
        taie,
        tassel,
        [CCR0, ta0cctl0, ta0ccr0],
        [CCR1, ta0cctl1, ta0ccr1],
        [CCR2, ta0cctl2, ta0ccr2]
    );

    // TA1 registers: SLASEO7C Table 9-45, p. 77 to p. 78; Timer_A registers: SLAU445I Table 13-3, p. 383
    timer_a_impl!(
        Ta1,
        ta1,
        ta1ctl,
        ta1ex0,
        ta1iv,
        ta1r,
        taclr,
        taifg,
        taidex,
        taie,
        tassel,
        [CCR0, ta1cctl0, ta1ccr0],
        [CCR1, ta1cctl1, ta1ccr1],
        [CCR2, ta1cctl2, ta1ccr2]
    );

    // TA2 registers: SLASEO7C Table 9-46, p. 78; Timer_A registers: SLAU445I Table 13-3, p. 383
    timer_a_impl!(
        Ta2,
        ta2,
        ta2ctl,
        ta2ex0,
        ta2iv,
        ta2r,
        taclr,
        taifg,
        taidex,
        taie,
        tassel,
        [CCR0, ta2cctl0, ta2ccr0],
        [CCR1, ta2cctl1, ta2ccr1],
        [CCR2, ta2cctl2, ta2ccr2]
    );

    // TA3 registers: SLASEO7C Table 9-47, p. 78; Timer_A registers: SLAU445I Table 13-3, p. 383
    timer_a_impl!(
        Ta3,
        ta3,
        ta3ctl,
        ta3ex0,
        ta3iv,
        ta3r,
        taclr,
        taifg,
        taidex,
        taie,
        tassel,
        [CCR0, ta3cctl0, ta3ccr0],
        [CCR1, ta3cctl1, ta3ccr1],
        [CCR2, ta3cctl2, ta3ccr2]
    );

    // TB0 registers, CCR0 to CCR6: SLASEO7C Table 9-48, p. 78 to p. 79; Timer_B registers:
    // SLAU445I Table 14-5, p. 408
    timer_b_impl!(
        Tb0,
        tb0,
        tb0ctl,
        tb0ex0,
        tb0iv,
        tb0r,
        tbclr,
        tbifg,
        tbidex,
        tbie,
        tbssel,
        [CCR0, tb0cctl0, tb0ccr0],
        [CCR1, tb0cctl1, tb0ccr1],
        [CCR2, tb0cctl2, tb0ccr2],
        [CCR3, tb0cctl3, tb0ccr3],
        [CCR4, tb0cctl4, tb0ccr4],
        [CCR5, tb0cctl5, tb0ccr5],
        [CCR6, tb0cctl6, tb0ccr6]
    );

    // The external clock input (TACLK/TBCLK) of each timer, from the timer signal connections
    // (SLASEO7C Tables 9-12 to 9-16, p. 55 to p. 60) and the pin function tables
    // (SLASEO7C Tables 9-23 to 9-28, p. 65 to p. 70). These functions need PxDIR = 0, so the pins are
    // inputs.
    impl TimerPeriph for Ta0 {
        // P1.0 TA0CLK, P1SELx = 10 (SLASEO7C Table 9-23, p. 65; SLASEO7C Table 9-12, p. 55)
        type Tbxclk = Pin<P1, Pin0, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta0 {} // CCR0 to CCR2 (SLASEO7C 9.10.8, p. 55)

    impl TimerPeriph for Ta1 {
        // P1.6 TA1CLK, P1SELx = 10 (SLASEO7C Table 9-23, p. 65; SLASEO7C Table 9-13, p. 56)
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta1 {} // CCR0 to CCR2 (SLASEO7C 9.10.8, p. 55)

    impl TimerPeriph<DefaultMapping> for Ta2 {
        // P3.4 TA2CLK, P3SELx = 01, TA2RMP = 0 (SLASEO7C Table 9-25, p. 67; SLASEO7C Table 9-16, p. 60)
        type Tbxclk = Pin<P3, Pin4, Alternate1<Input<Floating>>>;

        fn configure_pin_mapping() {
            // TA2RMP, SYSCFG3 bit 2, = 0: default pins (SLAU445I Table 1-32, p. 83;
            // SLASEO7C Table 9-16, p. 60)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.ta2rmp().clear_bit()) };
        }
    }
    impl TimerPeriph<RemappedMapping> for Ta2 {
        // P5.5 TA2CLK, P5SELx = 10, TA2RMP = 1 (SLASEO7C Table 9-27, p. 69; SLASEO7C Table 9-16, p. 60)
        type Tbxclk = Pin<P5, Pin5, Alternate2<Input<Floating>>>;

        fn configure_pin_mapping() {
            // TA2RMP, SYSCFG3 bit 2, = 1: remapped pins (SLAU445I Table 1-32, p. 83;
            // SLASEO7C Table 9-16, p. 60)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.ta2rmp().set_bit()) };
        }
    }
    impl CapCmpTimer3<DefaultMapping> for Ta2 {} // CCR0 to CCR2 (SLASEO7C 9.10.8, p. 55)
    impl CapCmpTimer3<RemappedMapping> for Ta2 {}

    impl TimerPeriph<DefaultMapping> for Ta3 {
        // P4.2 TA3CLK, P4SELx = 01, TA3RMP = 0 (SLASEO7C Table 9-26, p. 68; SLASEO7C Table 9-16, p. 60)
        type Tbxclk = Pin<P4, Pin2, Alternate1<Input<Floating>>>;

        fn configure_pin_mapping() {
            // TA3RMP, SYSCFG3 bit 3, = 0: default pins (SLAU445I Table 1-32, p. 83;
            // SLASEO7C Table 9-16, p. 60)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.ta3rmp().clear_bit()) };
        }
    }
    impl TimerPeriph<RemappedMapping> for Ta3 {
        // P5.4 TA3CLK, P5SELx = 10, TA3RMP = 1 (SLASEO7C Table 9-27, p. 69; SLASEO7C Table 9-16, p. 60)
        type Tbxclk = Pin<P5, Pin4, Alternate2<Input<Floating>>>;

        fn configure_pin_mapping() {
            // TA3RMP, SYSCFG3 bit 3, = 1: remapped pins (SLAU445I Table 1-32, p. 83;
            // SLASEO7C Table 9-16, p. 60)
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.ta3rmp().set_bit()) };
        }
    }
    impl CapCmpTimer3<DefaultMapping> for Ta3 {} // CCR0 to CCR2 (SLASEO7C 9.10.8, p. 55)
    impl CapCmpTimer3<RemappedMapping> for Ta3 {}

    impl TimerPeriph for Tb0 {
        // TB0CLK is only bonded out in the 48-pin package (SLASEO7C Table 7-2, p. 18: pin 40 of the PT
        // package, none on RHA and RHB). P6.1 TB0CLK, P6SELx = 01 (SLASEO7C Table 9-28, p. 70;
        // SLASEO7C Table 9-15, p. 59)
        type Tbxclk = Pin<P6, Pin1, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer7 for Tb0 {} // CCR0 to CCR6 (SLASEO7C Table 9-15, p. 59)

    // INCLK is the VLO on TA0 and TA2, and the CCR2 output of TA0 on TA1 and of TA2 on TA3. It
    // isn't connected on TB0. (SLASEO7C Tables 9-12 to 9-15, p. 55 to p. 59)
    impl VloclkTimer for Ta0 {}
    impl VloclkTimer for Ta2 {}
    impl CascadedTimer for Ta1 {
        type Source = Ta0;
    }
    impl CascadedTimer for Ta3 {
        type Source = Ta2;
    }

    // The TB0OUTH trigger is eCOMP0 or the TB0TRG pin, P3.5 (SLASEO7C Table 9-17, p. 61: TB0TRGSEL = 0
    // selects the eCOMP0 output, 1 selects P3.5). TB0TRGSEL is SYSCFG2 bit 15, "1b = External source
    // selected" (SLAU445I Table 1-31, p. 82). The multiplexer in SLASEO7C Figure 9-3, p. 60 shows the
    // inputs the other way round; the code follows SLASEO7C Table 9-17, p. 61.
    high_impedance_timer_impl!(Tb0, tb0trgsel);
    // P3.5 TB0TRG, P3SELx = 10, P3DIR.5 = 0 (SLASEO7C Table 9-25, p. 67)
    impl<PULL> HighImpedancePin<Tb0> for Pin<P3, Pin5, Alternate2<Input<PULL>>> {}
}

pub mod clock {
    use crate::gpio::*;
    use crate::_pac::cs::csctl3::Fllrefdiv;

    // The XT1 pins are defined once, here. Everything else, from the `Xt1Config` constructors to
    // keeping the pins selected through LPM3.5, derives the port, pin and PxSEL bits from these
    // types. Both pins are selected with P2SELx = 01 (SLASEO7C Table 9-24, p. 66: P2.1 XIN, P2.0 XOUT).
    /// XT1 input pin (XIN), in its XT1 function
    pub type Xt1Xin<DIR> = Pin<P2, Pin1, Alternate1<DIR>>;
    /// XT1 output pin (XOUT), in its XT1 function
    pub type Xt1Xout<DIR> = Pin<P2, Pin0, Alternate1<DIR>>;

    /// FLLREFDIV = 000b, the reference divided by 1 (SLAU445I Table 3-7, p. 116)
    pub const FLLREFDIV_1: Fllrefdiv = Fllrefdiv::_1;
}
