pub use msp430fr2433 as pac;

/// Re-exported PAC with standardised names. Despite having the same register contents, some peripheral names are different.
/// Rename them to match existing code.
pub mod _pac {
    pub use super::pac::*;
    pub use super::pac::{
        Crc16 as Crc, 
        BackupMemory as Bkmem, 
        Fram as Frctl, 
        RealTimeClock as Rtc, 
        real_time_clock as rtc, 
        watchdog_timer as wdt_a,
        WatchdogTimer as WdtA,
    };
}

pub mod gpio {
    // Re-export PAC GPIO peripherals
    pub use crate::_pac::{P1, P2, P3};
    use crate::hw_traits::gpio::gpio_impl;
    use crate::{adc, gpio::*};

    // Define alternate pin transitions
    // P1 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2
    impl<PULL> ToAlternate2 for Pin<P1, Pin0, Input<PULL>> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin2, DIR> {}
    impl       ToAlternate2 for Pin<P1, Pin3, Output> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin4, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin5, DIR> {}
    impl<PULL> ToAlternate2 for Pin<P1, Pin6, Input<PULL>> {}
    impl       ToAlternate2 for Pin<P1, Pin7, Output> {}

    // P1 ADCPCTLx. 'x' determined by ADC channel impl in adc module.
    impl<PIN: PinNum, DIR> ToAdcPctl for Pin<P1, PIN, DIR> where Self: adc::AdcPctlCapable {}

    // P2 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {}
    impl       ToAlternate1 for Pin<P2, Pin2, Output> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin4, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin5, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin6, DIR> {}

    // P3 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P3, Pin1, DIR> {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1);
}

/* ADC */
mod adc {
    use crate::{adc::*, gpio::*};

    impl_adc_channel_pin!(P1, Pin0, AdcMode => 0);
    impl_adc_channel_pin!(P1, Pin1, AdcMode => 1);
    impl_adc_channel_pin!(P1, Pin2, AdcMode => 2);
    impl_adc_channel_pin!(P1, Pin3, AdcMode => 3);
    impl_adc_channel_pin!(P1, Pin4, AdcMode => 4);
    impl_adc_channel_pin!(P1, Pin5, AdcMode => 5);
    impl_adc_channel_pin!(P1, Pin6, AdcMode => 6);
    impl_adc_channel_pin!(P1, Pin7, AdcMode => 7);
}

/* Backup Memory */
/// Size of the Backup Memory (BAKMEM register block) on this device, in bytes (SLASE59F 6.10.10, p. 52)
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::CapturePeriph, gpio::*, pac::*};

    impl CapturePeriph for Timer0A3 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin1, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin2, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Timer1A3 {
        type Gpio0 = ();
        type Gpio1 = Pin<P2, Pin5, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P2, Pin4, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
}

/* Clocks */
/// MODCLK frequency
pub const MODCLK_FREQ_HZ: u32 = 5_000_000;

/* eUSCI */
mod eusci {
    use crate::{
        hw_traits::{eusci::*, Steal},
        pac::*,
    };

    eusci_steal_impl!(UsciA0SpiMode);
    eusci_steal_impl!(UsciA0UartMode);

    eusci_steal_impl!(UsciA1SpiMode);
    eusci_steal_impl!(UsciA1UartMode);

    eusci_steal_impl!(UsciB0SpiMode);
    eusci_steal_impl!(UsciB0I2cMode);
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
        UsciB0I2cMode,
        ucb0ctlw0,
        ucb0ctlw1,
        ucb0brw,
        ucb0stat_i2c,
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
        ucb0ifg_i2c,
        ucb0iv,
        crate::pac::usci_b0_i2c_mode::ucb0ifg_i2c::R,
    );

    /// I2C SCL pin for eUSCI B0 (P1.3)
    pub struct UsciB0SCLPin;
    impl_i2c_pin!(UsciB0SCLPin, P1, Pin3);

    /// I2C SDA pin for eUSCI B0 (P1.2)
    pub struct UsciB0SDAPin;
    impl_i2c_pin!(UsciB0SDAPin, P1, Pin2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (P1.1)
    pub struct UsciB0UCLKIPin;
    impl_i2c_pin!(UsciB0UCLKIPin, P1, Pin1);

    impl I2cUsci for UsciB0I2cMode {
        type ClockPin = UsciB0SCLPin;
        type DataPin = UsciB0SDAPin;
        type ExternalClockPin = UsciB0UCLKIPin;
    }
}

/* Information Memory */
/// Size of the Information Memory segment on this device, in bytes (SLASE59F Table 6-23, p. 61)
pub const INFO_MEM_SIZE: usize = 512;

/* PWM */
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // TA0
    impl PwmPeriph<CCR1> for Timer0A3 {
        type Gpio = Pin<P1, Pin1, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Timer0A3 {
        type Gpio = Pin<P1, Pin2, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TA1
    impl PwmPeriph<CCR1> for Timer1A3 {
        type Gpio = Pin<P1, Pin5, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Timer1A3 {
        type Gpio = Pin<P1, Pin4, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TA2
    // None

    // TA3
    // None
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, serial::*};

    eusci_uart_impl!(
        UsciA0UartMode,
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
        crate::pac::usci_a0_uart_mode::uca0statw::R
    );

    eusci_uart_impl!(
        UsciA1UartMode,
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
        crate::pac::usci_a1_uart_mode::uca1statw::R
    );

    impl SerialUsci for UsciA0UartMode {
        type ClockPin = UsciA0ClockPin;
        type TxPin = UsciA0TxPin;
        type RxPin = UsciA0RxPin;
    }
    impl SerialUsci for UsciA1UartMode {
        type ClockPin = UsciA1ClockPin;
        type TxPin = UsciA1TxPin;
        type RxPin = UsciA1RxPin;
    }
    /// UCLK pin for E_USCI_A0 (P1.6)
    pub struct UsciA0ClockPin;
    impl_serial_pin!(UsciA0ClockPin, P1, Pin6);

    /// Tx pin for E_USCI_A0 (P1.4)
    pub struct UsciA0TxPin;
    impl_serial_pin!(UsciA0TxPin, P1, Pin4);

    /// Rx pin for E_USCI_A0 (P1.5)
    pub struct UsciA0RxPin;
    impl_serial_pin!(UsciA0RxPin, P1, Pin5);

    /// UCLK pin for E_USCI_A1 (P2.4)
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P2, Pin4);

    /// Tx pin for E_USCI_A1 (P2.6)
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P2, Pin6);

    /// Rx pin for E_USCI_A1 (P2.5)
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P2, Pin5);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, spi::*};

    eusci_spi_impl!(
        UsciA0SpiMode,
        uca0ctlw0_spi,
        uca0brw_spi,
        uca0statw_spi,
        uca0rxbuf_spi,
        uca0txbuf_spi,
        uca0ie_spi,
        uca0ifg_spi,
        uca0iv_spi,
        crate::pac::usci_a0_spi_mode::uca0statw_spi::R
    );
    eusci_spi_impl!(
        UsciA1SpiMode,
        uca1ctlw0_spi,
        uca1brw_spi,
        uca1statw_spi,
        uca1rxbuf_spi,
        uca1txbuf_spi,
        uca1ie_spi,
        uca1ifg_spi,
        uca1iv_spi,
        crate::pac::usci_a1_spi_mode::uca1statw_spi::R
    );
    eusci_spi_impl!(
        UsciB0SpiMode,
        ucb0ctlw0_spi,
        ucb0brw_spi,
        ucb0statw_spi,
        ucb0rxbuf_spi,
        ucb0txbuf_spi,
        ucb0ie_spi,
        ucb0ifg_spi,
        ucb0iv_spi,
        crate::pac::usci_b0_spi_mode::ucb0statw_spi::R
    );

    impl SpiUsci for UsciA0SpiMode {
        type MISO = UsciA0MISOPin;
        type MOSI = UsciA0MOSIPin;
        type SCLK = UsciA0SCLKPin;
        type STE = UsciA0STEPin;
    }

    impl SpiUsci for UsciA1SpiMode {
        type MISO = UsciA1MISOPin;
        type MOSI = UsciA1MOSIPin;
        type SCLK = UsciA1SCLKPin;
        type STE = UsciA1STEPin;
    }

    impl SpiUsci for UsciB0SpiMode {
        type MISO = UsciB0MISOPin;
        type MOSI = UsciB0MOSIPin;
        type SCLK = UsciB0SCLKPin;
        type STE = UsciB0STEPin;
    }

    /// SPI MISO pin for eUSCI A0 (P1.5)
    pub struct UsciA0MISOPin;
    impl_spi_pin!(UsciA0MISOPin, P1, Pin5);

    /// SPI MOSI pin for eUSCI A0 (P1.4)
    pub struct UsciA0MOSIPin;
    impl_spi_pin!(UsciA0MOSIPin, P1, Pin4);

    /// SPI SCLK pin for eUSCI A0 (P1.6)
    pub struct UsciA0SCLKPin;
    impl_spi_pin!(UsciA0SCLKPin, P1, Pin6);

    /// SPI STE pin for eUSCI A0 (P1.7)
    pub struct UsciA0STEPin;
    impl_spi_pin!(UsciA0STEPin, P1, Pin7);

    /// SPI MISO pin for eUSCI A1 (P2.5)
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P2, Pin5);

    /// SPI MOSI pin for eUSCI A1 (P2.6)
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P2, Pin6);

    /// SPI SCLK pin for eUSCI A1 (P2.4)
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P2, Pin4);
    /// SPI STE pin for eUSCI A1. This pin does not exist for the MSP430FR2433.
    pub struct UsciA1STEPin;

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
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_a::*, Steal},
        pac::{self, *},
        timer::*,
    };

    timer_a_impl!(
        Timer0A3,
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
        Timer1A3,
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
        Timer2A2,
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
        [CCR1, ta2cctl1, ta2ccr1]
    );

    timer_a_impl!(
        Timer3A2,
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
        [CCR1, ta3cctl1, ta3ccr1]
    );

    impl TimerPeriph for Timer0A3 {
        type Tbxclk = Pin<P1, Pin0, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Timer0A3 {}

    impl TimerPeriph for Timer1A3 {
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Timer1A3 {}
}
