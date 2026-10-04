pub use msp430fr2355 as pac;

/// PAC with standardised peripheral names. For the fr2x5x this is just the PAC.
pub use msp430fr2355 as _pac;
/*         GPIO          */
pub mod gpio {
    // Make PAC GPIO avilable as a re-export
    pub use crate::pac::{P1, P2, P3, P4, P5, P6};

    use crate::gpio::*;
    use crate::hw_traits::gpio::gpio_impl;

    // Define alternate pin transitions
    //
    // Alternate1, 2 and 3 are PxSELx (PxSEL1/PxSEL0) = 01, 10 and 11, the primary, secondary and
    // tertiary module functions (SLAU445I 8.2.5, Table 8-3, p. 314). Each impl is a row of the port's
    // pin function table (SLASEC4D Tables 6-63 to 6-68, p. 96 to p. 106). PxSEL doesn't set the direction
    // (SLAU445I 8.2.5, p. 314): `in` below is PxDIR = 0 and `out` is PxDIR = 1. A timer pin's direction
    // picks the capture input (CCIxA, in) or the compare output (out).
    //
    // Some functions work in one direction only, and most of these list VSS (ground) for the other one:
    // the module has no signal for that direction. With PxSELx = 01 or 10 the output driver drives the
    // module's signal (SLASEC4D Figures 6-4 to 6-9, p. 95 to p. 105), so an input-only function such as
    // TB3CLK drives the pin low with PxDIR = 1. An output-only function such as MCLK never reaches the pin
    // with PxDIR = 0, because the driver is off. Neither does anything useful, and nothing is missing:
    // these functions are only given to `Input` or `Output` pins.

    // P1 alternate 1, P1SELx = 01 (SLASEC4D Table 6-63, p. 96): P1.0 UCB0STE, P1.1 UCB0CLK,
    // P1.2 UCB0SIMO/UCB0SDA, P1.3 UCB0SOMI/UCB0SCL, P1.4 UCA0STE, P1.5 UCA0CLK, P1.6 UCA0RXD/UCA0SOMI,
    // P1.7 UCA0TXD/UCA0SIMO
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2, P1SELx = 10 (SLASEC4D Table 6-63, p. 96). P1.3 to P1.5 have no 10 function.
    impl       ToAlternate2 for Pin<P1, Pin0, Output> {} // 10: SMCLK out / VSS in
    impl       ToAlternate2 for Pin<P1, Pin1, Output> {} // 10: ACLK out / VSS in
    impl<PULL> ToAlternate2 for Pin<P1, Pin2, Input<PULL>> {} // 10: TB0TRG in (no out row)
    impl<DIR>  ToAlternate2 for Pin<P1, Pin6, DIR> {} // 10: TB0.CCI1A in / TB0.1 out
    impl<DIR>  ToAlternate2 for Pin<P1, Pin7, DIR> {} // 10: TB0.CCI2A in / TB0.2 out
    // P1 alternate 3, P1SELx = 11 (SLASEC4D Table 6-63, p. 96): the analog functions, A0 to A7 on
    // P1.0 to P1.7, with COMP0.0 and Veref+ on P1.0, OA0O and COMP0.1 on P1.1, OA0- and Veref- on P1.2,
    // OA0+ on P1.3, OA1O on P1.5, OA1- on P1.6, OA1+ and VREF+ on P1.7 (the OAx on the MSP430FR235x only)
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1, P2SELx = 01 (SLASEC4D Table 6-64, p. 98). P2.4 and P2.5 have no 01 function. The
    // name "P2.3/UCB0CLK/TB1TRG" lists UCB0CLK, but the table has no UCB0CLK row for P2.3, and UCB0CLK
    // is only on P1.1 (SLASEC4D Table 4-2, p. 24).
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {} // 01: TB1.CCI1A in / TB1.1 out
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {} // 01: TB1.CCI2A in / TB1.2 out
    impl<PULL> ToAlternate1 for Pin<P2, Pin2, Input<PULL>> {} // 01: TB1CLK in (no out row)
    impl<PULL> ToAlternate1 for Pin<P2, Pin3, Input<PULL>> {} // 01: TB1TRG in / VSS out
    impl       ToAlternate1 for Pin<P2, Pin6, Output> {} // 01: MCLK out / VSS in
    impl<PULL> ToAlternate1 for Pin<P2, Pin7, Input<PULL>> {} // 01: TB0CLK in / VSS out
    // P2 alternate 2, P2SELx = 10 (SLASEC4D Table 6-64, p. 98)
    impl ToAlternate2 for Pin<P2, Pin0, Output> {} // 10: COMP0.O out (no in row)
    impl ToAlternate2 for Pin<P2, Pin1, Output> {} // 10: COMP1.O out (no in row)
    impl<DIR> ToAlternate2 for Pin<P2, Pin6, DIR> {} // 10: XOUT, either direction
    impl<DIR> ToAlternate2 for Pin<P2, Pin7, DIR> {} // 10: XIN, either direction
    // P2 alternate 3, P2SELx = 11 (SLASEC4D Table 6-64, p. 98)
    impl<DIR> ToAlternate3 for Pin<P2, Pin4, DIR> {} // 11: COMP1.1
    impl<DIR> ToAlternate3 for Pin<P2, Pin5, DIR> {} // 11: COMP1.0

    // P3 alternate 1, P3SELx = 01 (SLASEC4D Table 6-65, p. 100)
    impl      ToAlternate1 for Pin<P3, Pin0, Output> {} // 01: MCLK out / VSS in
    impl      ToAlternate1 for Pin<P3, Pin4, Output> {} // 01: SMCLK out / VSS in
    // P3 alternate 3 is the SAC2 and SAC3 op-amp pins, on the MSP430FR235x only (P3SELx = 11,
    // SLASEC4D Table 6-65, p. 100, note 2: "MSP430FR235x devices only")
    #[cfg(feature = "sac")]
    impl<DIR> ToAlternate3 for Pin<P3, Pin1, DIR> {} // 11: OA2O
    #[cfg(feature = "sac")]
    impl<DIR> ToAlternate3 for Pin<P3, Pin2, DIR> {} // 11: OA2-
    #[cfg(feature = "sac")]
    impl<DIR> ToAlternate3 for Pin<P3, Pin3, DIR> {} // 11: OA2+
    #[cfg(feature = "sac")]
    impl<DIR> ToAlternate3 for Pin<P3, Pin5, DIR> {} // 11: OA3O
    #[cfg(feature = "sac")]
    impl<DIR> ToAlternate3 for Pin<P3, Pin6, DIR> {} // 11: OA3-
    #[cfg(feature = "sac")]
    impl<DIR> ToAlternate3 for Pin<P3, Pin7, DIR> {} // 11: OA3+

    // P4 alternate 1, P4SELx = 01 (SLASEC4D Table 6-66, p. 102): P4.0 UCA1STE, P4.1 UCA1CLK,
    // P4.2 UCA1RXD/UCA1SOMI, P4.3 UCA1TXD/UCA1SIMO, P4.4 UCB1STE, P4.5 UCB1CLK, P4.6 UCB1SIMO/UCB1SDA,
    // P4.7 UCB1SOMI/UCB1SCL
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P4, PIN, DIR> {}
    // P4 alternate 2, P4SELx = 10 (SLASEC4D Table 6-66, p. 102). On P4.0, ISORXD feeds UCA1RXD and
    // TB3.CCI2B, and ISOTXD is "the logical AND product of UCA1TXD and TB3.2B" (SLASEC4D Table 4-2,
    // p. 24).
    impl<DIR> ToAlternate2 for Pin<P4, Pin0, DIR> {} // 10: ISORXD in / ISOTXD out
    impl<DIR> ToAlternate2 for Pin<P4, Pin2, DIR> {} // 10: UCA1RXD, inverted
    impl<DIR> ToAlternate2 for Pin<P4, Pin3, DIR> {} // 10: UCA1TXD, inverted

    // P5 alternate 1, P5SELx = 01 (SLASEC4D Table 6-67, p. 104)
    impl<DIR> ToAlternate1 for Pin<P5, Pin0, DIR> {} // 01: TB2.CCI1A in / TB2.1 out
    impl<DIR> ToAlternate1 for Pin<P5, Pin1, DIR> {} // 01: TB2.CCI2A in / TB2.2 out
    impl<PULL> ToAlternate1 for Pin<P5, Pin2, Input<PULL>> {} // 01: TB2CLK in / VSS out
    impl<PULL> ToAlternate1 for Pin<P5, Pin3, Input<PULL>> {} // 01: TB2TRG in / VSS out
    // P5 alternate 2, P5SELx = 10 (SLASEC4D Table 6-67, p. 104)
    impl<DIR> ToAlternate2 for Pin<P5, Pin0, DIR> {} // 10: MFM.RX
    impl<DIR> ToAlternate2 for Pin<P5, Pin1, DIR> {} // 10: MFM.TX
    // P5 alternate 3, P5SELx = 11 (SLASEC4D Table 6-67, p. 104)
    impl<DIR> ToAlternate3 for Pin<P5, Pin0, DIR> {} // 11: A8
    impl<DIR> ToAlternate3 for Pin<P5, Pin1, DIR> {} // 11: A9
    impl<DIR> ToAlternate3 for Pin<P5, Pin2, DIR> {} // 11: A10
    impl<DIR> ToAlternate3 for Pin<P5, Pin3, DIR> {} // 11: A11

    // P6 alternate 1, P6SELx = 01 (SLASEC4D Table 6-68, p. 106)
    impl<DIR>  ToAlternate1 for Pin<P6, Pin0, DIR> {} // 01: TB3.CCI1A in / TB3.1 out
    impl<DIR>  ToAlternate1 for Pin<P6, Pin1, DIR> {} // 01: TB3.CCI2A in / TB3.2 out
    impl<DIR>  ToAlternate1 for Pin<P6, Pin2, DIR> {} // 01: TB3.CCI3A in / TB3.3 out
    impl<DIR>  ToAlternate1 for Pin<P6, Pin3, DIR> {} // 01: TB3.CCI4A in / TB3.4 out
    impl<DIR>  ToAlternate1 for Pin<P6, Pin4, DIR> {} // 01: TB3.CCI5A in / TB3.5 out
    impl<DIR>  ToAlternate1 for Pin<P6, Pin5, DIR> {} // 01: TB3.CCI6A in / TB3.6 out
    impl<PULL> ToAlternate1 for Pin<P6, Pin6, Input<PULL>> {} // 01: TB3CLK in / VSS out

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable. P1 to P4 have
    // PxSELC and the interrupt registers, P5 and P6 only PxSELC (SLASEC4D Tables 6-41 to 6-43, p. 86 to
    // p. 87): "Interrupt conditions are possible in P1, P2, P3, and P4" (SLASEC4D 6.10.3, p. 69; port
    // vectors in SLASEC4D Table 6-2, p. 64).
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
    gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
    gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1);
    gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1);

    // Pins per port (SLASEC4D 6.10.3, p. 69: "P1, P2, P3, and P4 are full 8-bit ports; P5 and P6
    // feature up to 5-bit and 7-bit ports"). The pins a port lacks are always the top ones: P5 has P5.0
    // to P5.4 and P6 has P6.0 to P6.6 (SLASEC4D Table 6-67, p. 104; SLASEC4D Table 6-68, p. 106).
    impl_port_pins!(P1, 8);
    impl_port_pins!(P2, 8);
    impl_port_pins!(P3, 8);
    impl_port_pins!(P4, 8);
    impl_port_pins!(P5, 5);
    impl_port_pins!(P6, 7);
}

/* ADC */
mod adc {
    use crate::{adc::*, gpio::*, pmm::VrefOutputPin};

    // The timer whose CCR1 output triggers conversions (SLASEC4D Table 6-22, p. 77: ADCSHSx = 10b is
    // TB1.1B; SLASEC4D Table 6-17, p. 74: the TB1 CCR1 output goes "To ADC trigger")
    impl AdcTriggerTimer for crate::pac::Tb1 {}

    // External reference inputs and the VREF+ output, each in its P1SELx = 11 function (SLASEC4D
    // Table 6-63, p. 96; SLASEC4D Table 4-2, p. 22 to p. 23)
    impl<DIR> VeRefPlusPin for Pin<P1, Pin0, Alternate3<DIR>> {} // 11: Veref+, "ADC positive reference"
    impl<DIR> VeRefMinusPin for Pin<P1, Pin2, Alternate3<DIR>> {} // 11: Veref-, "ADC negative reference"
    // 11: VREF+, the 1.2 V reference output: "When A7 is used, the PMM 1.2-V reference voltage can be
    // output to this pin" (SLASEC4D Table 6-21, note 1, p. 77). The 1.5 V, 2.0 V and 2.5 V references
    // "cannot be output to the VREF+ pin" (SLASEC4D Table 5-10, p. 41).
    impl<DIR> VrefOutputPin for Pin<P1, Pin7, Alternate3<DIR>> {}

    // The 12 external inputs A0 to A11 with their ADCINCHx value (SLASEC4D 6.10.12 and Table 6-21,
    // p. 77), each in its PxSELx = 11 function (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-67,
    // p. 104): "Set the PxSEL bit in the port register to disable the I/O functions" (SLAU445I 1.12.4.3,
    // p. 54).
    impl_adc_channel_pin!(P1, Pin0, Alternate3 => 0); // A0, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin1, Alternate3 => 1); // A1, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin2, Alternate3 => 2); // A2, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin3, Alternate3 => 3); // A3, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin4, Alternate3 => 4); // A4, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin5, Alternate3 => 5); // A5, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin6, Alternate3 => 6); // A6, P1SELx = 11
    impl_adc_channel_pin!(P1, Pin7, Alternate3 => 7); // A7, P1SELx = 11
    impl_adc_channel_pin!(P5, Pin0, Alternate3 => 8); // A8, P5SELx = 11
    impl_adc_channel_pin!(P5, Pin1, Alternate3 => 9); // A9, P5SELx = 11
    impl_adc_channel_pin!(P5, Pin2, Alternate3 => 10); // A10, P5SELx = 11
    impl_adc_channel_pin!(P5, Pin3, Alternate3 => 11); // A11, P5SELx = 11
}

/* Backup Memory */
/// Size of the Backup Memory segment on this device, in bytes (SLASEC4D 6.10.10, p. 76: "This device
/// provides up to 32 bytes that are retained during LPM3.5"; BAKMEM0 to BAKMEM15 in SLASEC4D
/// Table 6-54, p. 92)
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::{CapturePeriph, NoCapturePin}, gpio::*, pac::*};

    // Capture inputs CCInA (SLASEC4D Tables 6-16 to 6-19, p. 73 to p. 75), on the pins with PxDIR = 0
    // in the pin function tables. CCR0 has no pin: "The CCR0 registers on all timers are not
    // externally connected" (SLASEC4D 6.10.9, p. 73). Inside the device, CCI0A of TB0 is "From RTC
    // (internal)" and of TB1 "Timer3_B7 CCR0B output (internal)", selected with `()`; on TB2 and TB3 it is
    // "Not used", so it is `NoCapturePin`, which can't be selected (SLASEC4D Table 6-16, p. 73; SLASEC4D
    // Table 6-17, p. 74; SLASEC4D Table 6-18, p. 74; SLASEC4D Table 6-19, p. 75).

    // TB0: P1SELx = 10 (SLASEC4D Table 6-16, p. 73; SLASEC4D Table 6-63, p. 96)
    impl CapturePeriph for Tb0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin6, Alternate2<Input<Floating>>>; // TB0.CCI1A, P1SELx = 10
        type Gpio2 = Pin<P1, Pin7, Alternate2<Input<Floating>>>; // TB0.CCI2A, P1SELx = 10
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TB1: P2SELx = 01 (SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-64, p. 98)
    impl CapturePeriph for Tb1 {
        type Gpio0 = ();
        type Gpio1 = Pin<P2, Pin0, Alternate1<Input<Floating>>>; // TB1.CCI1A, P2SELx = 01
        type Gpio2 = Pin<P2, Pin1, Alternate1<Input<Floating>>>; // TB1.CCI2A, P2SELx = 01
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TB2: P5SELx = 01 (SLASEC4D Table 6-18, p. 74; SLASEC4D Table 6-67, p. 104)
    impl CapturePeriph for Tb2 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = Pin<P5, Pin0, Alternate1<Input<Floating>>>; // TB2.CCI1A, P5SELx = 01
        type Gpio2 = Pin<P5, Pin1, Alternate1<Input<Floating>>>; // TB2.CCI2A, P5SELx = 01
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TB3: P6SELx = 01 (SLASEC4D Table 6-19, p. 75; SLASEC4D Table 6-68, p. 106)
    impl CapturePeriph for Tb3 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = Pin<P6, Pin0, Alternate1<Input<Floating>>>; // TB3.CCI1A, P6SELx = 01
        type Gpio2 = Pin<P6, Pin1, Alternate1<Input<Floating>>>; // TB3.CCI2A, P6SELx = 01
        type Gpio3 = Pin<P6, Pin2, Alternate1<Input<Floating>>>; // TB3.CCI3A, P6SELx = 01
        type Gpio4 = Pin<P6, Pin3, Alternate1<Input<Floating>>>; // TB3.CCI4A, P6SELx = 01
        type Gpio5 = Pin<P6, Pin4, Alternate1<Input<Floating>>>; // TB3.CCI5A, P6SELx = 01
        type Gpio6 = Pin<P6, Pin5, Alternate1<Input<Floating>>>; // TB3.CCI6A, P6SELx = 01
    }
}

/* Clocks */
/// MODCLK frequency
pub const MODCLK_FREQ_HZ: u32 = 3_800_000;

/* eCOMP */
pub mod ecomp {
    use core::convert::Infallible;

    use crate::hw_traits::ecomp::*;
    use crate::pac::{EComp0, EComp1};
    use crate::{ecomp::*, gpio::*};
    #[cfg(feature = "sac")]
    use crate::{
        pac::{Sac0, Sac1, Sac2, Sac3},
        sac::Amplifier,
    };

    // eCOMP0 channels (SLASEC4D Table 6-23, p. 78): CPPSEL and CPNSEL 000b is COMP0.0 (P1.0), 001b
    // COMP0.1 (P1.1), 010b the low-power 1.2 V reference, 101b the SAC0 output OA0O (P1.1) on the
    // positive side and the SAC2 output OA2O (P3.1) on the negative side, and 110b the 6-bit DAC;
    // 011b and 100b are N/A. The input pins are in their PxSELx = 11 function (SLASEC4D Table 6-63,
    // p. 96; SLASEC4D Table 6-65, p. 100). The output COMP0.O is P2.0 with P2SELx = 10 and P2DIR = 1
    // (SLASEC4D Table 6-64, p. 98; SLASEC4D Table 6-25, p. 78).
    impl ECompInputs for EComp0 {
        type COMPx_0   = Pin<P1, Pin0, Alternate3<Input<Floating>>>; // COMP0.0, P1SELx = 11
        type COMPx_1   = Pin<P1, Pin1, Alternate3<Input<Floating>>>; // COMP0.1, P1SELx = 11
        type COMPx_2   = Infallible; // Not used: CPxSEL 011b is N/A
        type COMPx_3   = Infallible; // Not used: CPxSEL 100b is N/A
        type COMPx_Out = Pin<P2, Pin0, Alternate2<Output>>; // COMP0.O, P2SELx = 10, out
        #[cfg(feature = "sac")]
        type SACp = Amplifier<Sac0>; // CPPSEL 101b: OA0O
        #[cfg(feature = "sac")]
        type SACn = Amplifier<Sac2>; // CPNSEL 101b: OA2O

        type DeviceSpecific0    = (); // Internal 1.2V reference (CPxSEL 010b). No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        // CPPSEL 101b: OA0O, P1SELx = 11
        type DeviceSpecific3Pos = Pin<P1, Pin1, Alternate3<Input<Floating>>>;
        // CPNSEL 101b: OA2O, P3SELx = 11
        type DeviceSpecific3Neg = Pin<P3, Pin1, Alternate3<Input<Floating>>>;
    }
    // eCOMP1 channels (SLASEC4D Table 6-24, p. 78): CPPSEL and CPNSEL 000b is COMP1.0 (P2.5), 001b
    // COMP1.1 (P2.4), 010b the low-power 1.2 V reference, 101b the SAC1 output OA1O (P1.5) on the
    // positive side and the SAC3 output OA3O (P3.5) on the negative side, and 110b the 6-bit DAC;
    // 011b and 100b are N/A. The input pins are in their PxSELx = 11 function (SLASEC4D Table 6-63,
    // p. 96; SLASEC4D Table 6-64, p. 98; SLASEC4D Table 6-65, p. 100). The output COMP1.O is P2.1 with
    // P2SELx = 10 and P2DIR = 1 (SLASEC4D Table 6-64, p. 98; SLASEC4D Table 6-26, p. 79).
    impl ECompInputs for EComp1 {
        type COMPx_0   = Pin<P2, Pin5, Alternate3<Input<Floating>>>; // COMP1.0, P2SELx = 11
        type COMPx_1   = Pin<P2, Pin4, Alternate3<Input<Floating>>>; // COMP1.1, P2SELx = 11
        type COMPx_2   = Infallible; // Not used: CPxSEL 011b is N/A
        type COMPx_3   = Infallible; // Not used: CPxSEL 100b is N/A
        type COMPx_Out = Pin<P2, Pin1, Alternate2<Output>>; // COMP1.O, P2SELx = 10, out
        #[cfg(feature = "sac")]
        type SACp = Amplifier<Sac1>; // CPPSEL 101b: OA1O
        #[cfg(feature = "sac")]
        type SACn = Amplifier<Sac3>; // CPNSEL 101b: OA3O

        type DeviceSpecific0    = (); // Internal 1.2V reference (CPxSEL 010b). No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        // CPPSEL 101b: OA1O, P1SELx = 11
        type DeviceSpecific3Pos = Pin<P1, Pin5, Alternate3<Input<Floating>>>;
        // CPNSEL 101b: OA3O, P3SELx = 11
        type DeviceSpecific3Neg = Pin<P3, Pin5, Alternate3<Input<Floating>>>;
    }

    /// List of possible inputs to the positive input of an eCOMP comparator (CPPSEL, SLASEC4D
    /// Tables 6-23 and 6-24, p. 78).
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum PositiveInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.0 for COMP0, P2.5 for COMP1 (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-64, p. 98)
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P1.1 for COMP0, P2.4 for COMP1 (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-64, p. 98)
        COMPx_1(COMP::COMPx_1),
        /// Internal 1.2V reference: the low-power 1.2 V reference, "fixed at channel 2" (SLASEC4D
        /// 6.10.13, p. 78), 1.20 V typical (SLASEC4D Table 5-10, p. 41)
        _1V2,
        #[cfg(feature = "sac")]
        /// Output of amplifier SAC0 for eCOMP0, SAC1 for eCOMP1. (CPPSEL 101b: SLASEC4D Tables 6-23
        /// and 6-24, p. 78)
        ///
        /// Requires a reference to ensure that it has been configured.
        OAxO(&'a COMP::SACp),
        /// This eCOMP's internal 6-bit DAC (SLASEC4D 6.10.13, p. 78)
        ///
        /// Requires a reference to ensure that it has been configured.
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> PositiveInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cppsel(&self) -> u8 {
            // CPPSEL values (SLASEC4D Tables 6-23 and 6-24, p. 78)
            match self {
                PositiveInput::COMPx_0(_) => 0b000,
                PositiveInput::COMPx_1(_) => 0b001,
                PositiveInput::_1V2       => 0b010,
                #[cfg(feature = "sac")]
                PositiveInput::OAxO(_)    => 0b101,
                PositiveInput::Dac(_)     => 0b110,
            }
        }
    }

    /// List of possible inputs to the negative input of an eCOMP comparator (CPNSEL, SLASEC4D
    /// Tables 6-23 and 6-24, p. 78).
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum NegativeInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.0 for COMP0, P2.5 for COMP1 (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-64, p. 98)
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P1.1 for COMP0, P2.4 for COMP1 (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-64, p. 98)
        COMPx_1(COMP::COMPx_1),
        /// Internal 1.2V reference: the low-power 1.2 V reference, "fixed at channel 2" (SLASEC4D
        /// 6.10.13, p. 78), 1.20 V typical (SLASEC4D Table 5-10, p. 41)
        _1V2,
        #[cfg(feature = "sac")]
        /// Output of amplifier SAC2 for eCOMP0, SAC3 for eCOMP1. (CPNSEL 101b: SLASEC4D Tables 6-23
        /// and 6-24, p. 78)
        OAxO(&'a COMP::SACn),
        /// This eCOMP's internal 6-bit DAC (SLASEC4D 6.10.13, p. 78)
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> NegativeInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cpnsel(&self) -> u8 {
            // CPNSEL values (SLASEC4D Tables 6-23 and 6-24, p. 78)
            match self {
                NegativeInput::COMPx_0(_) => 0b000,
                NegativeInput::COMPx_1(_) => 0b001,
                NegativeInput::_1V2       => 0b010,
                #[cfg(feature = "sac")]
                NegativeInput::OAxO(_)    => 0b101,
                NegativeInput::Dac(_)     => 0b110,
            }
        }
    }

    // eCOMP registers (SLAU445I Table 18-1, p. 508): eCOMP0 at 08E0h (SLASEC4D Table 6-57, p. 93)
    impl_ecomp!(EComp0, cpctl0, cpctl1, cpdacctl, cpdacdata, cpint, cpiv);

    // eCOMP1 at 0900h (SLASEC4D Table 6-58, p. 93)
    impl_ecomp!(EComp1, cp1ctl0, cp1ctl1, cp1dacctl, cp1dacdata, cp1int, cp1iv);
}

/* eUSCI */
mod eusci {
    use crate::{
        hw_traits::{eusci::*, Steal},
        pac::*,
    };

    // eUSCI_A0, eUSCI_A1, eUSCI_B0 and eUSCI_B1 (SLASEC4D 6.10.8, p. 72)
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
    };

    // eUSCI_B registers in I2C mode (SLAU445I Table 24-3, p. 648): eUSCI_B0 at 0540h (SLASEC4D
    // Table 6-51, p. 90)
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
    // eUSCI_B1 at 05C0h (SLASEC4D Table 6-53, p. 91)
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
    // I2C pins, each in its PxSELx = 01 function, the macro's default Alternate1 (SLASEC4D Table 6-14,
    // p. 72; SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-66, p. 102). UCLKI is the UCSSELx = 00b clock
    // source (SLAU445I 24.4.1, p. 649), "Externally provided clock on the eUSCI_B SPI clock input pin"
    // (SLAU445I Figure 24-1, p. 628), so it is the UCBxCLK pin.

    /// I2C SCL pin for eUSCI B0: P1.3, UCB0SCL (P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0SCLPin;
    impl_i2c_pin!(UsciB0SCLPin, P1, Pin3);

    /// I2C SDA pin for eUSCI B0: P1.2, UCB0SDA (P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0SDAPin;
    impl_i2c_pin!(UsciB0SDAPin, P1, Pin2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. P1.1, UCB0CLK (P1SELx = 01:
    /// SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0UCLKIPin;
    impl_i2c_pin!(UsciB0UCLKIPin, P1, Pin1);

    /// I2C SCL pin for eUSCI B1: P4.7, UCB1SCL (P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1SCLPin;
    impl_i2c_pin!(UsciB1SCLPin, P4, Pin7);

    /// I2C SDA pin for eUSCI B1: P4.6, UCB1SDA (P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1SDAPin;
    impl_i2c_pin!(UsciB1SDAPin, P4, Pin6);

    /// UCLKI pin for eUSCI B1. Used as an external clock source. P4.5, UCB1CLK (P4SELx = 01:
    /// SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1UCLKIPin;
    impl_i2c_pin!(UsciB1UCLKIPin, P4, Pin5);

    impl I2cUsci for EUsciB0 {
        type ClockPin = UsciB0SCLPin;
        type DataPin = UsciB0SDAPin;
        type ExternalClockPin = UsciB0UCLKIPin;
    }
    impl I2cUsci for EUsciB1 {
        type ClockPin = UsciB1SCLPin;
        type DataPin = UsciB1SDAPin;
        type ExternalClockPin = UsciB1UCLKIPin;
    }
}

/* Information Memory */
/// Size of the Information Memory segment on this device, in bytes
pub const INFO_MEM_SIZE: usize = 512;

/* PWM */
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // Compare outputs TBx.n (SLASEC4D Tables 6-16 to 6-19, p. 73 to p. 75), on the pins with PxDIR = 1
    // in the pin function tables. CCR0 sets the period and has no pin (SLASEC4D 6.10.9, p. 73).

    // TB0: P1SELx = 10 (SLASEC4D Table 6-16, p. 73; SLASEC4D Table 6-63, p. 96)
    impl PwmPeriph<CCR1> for Tb0 {
        type Gpio = Pin<P1, Pin6, Alternate2<Output>>; // TB0.1, P1SELx = 10
    }
    impl PwmPeriph<CCR2> for Tb0 {
        type Gpio = Pin<P1, Pin7, Alternate2<Output>>; // TB0.2, P1SELx = 10
    }

    // TB1: P2SELx = 01 (SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-64, p. 98)
    impl PwmPeriph<CCR1> for Tb1 {
        type Gpio = Pin<P2, Pin0, Alternate1<Output>>; // TB1.1, P2SELx = 01
    }
    impl PwmPeriph<CCR2> for Tb1 {
        type Gpio = Pin<P2, Pin1, Alternate1<Output>>; // TB1.2, P2SELx = 01
    }

    // TB2: P5SELx = 01 (SLASEC4D Table 6-18, p. 74; SLASEC4D Table 6-67, p. 104)
    impl PwmPeriph<CCR1> for Tb2 {
        type Gpio = Pin<P5, Pin0, Alternate1<Output>>; // TB2.1, P5SELx = 01
    }
    impl PwmPeriph<CCR2> for Tb2 {
        type Gpio = Pin<P5, Pin1, Alternate1<Output>>; // TB2.2, P5SELx = 01
    }

    // TB3: P6SELx = 01 (SLASEC4D Table 6-19, p. 75; SLASEC4D Table 6-68, p. 106)
    impl PwmPeriph<CCR1> for Tb3 {
        type Gpio = Pin<P6, Pin0, Alternate1<Output>>; // TB3.1, P6SELx = 01
    }
    impl PwmPeriph<CCR2> for Tb3 {
        type Gpio = Pin<P6, Pin1, Alternate1<Output>>; // TB3.2, P6SELx = 01
    }
    impl PwmPeriph<CCR3> for Tb3 {
        type Gpio = Pin<P6, Pin2, Alternate1<Output>>; // TB3.3, P6SELx = 01
    }
    impl PwmPeriph<CCR4> for Tb3 {
        type Gpio = Pin<P6, Pin3, Alternate1<Output>>; // TB3.4, P6SELx = 01
    }
    impl PwmPeriph<CCR5> for Tb3 {
        type Gpio = Pin<P6, Pin4, Alternate1<Output>>; // TB3.5, P6SELx = 01
    }
    impl PwmPeriph<CCR6> for Tb3 {
        type Gpio = Pin<P6, Pin5, Alternate1<Output>>; // TB3.6, P6SELx = 01
    }
}

/* SAC */
#[cfg(feature = "sac")]
mod sac {
    use crate::pac::{Sac0, Sac1, Sac2, Sac3};
    use crate::{gpio::*, hw_traits::sac::*};

    // SAC pins, all three in their PxSELx = 11 function (the macro uses Alternate3): the OAx+ pin is
    // PSEL = 00 and the OAx- pin is NSEL = 00 (SLASEC4D Tables 6-27 to 6-30, p. 79 to p. 80), and OAxO
    // is the output pin (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-65, p. 100). The SAC registers are
    // in SLAU445I Table 20-5, p. 531.
    // SAC0: SLASEC4D Table 6-27, p. 79; registers at 0C80h (SLASEC4D Table 6-59, p. 93)
    impl_sac_periph!(
        Sac0,
        P1, Pin3, // Positive input pin: OA0+, P1SELx = 11
        P1, Pin2, // Negative input pin: OA0-, P1SELx = 11
        P1, Pin1, // Output pin: OA0O, P1SELx = 11
        sac0oa, sac0pga, sac0dac, sac0dat, sac0iv
    );
    // SAC1: SLASEC4D Table 6-29, p. 79; pins in SLASEC4D Table 6-63, p. 96; registers at 0C90h (SLASEC4D
    // Table 6-60, p. 93)
    impl_sac_periph!(
        Sac1,
        P1, Pin7, // OA1+, P1SELx = 11
        P1, Pin6, // OA1-, P1SELx = 11
        P1, Pin5, // OA1O, P1SELx = 11
        sac1oa, sac1pga, sac1dac, sac1dat, sac1iv
    );
    // SAC2: SLASEC4D Table 6-28, p. 79; pins in SLASEC4D Table 6-65, p. 100; registers at 0CA0h (SLASEC4D
    // Table 6-61, p. 93)
    impl_sac_periph!(
        Sac2,
        P3, Pin3, // OA2+, P3SELx = 11
        P3, Pin2, // OA2-, P3SELx = 11
        P3, Pin1, // OA2O, P3SELx = 11
        sac2oa, sac2pga, sac2dac, sac2dat, sac2iv
    );
    // SAC3: SLASEC4D Table 6-30, p. 80; pins in SLASEC4D Table 6-65, p. 100; registers at 0CB0h (SLASEC4D
    // Table 6-62, p. 94)
    impl_sac_periph!(
        Sac3,
        P3, Pin7, // OA3+, P3SELx = 11
        P3, Pin6, // OA3-, P3SELx = 11
        P3, Pin5, // OA3O, P3SELx = 11
        sac3oa, sac3pga, sac3dac, sac3dat, sac3iv
    );
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, serial::*};

    // eUSCI_A registers in UART mode (SLAU445I Table 22-7, p. 592): eUSCI_A0 at 0500h (SLASEC4D
    // Table 6-50, p. 90)
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

    // eUSCI_A1 at 0580h (SLASEC4D Table 6-52, p. 91)
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

    impl SerialUsci for EUsciA0 {
        type ClockPin = UsciA0ClockPin;
        type TxPin = UsciA0TxPin;
        type RxPin = UsciA0RxPin;
    }
    impl SerialUsci for EUsciA1 {
        type ClockPin = UsciA1ClockPin;
        type TxPin = UsciA1TxPin;
        type RxPin = UsciA1RxPin;
    }
    // UART pins, each in its PxSELx = 01 function, the macro's default Alternate1 (SLASEC4D Table 6-14,
    // p. 72; SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-66, p. 102). UCSSELx = 00b selects UCLK as
    // the clock source (SLAU445I 22.4.1, p. 593), an external clock of up to 24 MHz (SLASEC4D
    // Table 5-14, p. 45). The UCLK pins are taken to be the UCAxCLK pins: no document names the pin for
    // UART mode, and SLASEC4D Table 6-14, p. 72 lists no UART function on them.

    /// UCLK pin for E_USCI_A0: P1.5, UCA0CLK (P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0ClockPin;
    impl_serial_pin!(UsciA0ClockPin, P1, Pin5);

    /// Tx pin for E_USCI_A0: P1.7, UCA0TXD (P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0TxPin;
    impl_serial_pin!(UsciA0TxPin, P1, Pin7);

    /// Rx pin for E_USCI_A0: P1.6, UCA0RXD (P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0RxPin;
    impl_serial_pin!(UsciA0RxPin, P1, Pin6);

    /// UCLK pin for E_USCI_A1: P4.1, UCA1CLK (P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P4, Pin1);

    /// Tx pin for E_USCI_A1: P4.3, UCA1TXD (P4SELx = 01), or inverted (P4SELx = 10) (SLASEC4D
    /// Table 6-66, p. 102)
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P4, Pin3);
    // Alternate function 2 inverts the polarity of TXD and RXD (SLASEC4D 6.10.8, p. 73: "When PSEL = 10b,
    // the inverted UART mode is enabled"; SLASEC4D Table 6-66, p. 102: inverted UCA1TXD on P4.3 and
    // inverted UCA1RXD on P4.2 with P4SELx = 10. SLASEC4D Table 6-15, p. 73 gives RXD as P4.4, but
    // SLASEC4D Table 6-14, p. 72 and SLASEC4D Table 6-66, p. 102 put UCA1RXD on P4.2.)
    impl_serial_pin!(UsciA1TxPin, P4, Pin3, Alternate2);

    /// Rx pin for E_USCI_A1: P4.2, UCA1RXD (P4SELx = 01), or inverted (P4SELx = 10) (SLASEC4D
    /// Table 6-66, p. 102)
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P4, Pin2);
    impl_serial_pin!(UsciA1RxPin, P4, Pin2, Alternate2); // 10: UCA1RXD, inverted
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, spi::*};

    // eUSCI_A registers in SPI mode (SLAU445I Table 23-2, p. 612): eUSCI_A0 at 0500h (SLASEC4D
    // Table 6-50, p. 90)
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
    // eUSCI_A1 at 0580h (SLASEC4D Table 6-52, p. 91)
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
    // eUSCI_B registers in SPI mode (SLAU445I Table 23-11, p. 619): eUSCI_B0 at 0540h (SLASEC4D
    // Table 6-51, p. 90)
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
    // eUSCI_B1 at 05C0h (SLASEC4D Table 6-53, p. 91)
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

    impl SpiUsci for EUsciA0 {
        type MISO = UsciA0MISOPin;
        type MOSI = UsciA0MOSIPin;
        type SCLK = UsciA0SCLKPin;
        type STE = UsciA0STEPin;
    }

    impl SpiUsci for EUsciA1 {
        type MISO = UsciA1MISOPin;
        type MOSI = UsciA1MOSIPin;
        type SCLK = UsciA1SCLKPin;
        type STE = UsciA1STEPin;
    }

    impl SpiUsci for EUsciB0 {
        type MISO = UsciB0MISOPin;
        type MOSI = UsciB0MOSIPin;
        type SCLK = UsciB0SCLKPin;
        type STE = UsciB0STEPin;
    }

    impl SpiUsci for EUsciB1 {
        type MISO = UsciB1MISOPin;
        type MOSI = UsciB1MOSIPin;
        type SCLK = UsciB1SCLKPin;
        type STE = UsciB1STEPin;
    }

    // SPI pins, each in its PxSELx = 01 function, the macro's default Alternate1 (SLASEC4D Table 6-14,
    // p. 72; SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-66, p. 102)

    /// SPI MISO pin for eUSCI A0 (P1.6, UCA0SOMI, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0MISOPin;
    impl_spi_pin!(UsciA0MISOPin, P1, Pin6);

    /// SPI MOSI pin for eUSCI A0 (P1.7, UCA0SIMO, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0MOSIPin;
    impl_spi_pin!(UsciA0MOSIPin, P1, Pin7);

    /// SPI SCLK pin for eUSCI A0 (P1.5, UCA0CLK, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0SCLKPin;
    impl_spi_pin!(UsciA0SCLKPin, P1, Pin5);

    /// SPI STE pin for eUSCI A0 (P1.4, UCA0STE, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciA0STEPin;
    impl_spi_pin!(UsciA0STEPin, P1, Pin4);

    /// SPI MISO pin for eUSCI A1 (P4.2, UCA1SOMI, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P4, Pin2);

    /// SPI MOSI pin for eUSCI A1 (P4.3, UCA1SIMO, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P4, Pin3);

    /// SPI SCLK pin for eUSCI A1 (P4.1, UCA1CLK, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P4, Pin1);
    /// SPI STE pin for eUSCI A1 (P4.0, UCA1STE, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciA1STEPin;
    impl_spi_pin!(UsciA1STEPin, P4, Pin0);

    /// SPI MISO pin for eUSCI B0 (P1.3, UCB0SOMI, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0MISOPin;
    impl_spi_pin!(UsciB0MISOPin, P1, Pin3);

    /// SPI MOSI pin for eUSCI B0 (P1.2, UCB0SIMO, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0MOSIPin;
    impl_spi_pin!(UsciB0MOSIPin, P1, Pin2);

    /// SPI SCLK pin for eUSCI B0 (P1.1, UCB0CLK, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0SCLKPin;
    impl_spi_pin!(UsciB0SCLKPin, P1, Pin1);

    /// SPI STE pin for eUSCI B0 (P1.0, UCB0STE, P1SELx = 01: SLASEC4D Table 6-63, p. 96)
    pub struct UsciB0STEPin;
    impl_spi_pin!(UsciB0STEPin, P1, Pin0);

    /// SPI MISO pin for eUSCI B1 (P4.7, UCB1SOMI, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1MISOPin;
    impl_spi_pin!(UsciB1MISOPin, P4, Pin7);

    /// SPI MOSI pin for eUSCI B1 (P4.6, UCB1SIMO, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1MOSIPin;
    impl_spi_pin!(UsciB1MOSIPin, P4, Pin6);

    /// SPI SCLK pin for eUSCI B1 (P4.5, UCB1CLK, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1SCLKPin;
    impl_spi_pin!(UsciB1SCLKPin, P4, Pin5);

    /// SPI STE pin for eUSCI B1 (P4.4, UCB1STE, P4SELx = 01: SLASEC4D Table 6-66, p. 102)
    pub struct UsciB1STEPin;
    impl_spi_pin!(UsciB1STEPin, P4, Pin4);
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_b::*, Steal},
        pac::*,
        timer::*,
    };

    // Timer0_B3, Timer1_B3 and Timer2_B3 have three capture/compare registers each, Timer3_B7 seven
    // (SLASEC4D 6.10.9, p. 73). Timer_B registers: SLAU445I Table 14-5, p. 408.
    // Timer0_B3 at 0380h (SLASEC4D Table 6-45, p. 88)
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
        [CCR2, tb0cctl2, tb0ccr2]
    );

    // Timer1_B3 at 03C0h (SLASEC4D Table 6-46, p. 88)
    timer_b_impl!(
        Tb1,
        tb1,
        tb1ctl,
        tb1ex0,
        tb1iv,
        tb1r,
        tbclr,
        tbifg,
        tbidex,
        tbie,
        tbssel,
        [CCR0, tb1cctl0, tb1ccr0],
        [CCR1, tb1cctl1, tb1ccr1],
        [CCR2, tb1cctl2, tb1ccr2]
    );

    // Timer2_B3 at 0400h (SLASEC4D Table 6-47, p. 88)
    timer_b_impl!(
        Tb2,
        tb2,
        tb2ctl,
        tb2ex0,
        tb2iv,
        tb2r,
        tbclr,
        tbifg,
        tbidex,
        tbie,
        tbssel,
        [CCR0, tb2cctl0, tb2ccr0],
        [CCR1, tb2cctl1, tb2ccr1],
        [CCR2, tb2cctl2, tb2ccr2]
    );

    // Timer3_B7 at 0440h, with CCR0 to CCR6 (SLASEC4D Table 6-48, p. 89)
    timer_b_impl!(
        Tb3,
        tb3,
        tb3ctl,
        tb3ex0,
        tb3iv,
        tb3r,
        tbclr,
        tbifg,
        tbidex,
        tbie,
        tbssel,
        [CCR0, tb3cctl0, tb3ccr0],
        [CCR1, tb3cctl1, tb3ccr1],
        [CCR2, tb3cctl2, tb3ccr2],
        [CCR3, tb3cctl3, tb3ccr3],
        [CCR4, tb3cctl4, tb3ccr4],
        [CCR5, tb3cctl5, tb3ccr5],
        [CCR6, tb3cctl6, tb3ccr6]
    );

    // TBxCLK pins, the TBCLK input of each timer (SLASEC4D Tables 6-16 to 6-19, p. 73 to p. 75), each
    // with PxSELx = 01 and PxDIR = 0 (SLASEC4D Table 6-64, p. 98; SLASEC4D Table 6-67, p. 104; SLASEC4D
    // Table 6-68, p. 106). SLASEC4D Table 6-18, p. 74 lists TB2CLK on P2.7, but TB2CLK is P5.2
    // (SLASEC4D Table 6-67, p. 104; SLASEC4D Table 4-2, p. 25: TB2CLK on pin 41 of the PT package,
    // which SLASEC4D Table 4-2, p. 24 gives to P5.2).
    // TB0CLK: SLASEC4D Table 6-16, p. 73; SLASEC4D Table 6-64, p. 98
    impl TimerPeriph for Tb0 {
        type Tbxclk = Pin<P2, Pin7, Alternate1<Input<Floating>>>; // TB0CLK, P2SELx = 01
    }
    impl CapCmpTimer3 for Tb0 {}

    // TB1CLK: SLASEC4D Table 6-17, p. 74; SLASEC4D Table 6-64, p. 98
    impl TimerPeriph for Tb1 {
        type Tbxclk = Pin<P2, Pin2, Alternate1<Input<Floating>>>; // TB1CLK, P2SELx = 01
    }
    impl CapCmpTimer3 for Tb1 {}

    // TB2CLK: SLASEC4D Table 6-67, p. 104 (P5.2, not the P2.7 of SLASEC4D Table 6-18, p. 74, see above)
    impl TimerPeriph for Tb2 {
        type Tbxclk = Pin<P5, Pin2, Alternate1<Input<Floating>>>; // TB2CLK, P5SELx = 01
    }
    impl CapCmpTimer3 for Tb2 {}

    // TB3CLK: SLASEC4D Table 6-19, p. 75; SLASEC4D Table 6-68, p. 106
    impl TimerPeriph for Tb3 {
        type Tbxclk = Pin<P6, Pin6, Alternate1<Input<Floating>>>; // TB3CLK, P6SELx = 01
    }
    impl CapCmpTimer7 for Tb3 {}

    // INCLK is the CCR2 output of TB0 on TB1 ("Timer0_B3 CCR2B output", SLASEC4D Table 6-17, p. 74).
    // It isn't connected on TB0 (SLASEC4D Table 6-16, p. 73: "N/A"), and on TB2 and TB3 it is the
    // TBxCLK pin inverted (TB2CLK and TB3CLK written with an overline, SLASEC4D Table 6-18, p. 74 and
    // SLASEC4D Table 6-19, p. 75): the pin of `TimerConfig::tbclk`, counted on its falling edges, as
    // TBxR counts "with each rising edge of the clock signal" (SLAU445I 14.2.1, p. 393).
    impl CascadedTimer for Tb1 {
        type Source = Tb0;
    }

    // TODO: uncomment if pac supports it
    // // The TBxOUTH trigger is eCOMP0 or the TBxTRG pin for TB0 and TB1, eCOMP1 or the pin for TB2,
    // // and only eCOMP1 for TB3 (SLASEC4D Table 6-20, p. 76). TB0TRGSEL to TB3TRGSEL are SYSCFG2
    // // bits 15 to 12, 0 for the internal and 1 for the external source (SLAU445I 1.16.1.3, Table 1-26,
    // // p. 77).
    // high_impedance_timer_impl!(Tb0, tb0trgsel);
    // high_impedance_timer_impl!(Tb1, tb1trgsel);
    // high_impedance_timer_impl!(Tb2, tb2trgsel);
    // high_impedance_timer_impl!(Tb3, tb3trgsel);
    // // The TBxTRG pins, inputs (SLASEC4D Table 6-63, p. 96; SLASEC4D Table 6-64, p. 98; SLASEC4D
    // // Table 6-67, p. 104). TB3 has none: TB3TRGSEL = 1 is "N/A" (SLASEC4D Table 6-20, p. 76).
    // impl<PULL> HighImpedancePin<Tb0> for Pin<P1, Pin2, Alternate2<Input<PULL>>> {} // TB0TRG, P1SELx = 10
    // impl<PULL> HighImpedancePin<Tb1> for Pin<P2, Pin3, Alternate1<Input<PULL>>> {} // TB1TRG, P2SELx = 01
    // impl<PULL> HighImpedancePin<Tb2> for Pin<P5, Pin3, Alternate1<Input<PULL>>> {} // TB2TRG, P5SELx = 01
}

pub mod clock {
    use crate::gpio::*;
    use crate::_pac::cs::csctl3::Fllrefdiv;

    // The XT1 pins are defined once, here. Everything else, from the `Xt1Config` constructors to
    // keeping the pins selected through LPM3.5, derives the port, pin and PxSEL bits from these
    // types. Both pins are selected with P2SELx = 10. On 01 the pins are TB0CLK and MCLK. (SLASEC4D
    // Table 6-64, p. 98)
    /// XT1 input pin (XIN), in its XT1 function: P2.7 with P2SELx = 10 (SLASEC4D Table 6-64, p. 98)
    pub type Xt1Xin<DIR> = Pin<P2, Pin7, Alternate2<DIR>>;
    /// XT1 output pin (XOUT), in its XT1 function: P2.6 with P2SELx = 10 (SLASEC4D Table 6-64, p. 98)
    pub type Xt1Xout<DIR> = Pin<P2, Pin6, Alternate2<DIR>>;

    /// FLLREFDIV = 000b, the reference divided by 1 (SLAU445I Table 3-7, p. 116)
    pub const FLLREFDIV_1: Fllrefdiv = Fllrefdiv::_1;
}