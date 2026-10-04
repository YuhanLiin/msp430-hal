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
//! The remapping bits are in SYSCFG2 and SYSCFG3 (SLAU445I Table 1-31, p. 82; SLAU445I Table 1-32, p. 83).
//! The MSP430FR247x and MSP430FR25x2 use them to move eUSCI signals to other pins (SLASEO7C Table 9-11,
//! p. 54; SLASEE4C Table 6-11, p. 53): USCIB0RMP in SYSCFG2, USCIA0RMP and USCIB1RMP in SYSCFG3 on the
//! MSP430FR247x (SLAU445I Table 1-31, p. 82; SLAU445I Table 1-32, p. 83), USCIBRMP in SYSCFG2 and USCIARMP in
//! SYSCFG3 on the MSP430FR25x2 (SLASEE4C 6.10.7, p. 53).
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
