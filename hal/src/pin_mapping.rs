//! Pin mapping strategies for peripherals.
//!
//! Many peripherals support multiple pin multiplexing configurations depending on the
//! device's port mapping or remapping capabilities. This module provides marker types
//! that describe which pin configuration a peripheral should use.
//!
//! The mapping type is used at compile time to select the correct implementation for a
//! peripheral. HAL drivers may use this information to configure device registers that
//! control pin routing or remapping.
//!
//! In device configuration crates, the mapping implementation typically performs the
//! required register operations to enable the selected pin layout before the peripheral
//! is initialized.
//!
//! Two common mapping strategies are provided:
//!
//! * `DefaultMapping` — Uses the primary pin layout defined by the device.
//! * `RemappedMapping` — Uses an alternate pin layout enabled through a remapping register.
//!
//! The remapping bits are in SYSCFG2 and SYSCFG3 (SLAU445I Table 1-31, p. 82; SLAU445I Table 1-32, p. 83):
//! * the MSP430FR247x moves eUSCI signals to other pins with USCIB0RMP in SYSCFG2 and USCIA0RMP and USCIB1RMP
//!   in SYSCFG3 (SLASEO7C Table 9-11, p. 54), and the TA2 and TA3 pins with TA2RMP and TA3RMP in SYSCFG3
//!   (SLASEO7C Table 9-16, p. 60);
//! * the MSP430FR25x2 moves eUSCI signals with USCIB0RMP in SYSCFG2 and USCIA0RMP in SYSCFG3 (SLASEE4C
//!   Table 6-11, p. 53), which SLASEE4C 6.10.7, p. 53 calls USCIBRMP and USCIARMP.
//!
//! Peripheral implementations select one of these mapping strategies when implementing
//! traits such as `SerialUsci<M>`, allowing the HAL to remain generic while supporting
//! multiple device pin configurations.

/// Trait for types that define a specific pin multiplexing strategy.
pub trait PinMap {}

/// Use the primary/default pin configuration for the peripheral (remapping bit 0, "Default function is
/// selected", SLAU445I Table 1-32, p. 83).
pub struct DefaultMapping;
impl PinMap for DefaultMapping {}

/// Use the alternate/secondary pin configuration for the peripheral (remapping bit 1, "Re-mapped function is
/// selected", SLAU445I Table 1-32, p. 83).
pub struct RemappedMapping;
impl PinMap for RemappedMapping {}

// Write a peripheral's remapping bit, `$field` in SYSCFG2 or SYSCFG3, for a pin mapping: clear it for
// `DefaultMapping` ("0b = Default function is selected"), set it for `RemappedMapping` ("1b = Re-mapped
// function is selected"; SLAU445I Table 1-31, p. 82; SLAU445I Table 1-32, p. 83). `clear_bits()` and
// `set_bits()` change only that bit, in one instruction, so the register's other bits keep their values.
// The device files' `configure_pin_mapping()` use it.
#[cfg(feature = "remap")]
macro_rules! write_remap_bit {
    ($reg:ident . $field:ident, DefaultMapping) => {{
        let sys = unsafe { $crate::_pac::Sys::steal() };
        unsafe { sys.$reg().clear_bits(|w| w.$field().clear_bit()) };
    }};
    ($reg:ident . $field:ident, RemappedMapping) => {{
        let sys = unsafe { $crate::_pac::Sys::steal() };
        unsafe { sys.$reg().set_bits(|w| w.$field().set_bit()) };
    }};
}
#[cfg(feature = "remap")]
pub(crate) use write_remap_bit;
