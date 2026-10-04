//! Prelude

pub use crate::capture::CapturePeriph as _msp430fr2x5x_hal_CapturePeriph;
pub use crate::capture::CapturePin as _msp430fr2x5x_hal_CapturePin;
pub use crate::clock::Clock as _msp430fr2x5x_hal_Clock;
pub use crate::clock::SmclkState as _msp430fr2x5x_hal_SmclkState;
pub use crate::gpio::Alternate1 as _msp430fr2x5x_hal_Alternate1;
pub use crate::gpio::Alternate2 as _msp430fr2x5x_hal_Alternate2;
pub use crate::gpio::Alternate3 as _msp430fr2x5x_hal_Alternate3;
pub use crate::gpio::GpioFunction as _msp430fr2x5x_hal_GpioFunction;
pub use crate::gpio::PinNum as _msp430fr2x5x_hal_PinNum;
pub use crate::i2c::I2cRoleCommon as _msp430fr2x5x_hal_I2cRoleCommon;
pub use crate::i2c::I2cRoleMaster as _msp430fr2x5x_hal_I2cRoleMaster;
pub use crate::i2c::I2cRoleMulti as _msp430fr2x5x_hal_I2cRoleMulti;
pub use crate::i2c::I2cRoleSlave as _msp430fr2x5x_hal_I2cRoleSlave;
pub use crate::pwm::PwmPeriph as _msp430fr2x5x_hal_PwmPeriph;
pub use crate::rtc::RtcClockSrc as _msp430fr2x5x_hal_RtcClockSrc;
pub use crate::serial::SerialUsci as _msp430fr2x5x_hal_SerialUsci;
pub use crate::timer::CapCmp as _msp430fr2x5x_hal_CapCmp;
pub use crate::timer::CapCmpTimer2 as _msp430fr2x5x_hal_CapCmpTimer2;
pub use crate::timer::CapCmpTimer3 as _msp430fr2x5x_hal_CapCmpTimer3;
pub use crate::timer::CapCmpTimer7 as _msp430fr2x5x_hal_CapCmpTimer7;
pub use crate::timer::CascadedTimer as _msp430fr2x5x_hal_CascadedTimer;
pub use crate::timer::TimerPeriph as _msp430fr2x5x_hal_TimerPeriph;
pub use crate::timer::VloclkTimer as _msp430fr2x5x_hal_VloclkTimer;
pub use crate::watchdog::WatchdogSelect as _msp430fr2x5x_hal_WatchdogSelect;

#[cfg(feature = "embedded-hal-02")]
pub use embedded_hal_02::prelude;
