// This file re-exports device-specific implementation details in a generic manner.
// The actual business logic for device-specific implementations are handled in the device_specific/ folder.

// MSP430FR2x5x series: MSP430FR2355, MSP430FR2353, MSP430FR2155, MSP430FR2153 (SLASEC4D Table 3-1, p. 8)
#[cfg(feature = "2x5x")]
#[path = "device_specific/fr2x5x.rs"]
pub mod device;

// MSP430FR2433 (SLASE59F Table 3-1, p. 7)
#[cfg(feature = "msp430fr2433")]
#[path = "device_specific/fr2433.rs"]
pub mod device;

// MSP430FR247x series: MSP430FR2476, MSP430FR2475 (SLASEO7C Table 6-1, p. 6)
#[cfg(feature = "247x")]
#[path = "device_specific/fr247x.rs"]
pub mod device;

// MSP430FR25x2: MSP430FR2522, MSP430FR2512 (SLASEE4C Table 3-1, p. 8)
#[cfg(feature = "25x2")]
#[path = "device_specific/fr25x2.rs"]
pub mod device;

pub use device::*;
