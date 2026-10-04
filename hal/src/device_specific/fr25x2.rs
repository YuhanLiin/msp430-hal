pub use msp430fr25x2 as pac;

/// PAC with standardised peripheral names. For the FR25x2 this is just the PAC.
pub use msp430fr25x2 as _pac;

/*         GPIO          */
pub mod gpio {
    // Make PAC GPIO available as a re-export
    pub use crate::pac::{P1, P2};

    use crate::gpio::*;
    use crate::hw_traits::gpio::gpio_impl;
    use crate::adc;

    // Define alternate pin transitions (SLASEE4C Table 6-15, p. 58 for P1; SLASEE4C Table 6-16, p. 60 for
    // P2). Alternate1, 2 and 3 are PxSEL1/PxSEL0 = 01, 10 and 11, the primary, secondary and tertiary
    // module functions (SLAU445I Table 8-3, p. 314). PxSEL doesn't set the direction
    // (SLAU445I 8.2.5, p. 314), so the functions the tables list with a fixed PxDIR only take pins of
    // that direction.

    // P1 alternate 1, P1SELx = 01: eUSCI, any P1DIR (SLASEE4C Table 6-15, p. 58):
    // P1.0 UCB0STE, P1.1 UCB0CLK, P1.2 UCB0SIMO/UCB0SDA, P1.3 UCB0SOMI/UCB0SCL, all with USCIBRMP = 0;
    // P1.4 UCA0TXD/UCA0SIMO, P1.5 UCA0RXD/UCA0SOMI, both with USCIARMP = 0;
    // P1.6 UCA0CLK, P1.7 UCA0STE, with either USCIARMP (SLASEE4C Table 6-11, p. 53).
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2, P1SELx = 10 (SLASEE4C Table 6-15, p. 58). P1.0 and P1.7 have no function 10.
    impl       ToAlternate2 for Pin<P1, Pin1, Output> {} // ACLK, P1SELx = 10, P1DIR = 1
    impl       ToAlternate2 for Pin<P1, Pin2, Output> {} // SMCLK, P1SELx = 10, P1DIR = 1
    impl       ToAlternate2 for Pin<P1, Pin3, Output> {} // MCLK, P1SELx = 10, P1DIR = 1
    impl<DIR>  ToAlternate2 for Pin<P1, Pin4, DIR> {}    // TA0.1 / TA0.CCI1A, P1SELx = 10, P1DIR = 1 / 0
    impl<DIR>  ToAlternate2 for Pin<P1, Pin5, DIR> {}    // TA0.2 / TA0.CCI2A, P1SELx = 10, P1DIR = 1 / 0
    impl<PULL> ToAlternate2 for Pin<P1, Pin6, Input<PULL>> {} // TA0CLK, P1SELx = 10, P1DIR = 0
    // P1 alternate 3: CapTIvate. CAP1.0 to CAP1.3 only exist on the MSP430FR2522.
    // P1SELx = 11, any P1DIR (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-15 note 3, p. 58: "CapTIvate
    // channel 1 is available on the MSP430FR2522 only")
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin0, DIR> {} // CAP1.0, P1SELx = 11
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin1, DIR> {} // CAP1.1, P1SELx = 11
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin2, DIR> {} // CAP1.2, P1SELx = 11
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin3, DIR> {} // CAP1.3, P1SELx = 11
    impl<DIR>  ToAlternate3 for Pin<P1, Pin4, DIR> {} // CAP0.0, P1SELx = 11
    impl<DIR>  ToAlternate3 for Pin<P1, Pin5, DIR> {} // CAP0.1, P1SELx = 11
    impl<DIR>  ToAlternate3 for Pin<P1, Pin6, DIR> {} // CAP0.2, P1SELx = 11
    impl<DIR>  ToAlternate3 for Pin<P1, Pin7, DIR> {} // CAP0.3, P1SELx = 11

    // P2 alternate 1, P2SELx = 01 (SLASEE4C Table 6-16, p. 60). eUSCI_A0 uses P2.0 and P2.1 only with
    // USCIARMP = 1 (SLASEE4C Table 6-11, p. 53). P2.5 and P2.6 have no function 01.
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {}    // UCA0TXD/UCA0SIMO, P2SELx = 01, USCIARMP = 1
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {}    // UCA0RXD/UCA0SOMI, P2SELx = 01, USCIARMP = 1
    impl<DIR>  ToAlternate1 for Pin<P2, Pin2, DIR> {}    // TA1.1 / TA1.CCI1A, P2SELx = 01, P2DIR = 1 / 0
    impl<DIR>  ToAlternate1 for Pin<P2, Pin3, DIR> {}    // TA1.2 / TA1.CCI2A, P2SELx = 01, P2DIR = 1 / 0
    impl<PULL> ToAlternate1 for Pin<P2, Pin4, Input<PULL>> {} // TA1CLK, P2SELx = 01, P2DIR = 0
    // P2 alternate 2, P2SELx = 10 (SLASEE4C Table 6-16, p. 60). The eUSCI_B0 pins are its remapped ones,
    // used with USCIBRMP = 1 (SLASEE4C Table 6-11, p. 53). SYNC is the "CapTIvate synchronous trigger
    // input" (SLASEE4C Table 4-2, p. 13). P2.3 to P2.6 only exist on the 20-pin RHL package
    // (SLASEE4C Table 4-2, p. 14).
    impl<DIR>  ToAlternate2 for Pin<P2, Pin0, DIR> {}    // XOUT, P2SELx = 10
    impl<DIR>  ToAlternate2 for Pin<P2, Pin1, DIR> {}    // XIN, P2SELx = 10
    impl<PULL> ToAlternate2 for Pin<P2, Pin2, Input<PULL>> {} // CapTIvate SYNC, P2SELx = 10, P2DIR = 0
    impl<DIR>  ToAlternate2 for Pin<P2, Pin3, DIR> {}    // UCB0STE, P2SELx = 10, USCIBRMP = 1
    impl<DIR>  ToAlternate2 for Pin<P2, Pin4, DIR> {}    // UCB0CLK, P2SELx = 10, USCIBRMP = 1
    impl<DIR>  ToAlternate2 for Pin<P2, Pin5, DIR> {}    // UCB0SIMO/UCB0SDA, P2SELx = 10, USCIBRMP = 1
    impl<DIR>  ToAlternate2 for Pin<P2, Pin6, DIR> {}    // UCB0SOMI/UCB0SCL, P2SELx = 10, USCIBRMP = 1

    // ADC inputs A0 to A7 are enabled through SYSCFG2.ADCPCTLx, not through PxSEL
    // (SLASEE4C Table 6-15, p. 58 and SLASEE4C Table 6-16, p. 60: "ADCPCTLx = 1 ... from SYSCFG2";
    // SLAU445I Table 1-31, p. 82)
    impl<PIN: PinNum, DIR> ToAdcPctl for Pin<P1, PIN, DIR> where Self: adc::AdcPctlCapable {}
    impl<PIN: PinNum, DIR> ToAdcPctl for Pin<P2, PIN, DIR> where Self: adc::AdcPctlCapable {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable: P1 and P2 both have
    // edge-selectable interrupts (SLASEE4C 6.10.3, p. 51; port registers: SLASEE4C Table 6-28, p. 65;
    // descriptions in SLAU445I 8.4, p. 319). SLASEE4C Table 6-28, p. 65 gives P2SEL1 the offset 0Ch, the
    // same as P1SEL1; the PAC places it at 0Dh, as SLAU445I Table 8-4, p. 320 does.
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);

    // Pins per port (SLASEE4C 6.10.3, p. 51: "P1 implements 8 bits, and P2 implements 7 bits"). The pins a
    // port lacks are always the top ones: P2.7 here (SLASEE4C Table 6-16, p. 60).
    impl_port_pins!(P1, 8);
    impl_port_pins!(P2, 7);
}

/* ADC */
mod adc {
    use crate::{adc::*, gpio::*, pmm::VrefOutputPin};

    // The timer whose CCR1 output triggers conversions (SLASEE4C Table 6-14, p. 56, ADC trigger signal
    // connections: ADCSHSx = 10 is TA1.1B; SLASEE4C Figure 6-2, p. 54)
    impl AdcTriggerTimer for crate::pac::Ta1 {}

    // External reference inputs and the VREF+ output, selected with SYSCFG2.ADCPCTLx like the channels:
    // A0/Veref+ on P1.0 and A2/Veref- on P1.2 (SLASEE4C Table 6-13, p. 55, ADC channel connections);
    // VREF+ on P1.1, where the 1.2-V reference "can be output to P1.1/../A1/VREF+"
    // (SLASEE4C 6.10.1, p. 49), as A1,VREF+ (SLASEE4C Table 6-15, p. 58)
    impl<DIR> VeRefPlusPin for Pin<P1, Pin0, AdcMode<DIR>> {} // Veref+, ADCPCTL0 = 1
    impl<DIR> VeRefMinusPin for Pin<P1, Pin2, AdcMode<DIR>> {} // Veref-, ADCPCTL2 = 1
    impl<DIR> VrefOutputPin for Pin<P1, Pin1, AdcMode<DIR>> {} // VREF+, ADCPCTL1 = 1

    // Channels A0 to A7, ADCINCHx = 0 to 7 (SLASEE4C Table 6-13, p. 55), each pin enabled by its
    // SYSCFG2.ADCPCTLx (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60)
    impl_adc_channel_pin!(P1, Pin0, AdcMode => 0); // A0, ADCPCTL0 = 1
    impl_adc_channel_pin!(P1, Pin1, AdcMode => 1); // A1, ADCPCTL1 = 1
    impl_adc_channel_pin!(P1, Pin2, AdcMode => 2); // A2, ADCPCTL2 = 1
    impl_adc_channel_pin!(P1, Pin3, AdcMode => 3); // A3, ADCPCTL3 = 1
    impl_adc_channel_pin!(P2, Pin2, AdcMode => 4); // A4, ADCPCTL4 = 1
    impl_adc_channel_pin!(P2, Pin3, AdcMode => 5); // A5, ADCPCTL5 = 1
    impl_adc_channel_pin!(P2, Pin4, AdcMode => 6); // A6, ADCPCTL6 = 1
    impl_adc_channel_pin!(P2, Pin5, AdcMode => 7); // A7, ADCPCTL7 = 1
}

/* Backup Memory */
/// Size of the Backup Memory segment on this device, in bytes (SLASEE4C 6.10.10, p. 55: "up to 32 bytes";
/// SLASEE4C Table 6-20, p. 63: size 0020h)
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
// Capture input A of each CCR (SLASEE4C Figure 6-2, p. 54). CCR0 has no pin: "The CCR0 registers on both
// Timer0_A3 and Timer1_A3 are not externally connected" (SLASEE4C 6.10.8, p. 54). Inside the device, input
// A of TA0's CCR0 is ACLK, selected with `()`, and TA1's isn't connected, so it is `NoCapturePin`, which
// can't be selected (SLASEE4C Figure 6-2, p. 54).
mod capture {
    use crate::{capture::{CapturePeriph, NoCapturePin}, gpio::*, pac::*};

    // TA0.CCI1A on P1.4 and TA0.CCI2A on P1.5 (SLASEE4C Table 6-15, p. 58)
    impl CapturePeriph for Ta0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin4, Alternate2<Input<Floating>>>; // TA0.CCI1A, P1SELx = 10, P1DIR = 0
        type Gpio2 = Pin<P1, Pin5, Alternate2<Input<Floating>>>; // TA0.CCI2A, P1SELx = 10, P1DIR = 0
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TA1.CCI1A on P2.2 and TA1.CCI2A on P2.3 (SLASEE4C Table 6-16, p. 60)
    impl CapturePeriph for Ta1 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = Pin<P2, Pin2, Alternate1<Input<Floating>>>; // TA1.CCI1A, P2SELx = 01, P2DIR = 0
        type Gpio2 = Pin<P2, Pin3, Alternate1<Input<Floating>>>; // TA1.CCI2A, P2SELx = 01, P2DIR = 0
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
}

/* Clocks */
/// MODCLK frequency, typical (SLASEE4C Table 5-9, p. 28: 3.8 MHz to 5.8 MHz, 4.8 MHz typical). The
/// clock distribution table gives "5 MHz ±10%" instead (SLASEE4C Table 6-8, p. 49); this follows the
/// electrical specification.
pub const MODCLK_FREQ_HZ: u32 = 4_800_000;

/* eUSCI */
// The device's two eUSCI modules, eUSCI_A0 and eUSCI_B0 (SLASEE4C 6.10.7, p. 53)
mod eusci {
    use crate::{
        hw_traits::{eusci::*, Steal},
        pac::*,
    };

    eusci_steal_impl!(EUsciA0);
    eusci_steal_impl!(EUsciB0);
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

    // eUSCI_B0 registers in I2C mode: addresses in SLASEE4C Table 6-34, p. 67 to p. 68; descriptions in
    // SLAU445I 24.4, p. 648 (UCBxCTLW0: SLAU445I Table 24-4, p. 649)
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

    // eUSCI_B0 I2C pins (SLASEE4C Table 6-11, p. 53): SCL on P1.3 and SDA on P1.2 with USCIBRMP = 0, on
    // P2.6 and P2.5 with USCIBRMP = 1. The P1 pins use P1SELx = 01, Alternate1 (SLASEE4C Table 6-15, p. 58),
    // the P2 pins P2SELx = 10, Alternate2 (SLASEE4C Table 6-16, p. 60). UCLKI is the clock on the eUSCI_B
    // clock pin, UCB0CLK, selected with UCSSELx = 00 (SLAU445I Figure 24-1, p. 628;
    // SLASEE4C Table 6-8, p. 49).

    /// I2C SCL pin for eUSCI B0 (default mapping): P1.3, UCB0SCL with P1SELx = 01 and USCIBRMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0SCLPinDefault;
    impl_i2c_pin!(UsciB0SCLPinDefault, P1, Pin3);

    /// I2C SCL pin for eUSCI B0 (remapped mapping): P2.6, UCB0SCL with P2SELx = 10 and USCIBRMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0SCLPinRemapped;
    impl_i2c_pin!(UsciB0SCLPinRemapped, P2, Pin6, Alternate2);

    /// I2C SDA pin for eUSCI B0 (default mapping): P1.2, UCB0SDA with P1SELx = 01 and USCIBRMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0SDAPinDefault;
    impl_i2c_pin!(UsciB0SDAPinDefault, P1, Pin2);

    /// I2C SDA pin for eUSCI B0 (remapped mapping): P2.5, UCB0SDA with P2SELx = 10 and USCIBRMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0SDAPinRemapped;
    impl_i2c_pin!(UsciB0SDAPinRemapped, P2, Pin5, Alternate2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (default mapping): P1.1, UCB0CLK with
    /// P1SELx = 01 and USCIBRMP = 0 (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0UCLKIPinDefault;
    impl_i2c_pin!(UsciB0UCLKIPinDefault, P1, Pin1);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (remapped mapping): P2.4, UCB0CLK with
    /// P2SELx = 10 and USCIBRMP = 1 (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0UCLKIPinRemapped;
    impl_i2c_pin!(UsciB0UCLKIPinRemapped, P2, Pin4, Alternate2);

    impl I2cUsci<DefaultMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinDefault;
        type DataPin = UsciB0SDAPinDefault;
        type ExternalClockPin = UsciB0UCLKIPinDefault;

        // USCIBRMP = 0: eUSCI_B0 on P1.0 to P1.3 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG2.USCIB0RMP (SLAU445I Table 1-31, p. 82).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().clear_bits(|w| w.uscibrmp().clear_bit()) };
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinRemapped;
        type DataPin = UsciB0SDAPinRemapped;
        type ExternalClockPin = UsciB0UCLKIPinRemapped;

        // USCIBRMP = 1: eUSCI_B0 on P2.3 to P2.6 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG2.USCIB0RMP (SLAU445I Table 1-31, p. 82).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().set_bits(|w| w.uscibrmp().set_bit()) };
        }
    }
}

/* Information Memory */
/// Size of the Information Memory segment on this device, in bytes (SLASEE4C Table 6-19, p. 62: 256B,
/// 1800h to 18FFh)
pub const INFO_MEM_SIZE: usize = 256;

/* PWM */
// Only CCR1 and CCR2 have output pins (SLASEE4C 1.4, p. 4: "Only CCR1 and CCR2 are externally connected";
// SLASEE4C Figure 6-2, p. 54)
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // TA0: TA0.1 on P1.4 and TA0.2 on P1.5 (SLASEE4C Table 6-15, p. 58)
    impl PwmPeriph<CCR1> for Ta0 {
        type Gpio = Pin<P1, Pin4, Alternate2<Output>>; // TA0.1, P1SELx = 10, P1DIR = 1
    }
    impl PwmPeriph<CCR2> for Ta0 {
        type Gpio = Pin<P1, Pin5, Alternate2<Output>>; // TA0.2, P1SELx = 10, P1DIR = 1
    }

    // TA1: TA1.1 on P2.2 and TA1.2 on P2.3 (SLASEE4C Table 6-16, p. 60)
    impl PwmPeriph<CCR1> for Ta1 {
        type Gpio = Pin<P2, Pin2, Alternate1<Output>>; // TA1.1, P2SELx = 01, P2DIR = 1
    }
    impl PwmPeriph<CCR2> for Ta1 {
        type Gpio = Pin<P2, Pin3, Alternate1<Output>>; // TA1.2, P2SELx = 01, P2DIR = 1
    }
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, serial::*};

    // eUSCI_A0 registers in UART mode: addresses in SLASEE4C Table 6-33, p. 67; descriptions in
    // SLAU445I 22.4, p. 592 (UCAxCTLW0: SLAU445I Table 22-8, p. 593)
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

    impl SerialUsci<DefaultMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinDefault;
        type TxPin = UsciA0TxPinDefault;
        type RxPin = UsciA0RxPinDefault;

        // USCIARMP = 0: TXD and RXD on P1.4 and P1.5 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG3.USCIA0RMP (SLAU445I Table 1-32, p. 83). SLASEE4C Table 6-23, p. 64 lists no
        // SYSCFG3; the code follows SLASEE4C 6.10.7, p. 53 and SLAU445I Table 1-28, p. 79, which puts
        // SYSCFG3 at offset 26h on the MSP430FR25xx.
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.usciarmp().clear_bit()) };
        }
    }
    impl SerialUsci<RemappedMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinRemapped;
        type TxPin = UsciA0TxPinRemapped;
        type RxPin = UsciA0RxPinRemapped;

        // USCIARMP = 1: TXD and RXD on P2.0 and P2.1 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG3.USCIA0RMP (SLAU445I Table 1-32, p. 83).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.usciarmp().set_bit()) };
        }
    }

    // eUSCI_A0 UART pins (SLASEE4C Table 6-11, p. 53), all P1SELx or P2SELx = 01, Alternate1
    // (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60). The clock pin, UCA0CLK, is the external
    // clock of UCSSELx = 00 (SLASEE4C Table 6-8, p. 49) and stays on P1.6 in both mappings
    // (SLASEE4C Table 4-2 note 5, p. 14: "The CLK and STE assignments are fixed").

    /// UCLK pin for E_USCI_A0 (default mapping): P1.6, UCA0CLK with P1SELx = 01, with either USCIARMP
    /// (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 4-2 note 5, p. 14)
    pub struct UsciA0ClockPinDefault;
    impl_serial_pin!(UsciA0ClockPinDefault, P1, Pin6);

    /// UCLK pin for E_USCI_A0 (remapped mapping): also P1.6, UCA0CLK with P1SELx = 01, which USCIARMP
    /// doesn't move (SLASEE4C Table 6-15, p. 58; SLASEE4C Table 4-2 note 5, p. 14)
    pub struct UsciA0ClockPinRemapped;
    impl_serial_pin!(UsciA0ClockPinRemapped, P1, Pin6);

    /// Tx pin for E_USCI_A0 (default mapping): P1.4, UCA0TXD with P1SELx = 01 and USCIARMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0TxPinDefault;
    impl_serial_pin!(UsciA0TxPinDefault, P1, Pin4);

    /// Tx pin for E_USCI_A0 (remapped mapping): P2.0, UCA0TXD with P2SELx = 01 and USCIARMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciA0TxPinRemapped;
    impl_serial_pin!(UsciA0TxPinRemapped, P2, Pin0);

    /// Rx pin for E_USCI_A0 (default mapping): P1.5, UCA0RXD with P1SELx = 01 and USCIARMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0RxPinDefault;
    impl_serial_pin!(UsciA0RxPinDefault, P1, Pin5);

    /// Rx pin for E_USCI_A0 (remapped mapping): P2.1, UCA0RXD with P2SELx = 01 and USCIARMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciA0RxPinRemapped;
    impl_serial_pin!(UsciA0RxPinRemapped, P2, Pin1);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, spi::*};

    // eUSCI_A0 and eUSCI_B0 registers in SPI mode: addresses in SLASEE4C Table 6-33, p. 67 and
    // SLASEE4C Table 6-34, p. 67 to p. 68; descriptions in SLAU445I 23.4, p. 612 (eUSCI_A) and
    // SLAU445I 23.5, p. 619 (eUSCI_B)
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

    impl SpiUsci<DefaultMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinDefault;
        type MOSI = UsciA0MOSIPinDefault;
        type SCLK = UsciA0SCLKPinDefault;
        type STE = UsciA0STEPinDefault;

        // USCIARMP = 0: SIMO and SOMI on P1.4 and P1.5 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG3.USCIA0RMP (SLAU445I Table 1-32, p. 83).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.usciarmp().clear_bit()) };
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinRemapped;
        type MOSI = UsciA0MOSIPinRemapped;
        type SCLK = UsciA0SCLKPinRemapped;
        type STE = UsciA0STEPinRemapped;

        // USCIARMP = 1: SIMO and SOMI on P2.0 and P2.1 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG3.USCIA0RMP (SLAU445I Table 1-32, p. 83).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.usciarmp().set_bit()) };
        }
    }

    impl SpiUsci<DefaultMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinDefault;
        type MOSI = UsciB0MOSIPinDefault;
        type SCLK = UsciB0SCLKPinDefault;
        type STE = UsciB0STEPinDefault;

        // USCIBRMP = 0: eUSCI_B0 on P1.0 to P1.3 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG2.USCIB0RMP (SLAU445I Table 1-31, p. 82).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().clear_bits(|w| w.uscibrmp().clear_bit()) };
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinRemapped;
        type MOSI = UsciB0MOSIPinRemapped;
        type SCLK = UsciB0SCLKPinRemapped;
        type STE = UsciB0STEPinRemapped;

        // USCIBRMP = 1: eUSCI_B0 on P2.3 to P2.6 (SLASEE4C 6.10.7, p. 53; SLASEE4C Table 6-11, p. 53).
        // The bit is SYSCFG2.USCIB0RMP (SLAU445I Table 1-31, p. 82).
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().set_bits(|w| w.uscibrmp().set_bit()) };
        }
    }

    // eUSCI_A0 SPI pins (SLASEE4C Table 6-11, p. 53): SIMO, SOMI, SCLK and STE on P1.4 to P1.7 with
    // USCIARMP = 0; SIMO and SOMI move to P2.0 and P2.1 with USCIARMP = 1, while SCLK and STE stay on P1.6
    // and P1.7 (SLASEE4C Table 4-2 note 5, p. 14: "The CLK and STE assignments are fixed and shared by
    // both SPI function groups"). All use P1SELx or P2SELx = 01, Alternate1 (SLASEE4C Table 6-15, p. 58;
    // SLASEE4C Table 6-16, p. 60).

    /// SPI MISO pin for eUSCI A0 (P1.5) (default mapping): UCA0SOMI, P1SELx = 01, USCIARMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0MISOPinDefault;
    impl_spi_pin!(UsciA0MISOPinDefault, P1, Pin5);

    /// SPI MISO pin for eUSCI A0 (P2.1) (remapped mapping): UCA0SOMI, P2SELx = 01, USCIARMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciA0MISOPinRemapped;
    impl_spi_pin!(UsciA0MISOPinRemapped, P2, Pin1);

    /// SPI MOSI pin for eUSCI A0 (P1.4) (default mapping): UCA0SIMO, P1SELx = 01, USCIARMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0MOSIPinDefault;
    impl_spi_pin!(UsciA0MOSIPinDefault, P1, Pin4);

    /// SPI MOSI pin for eUSCI A0 (P2.0) (remapped mapping): UCA0SIMO, P2SELx = 01, USCIARMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciA0MOSIPinRemapped;
    impl_spi_pin!(UsciA0MOSIPinRemapped, P2, Pin0);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (default mapping): UCA0CLK, P1SELx = 01, USCIARMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0SCLKPinDefault;
    impl_spi_pin!(UsciA0SCLKPinDefault, P1, Pin6);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (remapped mapping): UCA0CLK, P1SELx = 01, on P1.6 with
    /// USCIARMP = 1 as well (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0SCLKPinRemapped;
    impl_spi_pin!(UsciA0SCLKPinRemapped, P1, Pin6);

    /// SPI STE pin for eUSCI A0 (P1.7) (default mapping): UCA0STE, P1SELx = 01, USCIARMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0STEPinDefault;
    impl_spi_pin!(UsciA0STEPinDefault, P1, Pin7);

    /// SPI STE pin for eUSCI A0 (P1.7) (remapped mapping): UCA0STE, P1SELx = 01, on P1.7 with
    /// USCIARMP = 1 as well (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciA0STEPinRemapped;
    impl_spi_pin!(UsciA0STEPinRemapped, P1, Pin7);

    // eUSCI_B0 SPI pins (SLASEE4C Table 6-11, p. 53): STE, SCLK, SIMO and SOMI on P1.0 to P1.3 with
    // USCIBRMP = 0, P1SELx = 01, Alternate1 (SLASEE4C Table 6-15, p. 58); on P2.3 to P2.6 with
    // USCIBRMP = 1, P2SELx = 10, Alternate2 (SLASEE4C Table 6-16, p. 60).

    /// SPI MISO pin for eUSCI B0 (P1.3) (default mapping): UCB0SOMI, P1SELx = 01, USCIBRMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0MISOPinDefault;
    impl_spi_pin!(UsciB0MISOPinDefault, P1, Pin3);

    /// SPI MISO pin for eUSCI B0 (P2.6) (remapped mapping): UCB0SOMI, P2SELx = 10, USCIBRMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0MISOPinRemapped;
    impl_spi_pin!(UsciB0MISOPinRemapped, P2, Pin6, Alternate2);

    /// SPI MOSI pin for eUSCI B0 (P1.2) (default mapping): UCB0SIMO, P1SELx = 01, USCIBRMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0MOSIPinDefault;
    impl_spi_pin!(UsciB0MOSIPinDefault, P1, Pin2);

    /// SPI MOSI pin for eUSCI B0 (P2.5) (remapped mapping): UCB0SIMO, P2SELx = 10, USCIBRMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0MOSIPinRemapped;
    impl_spi_pin!(UsciB0MOSIPinRemapped, P2, Pin5, Alternate2);

    /// SPI SCLK pin for eUSCI B0 (P1.1) (default mapping): UCB0CLK, P1SELx = 01, USCIBRMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0SCLKPinDefault;
    impl_spi_pin!(UsciB0SCLKPinDefault, P1, Pin1);

    /// SPI SCLK pin for eUSCI B0 (P2.4) (remapped mapping): UCB0CLK, P2SELx = 10, USCIBRMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0SCLKPinRemapped;
    impl_spi_pin!(UsciB0SCLKPinRemapped, P2, Pin4, Alternate2);

    /// SPI STE pin for eUSCI B0 (P1.0) (default mapping): UCB0STE, P1SELx = 01, USCIBRMP = 0
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58)
    pub struct UsciB0STEPinDefault;
    impl_spi_pin!(UsciB0STEPinDefault, P1, Pin0);

    /// SPI STE pin for eUSCI B0 (P2.3) (remapped mapping): UCB0STE, P2SELx = 10, USCIBRMP = 1
    /// (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-16, p. 60)
    pub struct UsciB0STEPinRemapped;
    impl_spi_pin!(UsciB0STEPinRemapped, P2, Pin3, Alternate2);
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_a::*, Steal},
        pac::*,
        timer::*,
    };

    // Timer0_A3 and Timer1_A3, three capture/compare registers each (SLASEE4C 6.10.8, p. 54; registers:
    // SLASEE4C Table 6-30, p. 66 and SLASEE4C Table 6-31, p. 66; descriptions in SLAU445I 13.3, p. 383)
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

    // External clock pins, TASSEL = 00 (SLASEE4C Table 6-8, p. 49)
    impl TimerPeriph for Ta0 {
        // TA0CLK on P1.6: P1SELx = 10 with P1DIR = 0 (SLASEE4C Table 6-15, p. 58)
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    }
    // Three capture/compare registers (SLASEE4C 6.10.8, p. 54)
    impl CapCmpTimer3 for Ta0 {}

    impl TimerPeriph for Ta1 {
        // TA1CLK on P2.4: P2SELx = 01 with P2DIR = 0 (SLASEE4C Table 6-16, p. 60)
        type Tbxclk = Pin<P2, Pin4, Alternate1<Input<Floating>>>;
    }
    // Three capture/compare registers (SLASEE4C 6.10.8, p. 54)
    impl CapCmpTimer3 for Ta1 {}

    // INCLK is the VLO on TA0 and the CCR2 output of TA0 on TA1, both TASSEL = 11
    // (SLASEE4C Figure 6-2, p. 54; SLASEE4C Table 6-8, p. 49 lists VLOCLK, 11b, for TA0 only)
    impl VloclkTimer for Ta0 {}
    impl CascadedTimer for Ta1 {
        type Source = Ta0;
    }
}

pub mod clock {
    use crate::gpio::*;
    use crate::_pac::cs::csctl3::Fllrefdiv;

    // The XT1 pins are defined once, here. Everything else, from the `Xt1Config` constructors to
    // keeping the pins selected through LPM3.5, derives the port, pin and PxSEL bits from these
    // types. Both pins are selected with P2SELx = 10. On 01 the pins are UCA0 (SLASEE4C Table 6-16, p. 60).
    /// XT1 input pin (XIN), in its XT1 function: P2.1 with P2SELx = 10 (SLASEE4C Table 6-16, p. 60)
    pub type Xt1Xin<DIR> = Pin<P2, Pin1, Alternate2<DIR>>;
    /// XT1 output pin (XOUT), in its XT1 function: P2.0 with P2SELx = 10 (SLASEE4C Table 6-16, p. 60)
    pub type Xt1Xout<DIR> = Pin<P2, Pin0, Alternate2<DIR>>;

    /// FLLREFDIV = 000b, the reference divided by 1 (SLAU445I Table 3-7, p. 116)
    pub const FLLREFDIV_1: Fllrefdiv = Fllrefdiv::_1;
}

/* LPM */
// The device's ports, P1 and P2 (SLASEE4C 6.10.3, p. 51)
pub(crate) mod lpm {
    crate::lpm::reset_all_pin_functions_impl!(P1, P2);
}