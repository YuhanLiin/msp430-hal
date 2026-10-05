//! A collection of peripheral drivers for the MSP430 family of microcontrollers (primarily the [MSP430FR2xxx/4xxx](http://www.ti.com/lit/ug/slau445i/slau445i.pdf)) with implementations of [`embedded_hal`] traits.
//!
//! The peripherals are described in the family user's guide, SLAU445I, and in the device data sheets
//! (SLASEC4D, SLASE59F, SLASEO7C and SLASEE4C); REFERENCES.md in the repository lists these documents and
//! the reference format used in the comments.
//!
//! The documentation on docs.rs is built for the MSP430FR2355. To build the documentation for your device
//! add this crate as a dependency to your project (see: [Feature Flags](#feature-flags)) then run `cargo doc --open --package msp430-hal`.
//! The github repository contains such projects for various devices under `device_examples/`.
//!
//! [`embedded_hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! # Usage
//!
//! Requires `msp430-elf-gcc` installed and in $PATH to build
//!
//! When using this crate as a dependency, make sure you include the appropriate `memory.x` file for
//! your microcontroller.
//!
//! # Examples
//!
//! The `device-examples/` directory in the repository contains projects for various supported devices, each containing a typical
//! project structure and a number of examples that show how to use the HAL abstractions. These examples typically target the relevant dev board,
//! such as the MSP-EXP430FR2355 for the MSP430FR2355 (SLAU680, p. 1).
//!
//! To flash the examples, make sure you have `mspdebug` with `tilib` support installed and in
//! $PATH. Invoke `cargo run --example whatever` from within the relevant project folder with the board plugged and the scripts should do
//! the trick, assuming your host is Linux and you are connected via Launchpad.
//!
//! # Feature Flags
//!
//! Exactly one device feature must be enabled to specify which microcontroller is present.
//! More info can be found in the repository README.
//!
//! An implementation of the pre-1.0 version of embedded-hal (e.g. 0.2.7 at time of writing) is
//! available behind the `embedded-hal-02` feature flag. These traits are implemented on the same
//! structs as the current embedded-hal implementation, so with this feature enabled you may mix and
//! match crates that require the pre-1.0 version with those that require the latest version. It isn't enabled by
//! default, as many of the trait names are similar (or identical) to their counterparts in the current
//! version, which can be confusing.
//!
//! Support for defmt is available behind the `defmt` feature.

#![no_std]
#![allow(incomplete_features)] // Enable specialization without warnings
#![feature(specialization)]
#![feature(asm_experimental_arch)]
#![feature(abi_msp430_interrupt)]
#![allow(stable_features)] // Feature flags used on older compiler versions
#![feature(const_option)]
#![feature(const_refs_to_cell)] // Register addresses from the PAC at compile time (hw_traits/gpio.rs)
#![deny(missing_docs)]

// The hardware modules and where the documents describe them. A module behind a feature exists only on
// some of the devices.
// Backup memory (BAKMEM): SLAU445I chapter 7, p. 309
pub mod bak_mem;
// Digital I/O: SLAU445I chapter 8, p. 311
pub mod batch_gpio;
// Timer_A and Timer_B capture: SLAU445I chapter 13, p. 367 and SLAU445I chapter 14, p. 390
pub mod capture;
// Clock System (CS): SLAU445I chapter 3, p. 98
pub mod clock;
// CRC module: SLAU445I chapter 11, p. 352
pub mod crc;
// Software delay counted in MCLK cycles of the CPU instructions: SLAU445I 4.5.1.5, p. 154
pub mod delay;
// FRAM controller (FRCTL): SLAU445I chapter 6, p. 300
pub mod fram;
// Digital I/O: SLAU445I chapter 8, p. 311
pub mod gpio;
// Interrupt Compare Controller (ICC): SLAU445I chapter 5, p. 280. Only the MSP430FR2x5x has one (SLASEC4D
// 1.1, p. 1: "Interrupt compare controller (ICC)").
#[cfg(feature = "icc")]
pub mod icc;
// Infrared modulation in the SYS module: SLAU445I 1.12.2.2, p. 50
pub mod ir;
// Operating (low-power) modes: SLAU445I 1.4, p. 36
pub mod lpm;
// Manchester Function Module (MFM): SLAU445I chapter 25, p. 665. Only the MSP430FR2x5x has one (SLASEC4D
// 1.1, p. 1: "Manchester codec (MFM)").
#[cfg(feature = "mfm")]
pub mod mfm;
// Pin remapping bits in SYSCFG2 and SYSCFG3: SLAU445I Table 1-31, p. 82 and SLAU445I Table 1-32, p. 83
pub mod pin_mapping;
// Power Management Module (PMM) and SVS: SLAU445I chapter 2, p. 84
pub mod pmm;
pub mod prelude;
// Timer_A and Timer_B output modes (PWM): SLAU445I chapter 13, p. 367 and SLAU445I chapter 14, p. 390
pub mod pwm;
// Real-Time Clock (RTC) counter: SLAU445I chapter 15, p. 415
pub mod rtc;
// eUSCI_A in UART mode: SLAU445I chapter 22, p. 574
pub mod serial;
// eUSCI_A and eUSCI_B in SPI mode: SLAU445I chapter 23, p. 603
pub mod spi;
// System Control Module (SYS): SLAU445I chapter 1, p. 29
pub mod sys;
// Timer_A and Timer_B: SLAU445I chapter 13, p. 367 and SLAU445I chapter 14, p. 390
pub mod timer;
// Device Descriptor Table (TLV): SLAU445I 1.13, p. 57
pub mod tlv;
// Watchdog Timer (WDT_A): SLAU445I chapter 12, p. 360
pub mod watchdog;

// ADC: SLAU445I chapter 21, p. 538
#[cfg(feature = "adc")]
pub mod adc;

// Enhanced Comparator (eCOMP): SLAU445I chapter 18, p. 503. The MSP430FR2x5x has two, the MSP430FR247x
// one (SLASEC4D 1.1, p. 1; SLASEO7C section 1, p. 1).
#[cfg(feature = "ecomp")]
pub mod ecomp;

// eUSCI_B in I2C mode: SLAU445I chapter 24, p. 626
#[cfg(feature = "eusci_b")]
pub mod i2c;

// Information memory, 1800h to 19FFh: SLAU445I 1.9.1, p. 44
#[cfg(feature = "info_mem")]
pub mod info_mem;

// Smart Analog Combo (SAC): SLAU445I chapter 20, p. 518. Only the MSP430FR235x has SAC-L3 (SLASEC4D 1.1,
// p. 1: "MSP430FR235x devices only").
#[cfg(feature = "sac")]
#[path = "sac_l3.rs"]
pub mod sac;

#[cfg(feature = "sac_l1")]
#[path = "sac_l1.rs"]
pub mod sac;

mod device_specific;
mod hw_traits;
mod util;

pub use device_specific::pac;

/// PAC with standardised peripheral names. Every device's PAC uses the same names, so this is just `pac`.
pub(crate) use device_specific::_pac;

#[cfg(feature = "embedded-hal-02")]
pub use embedded_hal_02 as ehal_02;

pub use embedded_hal as ehal;
