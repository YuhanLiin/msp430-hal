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

    // Define alternate pin transitions (SLASEE4C Table 6-15, p. 58 for P1; SLASEE4C Table 6-16, p. 60 for P2)

    // P1 alternate 1: UCB0STE, UCB0CLK, UCB0SIMO/UCB0SDA and UCB0SOMI/UCB0SCL on P1.0 to P1.3 with
    // USCIBRMP = 0, UCA0TXD/UCA0SIMO and UCA0RXD/UCA0SOMI on P1.4 and P1.5 with USCIARMP = 0, UCA0CLK and
    // UCA0STE on P1.6 and P1.7 (SLASEE4C Table 6-11, p. 53)
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2: P1.0 and P1.7 have none
    impl       ToAlternate2 for Pin<P1, Pin1, Output> {} // ACLK
    impl       ToAlternate2 for Pin<P1, Pin2, Output> {} // SMCLK
    impl       ToAlternate2 for Pin<P1, Pin3, Output> {} // MCLK
    impl<DIR>  ToAlternate2 for Pin<P1, Pin4, DIR> {}    // TA0.1 / TA0.CCI1A
    impl<DIR>  ToAlternate2 for Pin<P1, Pin5, DIR> {}    // TA0.2 / TA0.CCI2A
    impl<PULL> ToAlternate2 for Pin<P1, Pin6, Input<PULL>> {} // TA0CLK
    // P1 alternate 3: CapTIvate. CAP1.0 to CAP1.3 only exist on the MSP430FR2522 (SLASEE4C Table 6-15 note 3,
    // p. 58)
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin0, DIR> {} // CAP1.0
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin1, DIR> {} // CAP1.1
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin2, DIR> {} // CAP1.2
    #[cfg(feature = "msp430fr2522")]
    impl<DIR>  ToAlternate3 for Pin<P1, Pin3, DIR> {} // CAP1.3
    impl<DIR>  ToAlternate3 for Pin<P1, Pin4, DIR> {} // CAP0.0
    impl<DIR>  ToAlternate3 for Pin<P1, Pin5, DIR> {} // CAP0.1
    impl<DIR>  ToAlternate3 for Pin<P1, Pin6, DIR> {} // CAP0.2
    impl<DIR>  ToAlternate3 for Pin<P1, Pin7, DIR> {} // CAP0.3

    // P2 alternate 1: P2.5 and P2.6 have none. eUSCI_A0 on P2.0 and P2.1 needs USCIARMP = 1 (SLASEE4C
    // Table 6-11, p. 53)
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {}    // UCA0TXD/UCA0SIMO
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {}    // UCA0RXD/UCA0SOMI
    impl<DIR>  ToAlternate1 for Pin<P2, Pin2, DIR> {}    // TA1.1 / TA1.CCI1A
    impl<DIR>  ToAlternate1 for Pin<P2, Pin3, DIR> {}    // TA1.2 / TA1.CCI2A
    impl<PULL> ToAlternate1 for Pin<P2, Pin4, Input<PULL>> {} // TA1CLK
    // P2 alternate 2: the remapped eUSCI_B0 pins, USCIBRMP = 1 (SLASEE4C Table 6-11, p. 53). P2.3 to P2.6
    // only exist on the 20-pin RHL package (SLASEE4C Table 4-2, p. 14)
    impl<DIR>  ToAlternate2 for Pin<P2, Pin0, DIR> {}    // XOUT
    impl<DIR>  ToAlternate2 for Pin<P2, Pin1, DIR> {}    // XIN
    impl<PULL> ToAlternate2 for Pin<P2, Pin2, Input<PULL>> {} // CapTIvate SYNC (SLASEE4C Table 4-2, p. 13)
    impl<DIR>  ToAlternate2 for Pin<P2, Pin3, DIR> {}    // UCB0STE
    impl<DIR>  ToAlternate2 for Pin<P2, Pin4, DIR> {}    // UCB0CLK
    impl<DIR>  ToAlternate2 for Pin<P2, Pin5, DIR> {}    // UCB0SIMO/UCB0SDA
    impl<DIR>  ToAlternate2 for Pin<P2, Pin6, DIR> {}    // UCB0SOMI/UCB0SCL

    // ADC inputs A0 to A7 are enabled through SYSCFG2.ADCPCTLx, not through PxSEL (SLASEE4C Table 6-15, p. 58;
    // SLASEE4C Table 6-16, p. 60)
    impl<PIN: PinNum, DIR> ToAdcPctl for Pin<P1, PIN, DIR> where Self: adc::AdcPctlCapable {}
    impl<PIN: PinNum, DIR> ToAdcPctl for Pin<P2, PIN, DIR> where Self: adc::AdcPctlCapable {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable (SLASEE4C 6.10.3, p. 51;
    // port registers: SLASEE4C Table 6-28, p. 65; SLAU445I 8.4, p. 319)
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
}

/* ADC */
mod adc {
    use crate::{adc::*, gpio::*};

    // Channels A0 to A7 (SLASEE4C Table 6-13, p. 55)
    impl_adc_channel_pin!(P1, Pin0, AdcMode => 0);
    impl_adc_channel_pin!(P1, Pin1, AdcMode => 1);
    impl_adc_channel_pin!(P1, Pin2, AdcMode => 2);
    impl_adc_channel_pin!(P1, Pin3, AdcMode => 3);
    impl_adc_channel_pin!(P2, Pin2, AdcMode => 4);
    impl_adc_channel_pin!(P2, Pin3, AdcMode => 5);
    impl_adc_channel_pin!(P2, Pin4, AdcMode => 6);
    impl_adc_channel_pin!(P2, Pin5, AdcMode => 7);
}

/* Backup Memory */
/// Size of the Backup Memory segment on this device, in bytes (SLASEE4C 6.10.10, p. 55)
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
// CCR0 of both timers is not externally connected, so only CCR1 and CCR2 have a pin (SLASEE4C 6.10.8, p. 54)
mod capture {
    use crate::{capture::CapturePeriph, gpio::*, pac::*};

    // TA0.CCI1A on P1.4 and TA0.CCI2A on P1.5 (SLASEE4C Table 6-15, p. 58)
    impl CapturePeriph for Ta0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin4, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin5, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TA1.CCI1A on P2.2 and TA1.CCI2A on P2.3 (SLASEE4C Table 6-16, p. 60)
    impl CapturePeriph for Ta1 {
        type Gpio0 = ();
        type Gpio1 = Pin<P2, Pin2, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P2, Pin3, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
}

/* Clocks */
/// MODCLK frequency, typical (SLASEE4C Table 5-9, p. 28). The clock distribution table gives 5 MHz ±10%
/// (SLASEE4C Table 6-8, p. 49), this follows the electrical specification.
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

    // eUSCI_B0 registers: SLASEE4C Table 6-34, p. 67 to p. 68; SLAU445I 24.4, p. 648
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

    // eUSCI_B0 I2C pins (SLASEE4C Table 6-11, p. 53): SCL, SDA and the UCLKI clock, UCB0CLK, are P1.3, P1.2
    // and P1.1 with USCIBRMP = 0 (SLASEE4C Table 6-15, p. 58) and P2.6, P2.5 and P2.4 with USCIBRMP = 1
    // (SLASEE4C Table 6-16, p. 60)

    /// I2C SCL pin for eUSCI B0 (default mapping)
    pub struct UsciB0SCLPinDefault;
    impl_i2c_pin!(UsciB0SCLPinDefault, P1, Pin3);

    /// I2C SCL pin for eUSCI B0 (remapped mapping)
    pub struct UsciB0SCLPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_i2c_pin!(UsciB0SCLPinRemapped, P2, Pin6, Alternate2);

    /// I2C SDA pin for eUSCI B0 (default mapping)
    pub struct UsciB0SDAPinDefault;
    impl_i2c_pin!(UsciB0SDAPinDefault, P1, Pin2);

    /// I2C SDA pin for eUSCI B0 (remapped mapping)
    pub struct UsciB0SDAPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_i2c_pin!(UsciB0SDAPinRemapped, P2, Pin5, Alternate2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (default mapping)
    pub struct UsciB0UCLKIPinDefault;
    impl_i2c_pin!(UsciB0UCLKIPinDefault, P1, Pin1);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (remapped mapping)
    pub struct UsciB0UCLKIPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_i2c_pin!(UsciB0UCLKIPinRemapped, P2, Pin4, Alternate2);

    impl I2cUsci<DefaultMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinDefault;
        type DataPin = UsciB0SDAPinDefault;
        type ExternalClockPin = UsciB0UCLKIPinDefault;

        // USCIBRMP = 0: eUSCI_B0 on P1.0 to P1.3 (SLASEE4C 6.10.7, p. 53), SYSCFG2.USCIB0RMP is described in
        // SLAU445I Table 1-31, p. 82
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().clear_bits(|w| w.uscibrmp().clear_bit()) };
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinRemapped;
        type DataPin = UsciB0SDAPinRemapped;
        type ExternalClockPin = UsciB0UCLKIPinRemapped;

        // USCIBRMP = 1: eUSCI_B0 on P2.3 to P2.6
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
// Only CCR1 and CCR2 have output pins (SLASEE4C 1.4, p. 4; SLASEE4C Figure 6-2, p. 54)
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // TA0: TA0.1 on P1.4 and TA0.2 on P1.5 (SLASEE4C Table 6-15, p. 58)
    impl PwmPeriph<CCR1> for Ta0 {
        type Gpio = Pin<P1, Pin4, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Ta0 {
        type Gpio = Pin<P1, Pin5, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TA1: TA1.1 on P2.2 and TA1.2 on P2.3 (SLASEE4C Table 6-16, p. 60)
    impl PwmPeriph<CCR1> for Ta1 {
        type Gpio = Pin<P2, Pin2, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Ta1 {
        type Gpio = Pin<P2, Pin3, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, serial::*};

    // eUSCI_A0 registers: SLASEE4C Table 6-33, p. 67; SLAU445I 22.4, p. 592
    eusci_uart_impl!(
        EUsciA0,
        uca0ctlw0,
        uca0ctlw1,
        uca0brw,
        uca0mctlw,
        uca0statw,
        uca0rxbuf,
        uca0txbuf,
        uca0ie,
        uca0ifg,
        uca0iv,
        crate::pac::e_usci_a0::uca0statw::R
    );

    impl SerialUsci<DefaultMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinDefault;
        type TxPin = UsciA0TxPinDefault;
        type RxPin = UsciA0RxPinDefault;

        // USCIARMP = 0: TXD and RXD on P1.4 and P1.5 (SLASEE4C 6.10.7, p. 53), SYSCFG3.USCIA0RMP is described
        // in SLAU445I Table 1-32, p. 83. SLASEE4C Table 6-23, p. 64 lists no SYSCFG3, SLAU445I Table 1-28,
        // p. 79 has it at offset 26h on the MSP430FR25xx
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().clear_bits(|w| w.usciarmp().clear_bit()) };
        }
    }
    impl SerialUsci<RemappedMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinRemapped;
        type TxPin = UsciA0TxPinRemapped;
        type RxPin = UsciA0RxPinRemapped;

        // USCIARMP = 1: TXD and RXD on P2.0 and P2.1
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg3().set_bits(|w| w.usciarmp().set_bit()) };
        }
    }

    // eUSCI_A0 UART pins (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60).
    // UCA0CLK stays on P1.6 in both mappings (SLASEE4C Table 4-2 note 5, p. 14)

    /// UCLK pin for E_USCI_A0 (default mapping)
    pub struct UsciA0ClockPinDefault;
    impl_serial_pin!(UsciA0ClockPinDefault, P1, Pin6);

    /// UCLK pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0ClockPinRemapped;
    impl_serial_pin!(UsciA0ClockPinRemapped, P1, Pin6);

    /// Tx pin for E_USCI_A0 (default mapping)
    pub struct UsciA0TxPinDefault;
    impl_serial_pin!(UsciA0TxPinDefault, P1, Pin4);

    /// Tx pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0TxPinRemapped;
    impl_serial_pin!(UsciA0TxPinRemapped, P2, Pin0);

    /// Rx pin for E_USCI_A0 (default mapping)
    pub struct UsciA0RxPinDefault;
    impl_serial_pin!(UsciA0RxPinDefault, P1, Pin5);

    /// Rx pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0RxPinRemapped;
    impl_serial_pin!(UsciA0RxPinRemapped, P2, Pin1);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, spi::*};

    // eUSCI_A0 and eUSCI_B0 registers: SLASEE4C Table 6-33, p. 67 and SLASEE4C Table 6-34, p. 67 to p. 68;
    // SLAU445I 23.4, p. 612 and 23.5, p. 619
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

        // USCIARMP = 0: SIMO and SOMI on P1.4 and P1.5
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

        // USCIARMP = 1: SIMO and SOMI on P2.0 and P2.1
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

        // USCIBRMP = 0: eUSCI_B0 on P1.0 to P1.3
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

        // USCIBRMP = 1: eUSCI_B0 on P2.3 to P2.6
        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            unsafe { sys.syscfg2().set_bits(|w| w.uscibrmp().set_bit()) };
        }
    }

    // eUSCI_A0 SPI pins (SLASEE4C Table 6-11, p. 53; SLASEE4C Table 6-15, p. 58; SLASEE4C Table 6-16, p. 60).
    // SCLK and STE stay on P1.6 and P1.7 in both mappings (SLASEE4C Table 4-2 note 5, p. 14)

    /// SPI MISO pin for eUSCI A0 (P1.5) (default mapping)
    pub struct UsciA0MISOPinDefault;
    impl_spi_pin!(UsciA0MISOPinDefault, P1, Pin5);

    /// SPI MISO pin for eUSCI A0 (P2.1) (remapped mapping)
    pub struct UsciA0MISOPinRemapped;
    impl_spi_pin!(UsciA0MISOPinRemapped, P2, Pin1);

    /// SPI MOSI pin for eUSCI A0 (P1.4) (default mapping)
    pub struct UsciA0MOSIPinDefault;
    impl_spi_pin!(UsciA0MOSIPinDefault, P1, Pin4);

    /// SPI MOSI pin for eUSCI A0 (P2.0) (remapped mapping)
    pub struct UsciA0MOSIPinRemapped;
    impl_spi_pin!(UsciA0MOSIPinRemapped, P2, Pin0);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (default mapping)
    pub struct UsciA0SCLKPinDefault;
    impl_spi_pin!(UsciA0SCLKPinDefault, P1, Pin6);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (remapped mapping)
    pub struct UsciA0SCLKPinRemapped;
    impl_spi_pin!(UsciA0SCLKPinRemapped, P1, Pin6);

    /// SPI STE pin for eUSCI A0 (P1.7) (default mapping)
    pub struct UsciA0STEPinDefault;
    impl_spi_pin!(UsciA0STEPinDefault, P1, Pin7);

    /// SPI STE pin for eUSCI A0 (P1.7) (remapped mapping)
    pub struct UsciA0STEPinRemapped;
    impl_spi_pin!(UsciA0STEPinRemapped, P1, Pin7);

    // eUSCI_B0 SPI pins (SLASEE4C Table 6-11, p. 53): P1.0 to P1.3 with USCIBRMP = 0 (SLASEE4C Table 6-15,
    // p. 58), P2.3 to P2.6 with USCIBRMP = 1 (SLASEE4C Table 6-16, p. 60)

    /// SPI MISO pin for eUSCI B0 (P1.3) (default mapping)
    pub struct UsciB0MISOPinDefault;
    impl_spi_pin!(UsciB0MISOPinDefault, P1, Pin3);

    /// SPI MISO pin for eUSCI B0 (P2.6) (remapped mapping)
    pub struct UsciB0MISOPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_spi_pin!(UsciB0MISOPinRemapped, P2, Pin6, Alternate2);

    /// SPI MOSI pin for eUSCI B0 (P1.2) (default mapping)
    pub struct UsciB0MOSIPinDefault;
    impl_spi_pin!(UsciB0MOSIPinDefault, P1, Pin2);

    /// SPI MOSI pin for eUSCI B0 (P2.5) (remapped mapping)
    pub struct UsciB0MOSIPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_spi_pin!(UsciB0MOSIPinRemapped, P2, Pin5, Alternate2);

    /// SPI SCLK pin for eUSCI B0 (P1.1) (default mapping)
    pub struct UsciB0SCLKPinDefault;
    impl_spi_pin!(UsciB0SCLKPinDefault, P1, Pin1);

    /// SPI SCLK pin for eUSCI B0 (P2.4) (remapped mapping)
    pub struct UsciB0SCLKPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_spi_pin!(UsciB0SCLKPinRemapped, P2, Pin4, Alternate2);

    /// SPI STE pin for eUSCI B0 (P1.0) (default mapping)
    pub struct UsciB0STEPinDefault;
    impl_spi_pin!(UsciB0STEPinDefault, P1, Pin0);

    /// SPI STE pin for eUSCI B0 (P2.3) (remapped mapping)
    pub struct UsciB0STEPinRemapped;
    // TODO: support other mapping then only Alternate1
    // impl_spi_pin!(UsciB0STEPinRemapped, P2, Pin3, Alternate2);
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_a::*, Steal},
        pac::*,
        timer::*,
    };

    // Timer0_A3 and Timer1_A3 (SLASEE4C 6.10.8, p. 54); registers: SLASEE4C Table 6-30, p. 66 and
    // SLASEE4C Table 6-31, p. 66; SLAU445I 13.3, p. 383
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

    impl TimerPeriph for Ta0 {
        // TA0CLK on P1.6 (SLASEE4C Table 6-15, p. 58)
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta0 {}

    impl TimerPeriph for Ta1 {
        // TA1CLK on P2.4 (SLASEE4C Table 6-16, p. 60)
        type Tbxclk = Pin<P2, Pin4, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta1 {}
}
