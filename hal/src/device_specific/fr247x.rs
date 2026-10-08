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

    // P1 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2
    impl<PULL> ToAlternate2 for Pin<P1, Pin0, Input<PULL>> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin2, DIR> {}
    impl ToAlternate2 for Pin<P1, Pin3, Output> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin4, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin5, DIR> {}
    impl<PULL> ToAlternate2 for Pin<P1, Pin6, Input<PULL>> {}
    impl ToAlternate2 for Pin<P1, Pin7, Output> {}
    // P1 alternate 3
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1
    impl<DIR> ToAlternate1 for Pin<P2, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin1, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin3, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin4, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin5, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin6, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 2
    impl ToAlternate2 for Pin<P2, Pin2, Output> {}
    // P2 alternate 3
    impl<DIR> ToAlternate3 for Pin<P2, Pin2, DIR> {}

    // P3 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P3, Pin0, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P3, Pin1, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P3, Pin2, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P3, Pin3, DIR> {}
    impl<PULL> ToAlternate1 for Pin<P3, Pin4, Input<PULL>> {}
    impl<DIR>  ToAlternate1 for Pin<P3, Pin5, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P3, Pin6, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P3, Pin7, DIR> {}
    // P3 alternate 2
    impl ToAlternate2 for Pin<P3, Pin4, Output> {}
    impl<PULL> ToAlternate2 for Pin<P3, Pin5, Input<PULL>> {}


    // P4 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P4, Pin0, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P4, Pin1, DIR> {}
    impl<PULL> ToAlternate1 for Pin<P4, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate1 for Pin<P4, Pin3, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P4, Pin4, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P4, Pin5, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P4, Pin6, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P4, Pin7, DIR> {}
    // P4 alternate 2
    impl<DIR> ToAlternate2 for Pin<P4, Pin3, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin4, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin5, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin6, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin7, DIR> {}
    // P4 alternate 3
    impl<DIR> ToAlternate3 for Pin<P4, Pin3, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P4, Pin4, DIR> {}

    // P5 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P5, PIN, DIR> {}
    // P5 alternate 2
    impl<DIR>  ToAlternate2 for Pin<P5, Pin0, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P5, Pin1, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P5, Pin2, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P5, Pin3, DIR> {}
    impl<PULL> ToAlternate2 for Pin<P5, Pin4, Input<PULL>> {}
    impl<PULL> ToAlternate2 for Pin<P5, Pin5, Input<PULL>> {}
    impl<DIR>  ToAlternate2 for Pin<P5, Pin6, DIR> {}
    // P5 alternate 3
    impl<DIR> ToAlternate3 for Pin<P5, Pin3, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin4, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin7, DIR> {}

    // P6 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P6, Pin0, DIR> {}
    impl<PULL> ToAlternate1 for Pin<P6, Pin1, Input<PULL>> {}
    impl<DIR>  ToAlternate1 for Pin<P6, Pin2, DIR> {}
    // P6 alternate 3
    impl<DIR> ToAlternate3 for Pin<P6, Pin0, DIR> {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
    gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
    gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1, [p5ies, p5ie, p5ifg, p5iv]);
    gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1, [p6ies, p6ie, p6ifg, p6iv]);
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
/// Size of the Backup Memory (BAKMEM register block) on this device, in bytes (SLASEO7C 9.10.10, p. 61)
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::CapturePeriph, gpio::*, pac::*, pin_mapping::*};

    impl CapturePeriph for Ta0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin1, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin2, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Ta1 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin5, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin4, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph<DefaultMapping> for Ta2 {
        type Gpio0 = Pin<P2, Pin3, Alternate1<Input<Floating>>>;
        type Gpio1 = Pin<P3, Pin3, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P3, Pin0, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
    impl CapturePeriph<RemappedMapping> for Ta2 {
        type Gpio0 = Pin<P5, Pin6, Alternate2<Input<Floating>>>;
        type Gpio1 = Pin<P5, Pin7, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P6, Pin0, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph<DefaultMapping> for Ta3 {
        type Gpio0 = Pin<P4, Pin1, Alternate1<Input<Floating>>>;
        type Gpio1 = Pin<P4, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P3, Pin7, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
    impl CapturePeriph<RemappedMapping> for Ta3 {
        type Gpio0 = Pin<P5, Pin3, Alternate2<Input<Floating>>>;
        type Gpio1 = Pin<P4, Pin6, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P4, Pin5, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Tb0 {
        type Gpio0 = Pin<P6, Pin2, Alternate1<Input<Floating>>>;
        type Gpio1 = Pin<P4, Pin7, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P5, Pin0, Alternate2<Input<Floating>>>;
        type Gpio3 = Pin<P5, Pin1, Alternate2<Input<Floating>>>;
        type Gpio4 = Pin<P5, Pin2, Alternate2<Input<Floating>>>;
        type Gpio5 = Pin<P4, Pin3, Alternate2<Input<Floating>>>;
        type Gpio6 = Pin<P4, Pin6, Alternate2<Input<Floating>>>;
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

    impl ECompInputs for EComp0 {
        type COMPx_0   = Pin<P1, Pin1, Alternate3<Input<Floating>>>;
        type COMPx_1   = Pin<P2, Pin2, Alternate3<Input<Floating>>>;
        type COMPx_2   = Pin<P5, Pin7, Alternate3<Input<Floating>>>;
        type COMPx_3   = Pin<P6, Pin0, Alternate3<Input<Floating>>>;
        type COMPx_Out = Pin<P3, Pin4, Alternate2<Output>>;

        type DeviceSpecific0    = (); // Internal 1.2V reference. No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        type DeviceSpecific3Pos = Infallible; // Not used
        type DeviceSpecific3Neg = Infallible; // Not used
    }

    /// List of possible inputs to the positive input of an eCOMP comparator.
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum PositiveInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.1 for COMP0
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P2.2 for COMP0
        COMPx_1(COMP::COMPx_1),
        /// COMPx.2. P5.7 for COMP0
        COMPx_2(COMP::COMPx_2),
        /// COMPx.3. P6.0 for COMP0
        COMPx_3(COMP::COMPx_3),
        /// Internal 1.2V reference
        _1V2,
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
                PositiveInput::COMPx_2(_) => 0b011,
                PositiveInput::COMPx_3(_) => 0b100,
                PositiveInput::_1V2       => 0b010,
                PositiveInput::Dac(_)     => 0b110,
            }
        }
    }

    /// List of possible inputs to the negative input of an eCOMP comparator.
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum NegativeInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.1 for COMP0
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P2.2 for COMP0
        COMPx_1(COMP::COMPx_1),
        /// COMPx.2. P5.7 for COMP0
        COMPx_2(COMP::COMPx_2),
        /// COMPx.3. P6.0 for COMP0
        COMPx_3(COMP::COMPx_3),
        /// Internal 1.2V reference
        _1V2,
        /// This eCOMP's internal 6-bit DAC
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> NegativeInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cpnsel(&self) -> u8 {
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

    impl_ecomp!(EComp0, cp0ctl0, cp0ctl1, cp0dacctl, cp0dacdata, cpint, cpiv);
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
        pin_mapping::*,
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
    /// I2C SCL pin for eUSCI B0 (default mapping)
    pub struct UsciB0SCLPinDefault;
    impl_i2c_pin!(UsciB0SCLPinDefault, P1, Pin3);

    /// I2C SCL pin for eUSCI B0 (remapped mapping)
    pub struct UsciB0SCLPinRemapped;
    impl_i2c_pin!(UsciB0SCLPinRemapped, P4, Pin5);

    /// I2C SDA pin for eUSCI B0 (default mapping)
    pub struct UsciB0SDAPinDefault;
    impl_i2c_pin!(UsciB0SDAPinDefault, P1, Pin2);

    /// I2C SDA pin for eUSCI B0 (remapped mapping)
    pub struct UsciB0SDAPinRemapped;
    impl_i2c_pin!(UsciB0SDAPinRemapped, P4, Pin6);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (default mapping)
    pub struct UsciB0UCLKIPinDefault;
    impl_i2c_pin!(UsciB0UCLKIPinDefault, P1, Pin1);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (remapped mapping)
    pub struct UsciB0UCLKIPinRemapped;
    impl_i2c_pin!(UsciB0UCLKIPinRemapped, P5, Pin5);

    /// I2C SCL pin for eUSCI B1 (default mapping)
    pub struct UsciB1SCLPinDefault;
    impl_i2c_pin!(UsciB1SCLPinDefault, P3, Pin6);

    /// I2C SCL pin for eUSCI B1 (remapped mapping)
    pub struct UsciB1SCLPinRemapped;
    impl_i2c_pin!(UsciB1SCLPinRemapped, P4, Pin3);

    /// I2C SDA pin for eUSCI B1 (default mapping)
    pub struct UsciB1SDAPinDefault;
    impl_i2c_pin!(UsciB1SDAPinDefault, P3, Pin2);

    /// I2C SDA pin for eUSCI B1 (remapped mapping)
    pub struct UsciB1SDAPinRemapped;
    impl_i2c_pin!(UsciB1SDAPinRemapped, P4, Pin4);

    /// UCLKI pin for eUSCI B1. Used as an external clock source. (default mapping)
    pub struct UsciB1UCLKIPinDefault;
    impl_i2c_pin!(UsciB1UCLKIPinDefault, P3, Pin5);

    /// UCLKI pin for eUSCI B1. Used as an external clock source. (remapped mapping)
    pub struct UsciB1UCLKIPinRemapped;
    impl_i2c_pin!(UsciB1UCLKIPinRemapped, P5, Pin3);

    impl I2cUsci<DefaultMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinDefault;
        type DataPin = UsciB0SDAPinDefault;
        type ExternalClockPin = UsciB0UCLKIPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscib0rmp().clear_bit());
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinRemapped;
        type DataPin = UsciB0SDAPinRemapped;
        type ExternalClockPin = UsciB0UCLKIPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscib0rmp().set_bit());
        }
    }

    impl I2cUsci<DefaultMapping> for EUsciB1 {
        type ClockPin = UsciB1SCLPinDefault;
        type DataPin = UsciB1SDAPinDefault;
        type ExternalClockPin = UsciB1UCLKIPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscib1rmp().clear_bit());
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB1 {
        type ClockPin = UsciB1SCLPinRemapped;
        type DataPin = UsciB1SDAPinRemapped;
        type ExternalClockPin = UsciB1UCLKIPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscib1rmp().set_bit());
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

    impl SerialUsci<DefaultMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinDefault;
        type TxPin = UsciA0TxPinDefault;
        type RxPin = UsciA0RxPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscia0rmp().clear_bit());
        }
    }
    impl SerialUsci<RemappedMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinRemapped;
        type TxPin = UsciA0TxPinRemapped;
        type RxPin = UsciA0RxPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscia0rmp().set_bit());
        }
    }

    impl SerialUsci for EUsciA1 {
        type ClockPin = UsciA1ClockPin;
        type TxPin = UsciA1TxPin;
        type RxPin = UsciA1RxPin;
    }

    // Table 9-11

    /// UCLK pin for E_USCI_A0 (default mapping)
    pub struct UsciA0ClockPinDefault;
    // Default pin mapping for the eUSCI_A0 clock signal.
    // Active when the USCIA0RMP remap bit in SYSCFG2/SYSCFG3 is cleared.
    impl_serial_pin!(UsciA0ClockPinDefault, P1, Pin6);

    /// UCLK pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0ClockPinRemapped;
    // Alternate pin mapping selected when the USCIA0RMP remap bit is set.
    // Only one mapping (default or remapped) is active at a time.
    impl_serial_pin!(UsciA0ClockPinRemapped, P5, Pin0);

    /// Tx pin for E_USCI_A0 (default mapping)
    pub struct UsciA0TxPinDefault;
    // Default transmit pin mapping.
    // Used when the USCIA0RMP remap bit in SYSCFG2/SYSCFG3 is cleared.
    impl_serial_pin!(UsciA0TxPinDefault, P1, Pin4);

    /// Tx pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0TxPinRemapped;
    // Alternate transmit pin mapping selected via the USCIA0RMP remap bit.
    // Only one mapping (default or remapped) is active at a time.
    impl_serial_pin!(UsciA0TxPinRemapped, P5, Pin2);

    /// Rx pin for E_USCI_A0 (default mapping)
    pub struct UsciA0RxPinDefault;
    // Default receive pin mapping.
    // Active when the USCIA0RMP remap bit in SYSCFG2/SYSCFG3 is cleared.
    impl_serial_pin!(UsciA0RxPinDefault, P1, Pin5);

    /// Rx pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0RxPinRemapped;
    // Alternate receive pin mapping selected when the USCIA0RMP remap bit is set.
    // Only one mapping (default or remapped) is active at a time.
    impl_serial_pin!(UsciA0RxPinRemapped, P5, Pin1);

    /// UCLK pin for E_USCI_A1
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P2, Pin4);

    /// Tx pin for E_USCI_A1
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P2, Pin6);

    /// Rx pin for E_USCI_A1
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P2, Pin5);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, spi::*};

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

    impl SpiUsci<DefaultMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinDefault;
        type MOSI = UsciA0MOSIPinDefault;
        type SCLK = UsciA0SCLKPinDefault;
        type STE = UsciA0STEPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscia0rmp().clear_bit());
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinRemapped;
        type MOSI = UsciA0MOSIPinRemapped;
        type SCLK = UsciA0SCLKPinRemapped;
        type STE = UsciA0STEPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscia0rmp().set_bit());
        }
    }

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
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscib0rmp().clear_bit());
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinRemapped;
        type MOSI = UsciB0MOSIPinRemapped;
        type SCLK = UsciB0SCLKPinRemapped;
        type STE = UsciB0STEPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscib0rmp().set_bit());
        }
    }

    impl SpiUsci<DefaultMapping> for EUsciB1 {
        type MISO = UsciB1MISOPinDefault;
        type MOSI = UsciB1MOSIPinDefault;
        type SCLK = UsciB1SCLKPinDefault;
        type STE = UsciB1STEPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscib1rmp().clear_bit());
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciB1 {
        type MISO = UsciB1MISOPinRemapped;
        type MOSI = UsciB1MOSIPinRemapped;
        type SCLK = UsciB1SCLKPinRemapped;
        type STE = UsciB1STEPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.uscib1rmp().set_bit());
        }
    }
    /// SPI MISO pin for eUSCI A0 (P1.5) (default mapping)
    pub struct UsciA0MISOPinDefault;
    impl_spi_pin!(UsciA0MISOPinDefault, P1, Pin5);

    /// SPI MISO pin for eUSCI A0 (P5.1) (remapped mapping)
    pub struct UsciA0MISOPinRemapped;
    impl_spi_pin!(UsciA0MISOPinRemapped, P5, Pin1);

    /// SPI MOSI pin for eUSCI A0 (P1.4) (default mapping)
    pub struct UsciA0MOSIPinDefault;
    impl_spi_pin!(UsciA0MOSIPinDefault, P1, Pin4);

    /// SPI MOSI pin for eUSCI A0 (P5.2) (remapped mapping)
    pub struct UsciA0MOSIPinRemapped;
    impl_spi_pin!(UsciA0MOSIPinRemapped, P5, Pin2);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (default mapping)
    pub struct UsciA0SCLKPinDefault;
    impl_spi_pin!(UsciA0SCLKPinDefault, P1, Pin6);

    /// SPI SCLK pin for eUSCI A0 (P5.0) (remapped mapping)
    pub struct UsciA0SCLKPinRemapped;
    impl_spi_pin!(UsciA0SCLKPinRemapped, P5, Pin0);

    /// SPI STE pin for eUSCI A0 (P1.7) (default mapping)
    pub struct UsciA0STEPinDefault;
    impl_spi_pin!(UsciA0STEPinDefault, P1, Pin7);

    /// SPI STE pin for eUSCI A0 (P4.7) (remapped mapping)
    pub struct UsciA0STEPinRemapped;
    impl_spi_pin!(UsciA0STEPinRemapped, P4, Pin7);

    /// SPI MISO pin for eUSCI A1 (P2.5) (default mapping)
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P2, Pin5);

    /// SPI MISO pin for eUSCI A1 (P2.6) (default mapping)
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P2, Pin6);

    /// SPI SCLK pin for eUSCI A1 (P2.4) (default mapping)
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P2, Pin4);
    /// SPI STE pin for eUSCI A1 (P3.1) (default mapping)
    pub struct UsciA1STEPin;
    impl_spi_pin!(UsciA1STEPin, P3, Pin1);

    /// SPI MISO pin for eUSCI B0 (P1.2) (default mapping)
    pub struct UsciB0MISOPinDefault;
    impl_spi_pin!(UsciB0MISOPinDefault, P1, Pin3);

    /// SPI MISO pin for eUSCI A0 (P4.5) (remapped mapping)
    pub struct UsciB0MISOPinRemapped;
    impl_spi_pin!(UsciB0MISOPinRemapped, P4, Pin5);

    /// SPI MOSI pin for eUSCI B0 (P1.3) (default mapping)
    pub struct UsciB0MOSIPinDefault;
    impl_spi_pin!(UsciB0MOSIPinDefault, P1, Pin2);

    /// SPI MOSI pin for eUSCI A0 (P4.6) (remapped mapping)
    pub struct UsciB0MOSIPinRemapped;
    impl_spi_pin!(UsciB0MOSIPinRemapped, P4, Pin6);

    /// SPI SCLK pin for eUSCI B0 (P1.1) (default mapping)
    pub struct UsciB0SCLKPinDefault;
    impl_spi_pin!(UsciB0SCLKPinDefault, P1, Pin1);

    /// SPI SCLK pin for eUSCI A0 (P5.5) (remapped mapping)
    pub struct UsciB0SCLKPinRemapped;
    impl_spi_pin!(UsciB0SCLKPinRemapped, P5, Pin5);

    /// SPI STE pin for eUSCI B0 (P1.0) (default mapping)
    pub struct UsciB0STEPinDefault;
    impl_spi_pin!(UsciB0STEPinDefault, P1, Pin0);

    /// SPI STE pin for eUSCI A0 (P5.6) (remapped mapping)
    pub struct UsciB0STEPinRemapped;
    impl_spi_pin!(UsciB0STEPinRemapped, P5, Pin6);

    /// SPI MISO pin for eUSCI B1 (P3.2) (default mapping)
    pub struct UsciB1MISOPinDefault;
    impl_spi_pin!(UsciB1MISOPinDefault, P3, Pin6);

    /// SPI MISO pin for eUSCI B1 (P4.3) (remapped mapping)
    pub struct UsciB1MISOPinRemapped;
    impl_spi_pin!(UsciB1MISOPinRemapped, P4, Pin3);

    /// SPI MOSI pin for eUSCI B1 (P3.6) (default mapping)
    pub struct UsciB1MOSIPinDefault;
    impl_spi_pin!(UsciB1MOSIPinDefault, P3, Pin2);

    /// SPI MOSI pin for eUSCI B1 (P4.4) (remapped mapping)
    pub struct UsciB1MOSIPinRemapped;
    impl_spi_pin!(UsciB1MOSIPinRemapped, P4, Pin4);

    /// SPI SCLK pin for eUSCI B1 (P3.5) (default mapping)
    pub struct UsciB1SCLKPinDefault;
    impl_spi_pin!(UsciB1SCLKPinDefault, P3, Pin5);

    /// SPI SCLK pin for eUSCI B1 (P5.3) (remapped mapping)
    pub struct UsciB1SCLKPinRemapped;
    impl_spi_pin!(UsciB1SCLKPinRemapped, P5, Pin3);

    /// SPI STE pin for eUSCI B1 (P2.7) (default mapping)
    pub struct UsciB1STEPinDefault;
    impl_spi_pin!(UsciB1STEPinDefault, P2, Pin7);

    /// SPI STE pin for eUSCI B1 (P5.4) (remapped mapping)
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

    impl TimerPeriph for Ta0 {
        type Tbxclk = Pin<P1, Pin0, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta0 {}

    impl TimerPeriph for Ta1 {
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta1 {}

    impl TimerPeriph<DefaultMapping> for Ta2 {
        type Tbxclk = Pin<P3, Pin4, Alternate1<Input<Floating>>>;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.ta2rmp().clear_bit());
        }
    }
    impl TimerPeriph<RemappedMapping> for Ta2 {
        type Tbxclk = Pin<P5, Pin5, Alternate2<Input<Floating>>>;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.ta2rmp().set_bit());
        }
    }
    impl CapCmpTimer3<DefaultMapping> for Ta2 {}
    impl CapCmpTimer3<RemappedMapping> for Ta2 {}

    impl TimerPeriph<DefaultMapping> for Ta3 {
        type Tbxclk = Pin<P4, Pin2, Alternate1<Input<Floating>>>;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.ta3rmp().clear_bit());
        }
    }
    impl TimerPeriph<RemappedMapping> for Ta3 {
        type Tbxclk = Pin<P5, Pin4, Alternate2<Input<Floating>>>;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.ta3rmp().set_bit());
        }
    }
    impl CapCmpTimer3<DefaultMapping> for Ta3 {}
    impl CapCmpTimer3<RemappedMapping> for Ta3 {}

    impl TimerPeriph for Tb0 {
        type Tbxclk = Pin<P2, Pin7, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer7 for Tb0 {}
}
