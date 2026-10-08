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

    // P1 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2
    impl<DIR>  ToAlternate2 for Pin<P1, Pin0, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}
    impl<PULL> ToAlternate2 for Pin<P1, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin6, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin7, DIR> {}
    // P1 alternate 3
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {}
    impl<PULL> ToAlternate1 for Pin<P2, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin3, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin6, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 2
    impl ToAlternate2 for Pin<P2, Pin0, Output> {}
    impl ToAlternate2 for Pin<P2, Pin1, Output> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin6, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 3
    impl<DIR> ToAlternate3 for Pin<P2, Pin4, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P2, Pin5, DIR> {}

    // P3 alternate 1
    impl<DIR> ToAlternate1 for Pin<P3, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P3, Pin4, DIR> {}
    // P3 alternate 3
    impl<DIR> ToAlternate3 for Pin<P3, Pin1, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin2, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin3, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin5, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin6, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin7, DIR> {}

    // P4 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P4, PIN, DIR> {}
    // P4 alternate 2
    impl<DIR> ToAlternate2 for Pin<P4, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin2, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin3, DIR> {}

    // P5 alternate 1
    impl<DIR> ToAlternate1 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin1, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin2, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin3, DIR> {}
    // P5 alternate 2
    impl<DIR> ToAlternate2 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P5, Pin1, DIR> {}
    // P5 alternate 3
    impl<DIR> ToAlternate3 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin1, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin2, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin3, DIR> {}

    // P6 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P6, PIN, DIR> {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
    gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
    gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1);
    gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1);
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
    impl_adc_channel_pin!(P5, Pin0, Alternate3 => 8);
    impl_adc_channel_pin!(P5, Pin1, Alternate3 => 9);
    impl_adc_channel_pin!(P5, Pin2, Alternate3 => 10);
    impl_adc_channel_pin!(P5, Pin3, Alternate3 => 11);
}

/* Backup Memory */
/// Size of the Backup Memory segment on this device, in bytes
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::CapturePeriph, gpio::*, pac::*};

    impl CapturePeriph for Tb0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin7, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Tb1 {
        type Gpio0 = ();
        type Gpio1 = Pin<P2, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P2, Pin1, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Tb2 {
        type Gpio0 = ();
        type Gpio1 = Pin<P5, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P5, Pin1, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Tb3 {
        type Gpio0 = ();
        type Gpio1 = Pin<P6, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P6, Pin1, Alternate1<Input<Floating>>>;
        type Gpio3 = Pin<P6, Pin2, Alternate1<Input<Floating>>>;
        type Gpio4 = Pin<P6, Pin3, Alternate1<Input<Floating>>>;
        type Gpio5 = Pin<P6, Pin4, Alternate1<Input<Floating>>>;
        type Gpio6 = Pin<P6, Pin5, Alternate1<Input<Floating>>>;
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

    impl ECompInputs for EComp0 {
        type COMPx_0   = Pin<P1, Pin0, Alternate2<Input<Floating>>>;
        type COMPx_1   = Pin<P1, Pin1, Alternate2<Input<Floating>>>;
        type COMPx_2   = Infallible; // Not used
        type COMPx_3   = Infallible; // Not used
        type COMPx_Out = Pin<P2, Pin0, Alternate2<Output>>;
        #[cfg(feature = "sac")]
        type SACp = Amplifier<Sac0>;
        #[cfg(feature = "sac")]
        type SACn = Amplifier<Sac2>;
        
        type DeviceSpecific0    = (); // Internal 1.2V reference. No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        type DeviceSpecific3Pos = Pin<P1, Pin1, Alternate2<Input<Floating>>>;
        type DeviceSpecific3Neg = Pin<P3, Pin1, Alternate2<Input<Floating>>>;
    }
    impl ECompInputs for EComp1 {
        type COMPx_0   = Pin<P2, Pin5, Alternate2<Input<Floating>>>;
        type COMPx_1   = Pin<P2, Pin4, Alternate2<Input<Floating>>>;
        type COMPx_2   = Infallible; // Not used
        type COMPx_3   = Infallible; // Not used
        type COMPx_Out = Pin<P2, Pin1, Alternate2<Output>>;
        #[cfg(feature = "sac")]
        type SACp = Amplifier<Sac1>;
        #[cfg(feature = "sac")]
        type SACn = Amplifier<Sac3>;

        type DeviceSpecific0    = (); // Internal 1.2V reference. No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        type DeviceSpecific3Pos = Pin<P1, Pin5, Alternate2<Input<Floating>>>;
        type DeviceSpecific3Neg = Pin<P3, Pin5, Alternate2<Input<Floating>>>;
    }

    /// List of possible inputs to the positive input of an eCOMP comparator.
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum PositiveInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.0 for COMP0, P2.5 for COMP1
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P1.1 for COMP0, P2.4 for COMP1
        COMPx_1(COMP::COMPx_1),
        /// Internal 1.2V reference
        _1V2,
        #[cfg(feature = "sac")]
        /// Output of amplifier SAC0 for eCOMP0, SAC2 for eCOMP1.
        ///
        /// Requires a reference to ensure that it has been configured.
        OAxO(&'a COMP::SACp),
        /// This eCOMP's internal 6-bit DAC
        ///
        /// Requires a reference to ensure that it has been configured.
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> PositiveInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cppsel(&self) -> u8 {
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

    /// List of possible inputs to the negative input of an eCOMP comparator.
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum NegativeInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.0 for COMP0, P2.5 for COMP1
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P1.1 for COMP0, P2.4 for COMP1
        COMPx_1(COMP::COMPx_1),
        /// Internal 1.2V reference
        _1V2,
        #[cfg(feature = "sac")]
        /// Output of amplifier SAC1 for eCOMP0, SAC3 for eCOMP1.
        OAxO(&'a COMP::SACn),
        /// This eCOMP's internal 6-bit DAC
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> NegativeInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cpnsel(&self) -> u8 {
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

    impl_ecomp!(EComp0, cpctl0, cpctl1, cpdacctl, cpdacdata, cpint, cpiv);

    impl_ecomp!(EComp1, cp1ctl0, cp1ctl1, cp1dacctl, cp1dacdata, cp1int, cp1iv);
}

/* eUSCI */
mod eusci {
    use crate::{
        hw_traits::{eusci::*, Steal},
        pac::*,
    };

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
    /// I2C SCL pin for eUSCI B0
    pub struct UsciB0SCLPin;
    impl_i2c_pin!(UsciB0SCLPin, P1, Pin3);

    /// I2C SDA pin for eUSCI B0
    pub struct UsciB0SDAPin;
    impl_i2c_pin!(UsciB0SDAPin, P1, Pin2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source.
    pub struct UsciB0UCLKIPin;
    impl_i2c_pin!(UsciB0UCLKIPin, P1, Pin1);

    /// I2C SCL pin for eUSCI B1
    pub struct UsciB1SCLPin;
    impl_i2c_pin!(UsciB1SCLPin, P4, Pin7);

    /// I2C SDA pin for eUSCI B1
    pub struct UsciB1SDAPin;
    impl_i2c_pin!(UsciB1SDAPin, P4, Pin6);

    /// UCLKI pin for eUSCI B1. Used as an external clock source.
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
/// Size of the Information Memory segment on this device, in bytes (SLASEC4D Table 6-4, p. 65)
pub const INFO_MEM_SIZE: usize = 512;

/* PWM */
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // TB0
    impl PwmPeriph<CCR1> for Tb0 {
        type Gpio = Pin<P1, Pin6, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Tb0 {
        type Gpio = Pin<P1, Pin7, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TB1
    impl PwmPeriph<CCR1> for Tb1 {
        type Gpio = Pin<P2, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Tb1 {
        type Gpio = Pin<P2, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }

    // TB2
    impl PwmPeriph<CCR1> for Tb2 {
        type Gpio = Pin<P5, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Tb2 {
        type Gpio = Pin<P5, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }

    // TB3
    impl PwmPeriph<CCR1> for Tb3 {
        type Gpio = Pin<P6, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Tb3 {
        type Gpio = Pin<P6, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR3> for Tb3 {
        type Gpio = Pin<P6, Pin2, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR4> for Tb3 {
        type Gpio = Pin<P6, Pin3, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR5> for Tb3 {
        type Gpio = Pin<P6, Pin4, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR6> for Tb3 {
        type Gpio = Pin<P6, Pin5, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
}

/* SAC */
mod sac {
    use crate::pac::{Sac0, Sac1, Sac2, Sac3};
    use crate::{gpio::*, hw_traits::sac::*};

    impl_sac_periph!(
        Sac0, 
        P1, Pin3, // Positive input pin
        P1, Pin2, // Negative input pin
        P1, Pin1, // Output pin
        sac0oa, sac0pga, sac0dac, sac0dat
    );
    impl_sac_periph!(
        Sac1, 
        P1, Pin7, 
        P1, Pin6, 
        P1, Pin5,
        sac1oa, sac1pga, sac1dac, sac1dat
    );
    impl_sac_periph!(
        Sac2, 
        P3, Pin3, 
        P3, Pin2, 
        P3, Pin1,
        sac2oa, sac2pga, sac2dac, sac2dat
    );
    impl_sac_periph!(
        Sac3, 
        P3, Pin7, 
        P3, Pin6, 
        P3, Pin5,
        sac3oa, sac3pga, sac3dac, sac3dat
    );
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, serial::*};

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

    eusci_uart_impl!(
        EUsciA1,
        uca1ctlw0,
        uca1ctlw1,
        uca1brw,
        uca1mctlw,
        uca1statw,
        uca1rxbuf,
        uca1txbuf,
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
    /// UCLK pin for E_USCI_A0
    pub struct UsciA0ClockPin;
    impl_serial_pin!(UsciA0ClockPin, P1, Pin5);

    /// Tx pin for E_USCI_A0
    pub struct UsciA0TxPin;
    impl_serial_pin!(UsciA0TxPin, P1, Pin7);

    /// Rx pin for E_USCI_A0
    pub struct UsciA0RxPin;
    impl_serial_pin!(UsciA0RxPin, P1, Pin6);

    /// UCLK pin for E_USCI_A1
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P4, Pin1);

    /// Tx pin for E_USCI_A1
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P4, Pin3);

    /// Rx pin for E_USCI_A1
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P4, Pin2);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, spi::*};

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
    /// SPI MISO pin for eUSCI A0 (P1.6)
    pub struct UsciA0MISOPin;
    impl_spi_pin!(UsciA0MISOPin, P1, Pin6);

    /// SPI MOSI pin for eUSCI A0 (P1.7)
    pub struct UsciA0MOSIPin;
    impl_spi_pin!(UsciA0MOSIPin, P1, Pin7);

    /// SPI SCLK pin for eUSCI A0 (P1.5)
    pub struct UsciA0SCLKPin;
    impl_spi_pin!(UsciA0SCLKPin, P1, Pin5);

    /// SPI STE pin for eUSCI A0 (P1.4)
    pub struct UsciA0STEPin;
    impl_spi_pin!(UsciA0STEPin, P1, Pin4);

    /// SPI MISO pin for eUSCI A1 (P4.2)
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P4, Pin2);

    /// SPI MOSI pin for eUSCI A1 (P4.3)
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P4, Pin3);

    /// SPI SCLK pin for eUSCI A1 (P4.1)
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P4, Pin1);
    /// SPI STE pin for eUSCI A1 (P4.0)
    pub struct UsciA1STEPin;
    impl_spi_pin!(UsciA1STEPin, P4, Pin0);

    /// SPI MISO pin for eUSCI B0 (P1.3)
    pub struct UsciB0MISOPin;
    impl_spi_pin!(UsciB0MISOPin, P1, Pin3);

    /// SPI MOSI pin for eUSCI B0 (P1.2)
    pub struct UsciB0MOSIPin;
    impl_spi_pin!(UsciB0MOSIPin, P1, Pin2);

    /// SPI SCLK pin for eUSCI B0 (P1.1)
    pub struct UsciB0SCLKPin;
    impl_spi_pin!(UsciB0SCLKPin, P1, Pin1);

    /// SPI STE pin for eUSCI B0 (P1.0)
    pub struct UsciB0STEPin;
    impl_spi_pin!(UsciB0STEPin, P1, Pin0);

    /// SPI MISO pin for eUSCI B1 (P4.7)
    pub struct UsciB1MISOPin;
    impl_spi_pin!(UsciB1MISOPin, P4, Pin7);

    /// SPI MOSI pin for eUSCI B1 (P4.6)
    pub struct UsciB1MOSIPin;
    impl_spi_pin!(UsciB1MOSIPin, P4, Pin6);

    /// SPI SCLK pin for eUSCI B1 (P4.5)
    pub struct UsciB1SCLKPin;
    impl_spi_pin!(UsciB1SCLKPin, P4, Pin5);

    /// SPI STE pin for eUSCI B1 (P4.4)
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

    impl TimerPeriph for Tb0 {
        type Tbxclk = Pin<P2, Pin7, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Tb0 {}

    impl TimerPeriph for Tb1 {
        type Tbxclk = Pin<P2, Pin2, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Tb1 {}

    impl TimerPeriph for Tb2 {
        type Tbxclk = Pin<P5, Pin2, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Tb2 {}

    impl TimerPeriph for Tb3 {
        type Tbxclk = Pin<P6, Pin6, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer7 for Tb3 {}
}
