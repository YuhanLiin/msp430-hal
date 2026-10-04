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

    // Define alternate pin transitions. Alternate1, 2 and 3 set the PxSEL1/PxSEL0 bit pair, which the pin
    // function tables (SLASE59F Tables 6-17 to 6-20, p. 55 to p. 59) call PxSELx, to 01, 10 and 11: the
    // primary, secondary and tertiary module function (SLAU445I 8.2.5, Table 8-3, p. 314). A function that
    // needs a fixed PxDIR (a clock input or output) is only offered in that direction, because PxSELx
    // "does not automatically set the pin direction" (SLAU445I 8.2.5, p. 314).
    // P1 alternate 1, P1SELx = 01 with any P1DIR (SLASE59F Table 6-17, p. 55): P1.0 UCB0STE, P1.1 UCB0CLK,
    // P1.2 UCB0SIMO/UCB0SDA, P1.3 UCB0SOMI/UCB0SCL, P1.4 UCA0TXD/UCA0SIMO, P1.5 UCA0RXD/UCA0SOMI,
    // P1.6 UCA0CLK, P1.7 UCA0STE
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2, P1SELx = 10 (SLASE59F Table 6-17, p. 55). On the timer pins P1DIR picks the capture
    // input (0) or the compare output (1).
    impl<PULL> ToAlternate2 for Pin<P1, Pin0, Input<PULL>> {} // TA0CLK: P1SELx = 10, P1DIR = 0
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}         // TA0.CCI1A in / TA0.1 out: P1SELx = 10
    impl<DIR>  ToAlternate2 for Pin<P1, Pin2, DIR> {}         // TA0.CCI2A in / TA0.2 out: P1SELx = 10
    impl       ToAlternate2 for Pin<P1, Pin3, Output> {}      // MCLK: P1SELx = 10, P1DIR = 1
    impl<DIR>  ToAlternate2 for Pin<P1, Pin4, DIR> {}         // TA1.CCI2A in / TA1.2 out: P1SELx = 10
    impl<DIR>  ToAlternate2 for Pin<P1, Pin5, DIR> {}         // TA1.CCI1A in / TA1.1 out: P1SELx = 10
    impl<PULL> ToAlternate2 for Pin<P1, Pin6, Input<PULL>> {} // TA1CLK: P1SELx = 10, P1DIR = 0
    impl       ToAlternate2 for Pin<P1, Pin7, Output> {}      // SMCLK: P1SELx = 10, P1DIR = 1

    // P1 ADCPCTLx. 'x' determined by ADC channel impl in adc module. ADCPCTLx = 1 in SYSCFG2 selects
    // A0 to A7 on P1.0 to P1.7, whatever P1DIR and P1SELx are (SLASE59F Table 6-17, p. 55; SLAU445I
    // 1.12.2.3, p. 51; ADCPCTL0 to ADCPCTL7 in SYSCFG2: SLAU445I Table 1-31, p. 82).
    impl<PIN: PinNum, DIR> ToAdcPctl for Pin<P1, PIN, DIR> where Self: adc::AdcPctlCapable {}

    // P2 alternate 1, P2SELx = 01 with any P2DIR (SLASE59F Table 6-18, p. 56, for P2.0 to P2.2 and
    // SLASE59F Table 6-19, p. 58, for P2.3 to P2.7). P2.3 and P2.7 are GPIO only.
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {} // XOUT: P2SELx = 01
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {} // XIN: P2SELx = 01
    impl<DIR>  ToAlternate1 for Pin<P2, Pin4, DIR> {} // UCA1CLK: P2SELx = 01
    impl<DIR>  ToAlternate1 for Pin<P2, Pin5, DIR> {} // UCA1RXD/UCA1SOMI: P2SELx = 01
    impl<DIR>  ToAlternate1 for Pin<P2, Pin6, DIR> {} // UCA1TXD/UCA1SIMO: P2SELx = 01
    // P2 alternate 2, P2SELx = 10 (SLASE59F Table 6-18, p. 56)
    impl       ToAlternate2 for Pin<P2, Pin2, Output> {} // ACLK: P2SELx = 10, P2DIR = 1

    // P3 alternate 1, P3SELx = 01 with any P3DIR (SLASE59F Table 6-20, p. 59). P3.0 and P3.2 are GPIO only.
    impl<DIR>  ToAlternate1 for Pin<P3, Pin1, DIR> {} // UCA1STE: P3SELx = 01

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable. Only P1 and P2 have
    // interrupts (SLASE59F 6.10.3, p. 46; SLASE59F Tables 6-32 and 6-33, p. 64), with the vectors at FFDCh
    // and FFDAh (SLASE59F Table 6-2, p. 42). Registers: PxIN, PxOUT, PxDIR, PxREN, PxSEL0, PxSEL1, PxSELC,
    // PxIES, PxIE and PxIFG (SLAU445I Tables 8-9 to 8-18, p. 334 to p. 337), P1IV and P2IV (SLAU445I
    // Tables 8-5 and 8-6, p. 332).
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1);

    // Pins per port (SLASE59F 6.10.3, p. 46: "P1 and P2 are full 8-bit ports; P3 has 3 bits implemented").
    // The pins a port lacks are always the top ones. The DSBGA package also lacks P2.4 and P3.1 (SLASE59F
    // Table 4-1, p. 11).
    impl_port_pins!(P1, 8);
    impl_port_pins!(P2, 8);
    impl_port_pins!(P3, 3);
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
/// Size of the Backup Memory segment on this device, in bytes
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::{CapturePeriph, NoCapturePin}, gpio::*, pac::*};

    // Capture input A (CCIxA) of each capture/compare register, on its pin in the timer function: PxSELx = 10
    // with PxDIR = 0 (SLASE59F Table 6-11, p. 50, and SLASE59F Table 6-12, p. 51; SLASE59F Table 6-17,
    // p. 55). Both signal connection tables leave the device input of CCI0A empty, and "The CCR0 registers
    // on Timer0_A3 and Timer1_A3 are not externally connected" (SLASE59F 6.10.8, p. 50), so input A of
    // capture pin 0 is `NoCapturePin`, which can't be selected. Gpio3 to Gpio6 are unused: these timers
    // have CCR0 to CCR2 only (SLASE59F 6.10.8, p. 50). CCIS in TAxCCTLn selects 00b = CCIxA, 01b = CCIxB,
    // 10b = GND, 11b = VCC (SLAU445I Table 13-6, p. 386).
    impl CapturePeriph for Ta0 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = Pin<P1, Pin1, Alternate2<Input<Floating>>>; // TA0.CCI1A on P1.1: P1SELx = 10, P1DIR = 0
        type Gpio2 = Pin<P1, Pin2, Alternate2<Input<Floating>>>; // TA0.CCI2A on P1.2: P1SELx = 10, P1DIR = 0
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Ta1 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = Pin<P1, Pin5, Alternate2<Input<Floating>>>; // TA1.CCI1A on P1.5: P1SELx = 10, P1DIR = 0
        type Gpio2 = Pin<P1, Pin4, Alternate2<Input<Floating>>>; // TA1.CCI2A on P1.4: P1SELx = 10, P1DIR = 0
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    // TA2 and TA3 have no pins. Input B of TA3's capture pins 0 and 1 are the CCR0 and CCR1 outputs of
    // TA2 (SLASE59F Table 6-13, p. 51, and SLASE59F Table 6-14, p. 52, which call TA3 "Timer3_A3"), and both
    // timers can capture from software (SLAU445I 13.2.4.1.1, p. 376).
    impl CapturePeriph for Ta2 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = NoCapturePin;
        type Gpio2 = NoCapturePin;
        type Gpio3 = NoCapturePin;
        type Gpio4 = NoCapturePin;
        type Gpio5 = NoCapturePin;
        type Gpio6 = NoCapturePin;
    }

    impl CapturePeriph for Ta3 {
        type Gpio0 = NoCapturePin;
        type Gpio1 = NoCapturePin;
        type Gpio2 = NoCapturePin;
        type Gpio3 = NoCapturePin;
        type Gpio4 = NoCapturePin;
        type Gpio5 = NoCapturePin;
        type Gpio6 = NoCapturePin;
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

    // eUSCI_A0 and eUSCI_A1 (UART or SPI) and eUSCI_B0 (SPI or I2C) (SLASE59F 6.10.7, p. 49; SLASE59F
    // Table 6-24, p. 62)
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

    // eUSCI_B0 registers (SLASE59F Table 6-42, p. 67), in I2C mode: UCBxCTLW0 (SLAU445I Table 24-4,
    // p. 649), UCBxCTLW1 (SLAU445I Table 24-5, p. 651), UCBxBRW and UCBxSTATW (SLAU445I Tables 24-6 and
    // 24-7, p. 653), UCBxTBCNT (SLAU445I Table 24-8, p. 654), UCBxRXBUF and UCBxTXBUF (SLAU445I Tables
    // 24-9 and 24-10, p. 655), UCBxI2COA0 to UCBxI2COA3 (SLAU445I Tables 24-11 to 24-14, p. 656 to
    // p. 658), UCBxADDRX (SLAU445I Table 24-15, p. 658), UCBxADDMASK and UCBxI2CSA (SLAU445I Tables 24-16
    // and 24-17, p. 659), UCBxIE (SLAU445I Table 24-18, p. 660), UCBxIFG (SLAU445I Table 24-19, p. 662)
    // and UCBxIV (SLAU445I Table 24-20, p. 664)
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

    // The pins in their eUSCI_B0 function, P1SELx = 01 (SLASE59F Table 6-10, p. 49; SLASE59F Table 6-17,
    // p. 55)
    /// I2C SCL pin for eUSCI B0 (P1.3, UCB0SCL, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciB0SCLPin;
    impl_i2c_pin!(UsciB0SCLPin, P1, Pin3);

    /// I2C SDA pin for eUSCI B0 (P1.2, UCB0SDA, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciB0SDAPin;
    impl_i2c_pin!(UsciB0SDAPin, P1, Pin2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (P1.1, UCB0CLK, P1SELx = 01: SLASE59F
    /// Table 6-17, p. 55; UCSSELx = 00b selects it: SLASE59F Table 6-7, p. 46 and SLAU445I Table 24-4,
    /// p. 649)
    pub struct UsciB0UCLKIPin;
    impl_i2c_pin!(UsciB0UCLKIPin, P1, Pin1);

    impl I2cUsci for UsciB0I2cMode {
        type ClockPin = UsciB0SCLPin;
        type DataPin = UsciB0SDAPin;
        type ExternalClockPin = UsciB0UCLKIPin;
    }
}

/* Information Memory */
/// Size of the Information Memory segment on this device, in bytes
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

    // eUSCI_A0 and eUSCI_A1 registers (SLASE59F Table 6-40, p. 66, and SLASE59F Table 6-41, p. 67), in
    // UART mode: UCAxCTLW0 (SLAU445I Table 22-8, p. 593), UCAxCTLW1 (SLAU445I Table 22-9, p. 594), UCAxBRW
    // and UCAxMCTLW (SLAU445I Tables 22-10 and 22-11, p. 595), UCAxSTATW (SLAU445I Table 22-12, p. 596),
    // UCAxRXBUF and UCAxTXBUF (SLAU445I Tables 22-13 and 22-14, p. 597), UCAxABCTL (SLAU445I Table 22-15,
    // p. 598), UCAxIE (SLAU445I Table 22-17, p. 600), UCAxIFG
    // (SLAU445I Table 22-18, p. 601) and UCAxIV (SLAU445I Table 22-19, p. 602)
    eusci_uart_impl!(
        UsciA0UartMode,
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
        uca1abctl,
        uca1ie,
        uca1ifg,
        uca1iv,
        crate::pac::usci_a1_uart_mode::uca1statw::R
    );

    impl SerialUsci for UsciA0UartMode{
        type ClockPin = UsciA0ClockPin;
        type TxPin = UsciA0TxPin;
        type RxPin = UsciA0RxPin;
    }
    impl SerialUsci for UsciA1UartMode {
        type ClockPin = UsciA1ClockPin;
        type TxPin = UsciA1TxPin;
        type RxPin = UsciA1RxPin;
    }
    // The pins in their eUSCI_A function, PxSELx = 01 (SLASE59F Table 6-10, p. 49; SLASE59F Table 6-17,
    // p. 55, and SLASE59F Table 6-19, p. 58). UCLK is the external clock input that UCSSELx = 00b selects
    // (SLASE59F Table 6-7, p. 46; UCSSELx in UCAxCTLW0: SLAU445I Table 22-8, p. 593).
    /// UCLK pin for E_USCI_A0 (P1.6, UCA0CLK, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0ClockPin;
    impl_serial_pin!(UsciA0ClockPin, P1, Pin6);

    /// Tx pin for E_USCI_A0 (P1.4, UCA0TXD, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0TxPin;
    impl_serial_pin!(UsciA0TxPin, P1, Pin4);

    /// Rx pin for E_USCI_A0 (P1.5, UCA0RXD, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0RxPin;
    impl_serial_pin!(UsciA0RxPin, P1, Pin5);

    /// UCLK pin for E_USCI_A1 (P2.4, UCA1CLK, P2SELx = 01: SLASE59F Table 6-19, p. 58; not in the DSBGA
    /// package: SLASE59F Table 4-1, p. 11)
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P2, Pin4);

    /// Tx pin for E_USCI_A1 (P2.6, UCA1TXD, P2SELx = 01: SLASE59F Table 6-19, p. 58)
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P2, Pin6);

    /// Rx pin for E_USCI_A1 (P2.5, UCA1RXD, P2SELx = 01: SLASE59F Table 6-19, p. 58)
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P2, Pin5);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, spi::*};

    // eUSCI registers (SLASE59F Table 6-40, p. 66, and SLASE59F Tables 6-41 and 6-42, p. 67). In SPI mode,
    // eUSCI_A: UCAxCTLW0 (SLAU445I Table 23-3, p. 613), UCAxBRW (SLAU445I Table 23-4, p. 614), UCAxSTATW
    // (SLAU445I Table 23-5, p. 615), UCAxRXBUF and UCAxTXBUF (SLAU445I Tables 23-6 and 23-7, p. 616), UCAxIE
    // and UCAxIFG (SLAU445I Tables 23-8 and 23-9, p. 617), UCAxIV (SLAU445I Table 23-10, p. 618).
    // eUSCI_B: UCBxCTLW0 (SLAU445I Table 23-12, p. 620), UCBxBRW (SLAU445I Table 23-13, p. 621), UCBxSTATW
    // (SLAU445I Table 23-14, p. 622), UCBxRXBUF and UCBxTXBUF (SLAU445I Tables 23-15 and 23-16, p. 623),
    // UCBxIE and UCBxIFG (SLAU445I Tables 23-17 and 23-18, p. 624), UCBxIV (SLAU445I Table 23-19, p. 625).
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

    // The pins in their eUSCI function, PxSELx = 01 (SLASE59F Table 6-10, p. 49; SLASE59F Table 6-17,
    // p. 55, SLASE59F Table 6-19, p. 58, and SLASE59F Table 6-20, p. 59)
    /// SPI MISO pin for eUSCI A0 (P1.5, UCA0SOMI, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0MISOPin;
    impl_spi_pin!(UsciA0MISOPin, P1, Pin5);

    /// SPI MOSI pin for eUSCI A0 (P1.4, UCA0SIMO, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0MOSIPin;
    impl_spi_pin!(UsciA0MOSIPin, P1, Pin4);

    /// SPI SCLK pin for eUSCI A0 (P1.6, UCA0CLK, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0SCLKPin;
    impl_spi_pin!(UsciA0SCLKPin, P1, Pin6);

    /// SPI STE pin for eUSCI A0 (P1.7, UCA0STE, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciA0STEPin;
    impl_spi_pin!(UsciA0STEPin, P1, Pin7);

    /// SPI MISO pin for eUSCI A1 (P2.5, UCA1SOMI, P2SELx = 01: SLASE59F Table 6-19, p. 58)
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P2, Pin5);

    /// SPI MOSI pin for eUSCI A1 (P2.6, UCA1SIMO, P2SELx = 01: SLASE59F Table 6-19, p. 58)
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P2, Pin6);

    /// SPI SCLK pin for eUSCI A1 (P2.4, UCA1CLK, P2SELx = 01: SLASE59F Table 6-19, p. 58; not in the
    /// DSBGA package: SLASE59F Table 4-1, p. 11)
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P2, Pin4);
    /// SPI STE pin for eUSCI A1 (P3.1, UCA1STE, P3SELx = 01: SLASE59F Table 6-20, p. 59; not in the
    /// DSBGA package: SLASE59F Table 4-1, p. 11)
    pub struct UsciA1STEPin;
    impl_spi_pin!(UsciA1STEPin, P3, Pin1);

    /// SPI MISO pin for eUSCI B0 (P1.3, UCB0SOMI, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciB0MISOPin;
    impl_spi_pin!(UsciB0MISOPin, P1, Pin3);

    /// SPI MOSI pin for eUSCI B0 (P1.2, UCB0SIMO, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciB0MOSIPin;
    impl_spi_pin!(UsciB0MOSIPin, P1, Pin2);

    /// SPI SCLK pin for eUSCI B0 (P1.1, UCB0CLK, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciB0SCLKPin;
    impl_spi_pin!(UsciB0SCLKPin, P1, Pin1);

    /// SPI STE pin for eUSCI B0 (P1.0, UCB0STE, P1SELx = 01: SLASE59F Table 6-17, p. 55)
    pub struct UsciB0STEPin;
    impl_spi_pin!(UsciB0STEPin, P1, Pin0);
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_a::*, Steal},
        pac::*,
        timer::*,
    };

    // TA0 and TA1 have CCR0 to CCR2, TA2 and TA3 CCR0 and CCR1 (SLASE59F 6.10.8, p. 50 to p. 51; register
    // tables: SLASE59F Tables 6-35 to 6-38, p. 65). Registers: TAxCTL with TASSEL, TAIE, TAIFG and TACLR
    // (SLAU445I Table 13-4, p. 384), TAxR (SLAU445I Table 13-5, p. 385), TAxCCTLn (SLAU445I Table 13-6,
    // p. 386), TAxCCRn and TAxIV (SLAU445I Tables 13-7 and 13-8, p. 388), TAxEX0 with TAIDEX (SLAU445I
    // Table 13-9, p. 389).
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

    // The external clock inputs TAxCLK, P1SELx = 10 with P1DIR = 0 (SLASE59F Table 6-11, p. 50, and
    // SLASE59F Table 6-12, p. 51; SLASE59F Table 6-17, p. 55), which TASSEL = 00b selects (SLAU445I
    // Table 13-4, p. 384; SLASE59F Table 6-7, p. 46)
    impl TimerPeriph for Timer0A3 {
        type Tbxclk = Pin<P1, Pin0, Alternate2<Input<Floating>>>; // TA0CLK on P1.0: P1SELx = 10, P1DIR = 0
    }
    impl CapCmpTimer3 for Timer0A3 {}

    impl TimerPeriph for Timer1A3 {
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>; // TA1CLK on P1.6: P1SELx = 10, P1DIR = 0
    }
    impl CapCmpTimer3 for Timer1A3 {}

    // TA2 and TA3 aren't connected to any pins, so they have no clock pin, no PWM output and no capture
    // pins (SLASE59F 6.10.8, p. 51: "only internally connected and do not support PWM output"; SLASE59F
    // Table 6-13, p. 51, and SLASE59F Table 6-14, p. 52)
    impl TimerPeriph for Timer2A2 {
        type Tbxclk = NoTbxclkPin;
    }
    impl CapCmpTimer2 for Timer2A2 {}

    impl TimerPeriph for Timer3A2 {
        type Tbxclk = NoTbxclkPin;
    }
    impl CapCmpTimer2 for Timer3A2 {}

    // INCLK isn't connected on any timer, so there are no VLOCLK or cascaded timers (SLASE59F Tables
    // 6-11 to 6-14, p. 50 to p. 52, list no INCLK input. SLAU445I Figure 1-8, p. 50, still draws INCLK on
    // the TA0 and TA1 clock selects, TA0's "from CapTouchIO", which SLASE59F doesn't list.)
}
