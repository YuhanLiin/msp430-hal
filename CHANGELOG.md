# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

- The example projects now link `libmul_f5`, so multiplication uses the hardware multiplier (MPY32): 16-bit products are about 5 times and 32/64-bit products about 10 times faster than with `libmul_none`. The commented-out `libmul_32` used the register addresses of other devices, which are PM5CTL0 here.
- Document the CRC module with references to the user's guide.
- Fixed `delay_ns()` and `delay_us()`, which waited whole milliseconds, at most 1 ms. They now wait the requested time, counted in MCLK cycles, plus about 60 MCLK cycles for the call. `SysDelay` also implements the embedded-hal 0.2 `DelayUs` now.
- Add `delay::delay_cycles()`, which busy-waits for at least a number of MCLK cycles, like TI's `__delay_cycles()`. It needs no configured clock and works in steps of 3 cycles: the count is rounded up to a multiple of 3, so a count above 0 waits at least 3 cycles.
- Document the backup memory, its LPM3.5 retention and its registers, with references to the user's guide.
- Fixed `InfoMemory::write()` and `into_unprotected()` switching off the program FRAM write protection (PFWP) until the next reset. `write()` now runs with interrupts disabled, as the user's guide recommends.

## [v0.8.0] - 2026-08-14
- Changed name of project from `msp430fr2x5c-hal` to `msp430-hal` to better represent the scope of the project.
  - On the old `msp430fr2x5c-hal` crate, this added a build error telling users to switch to the new `msp430-hal` crate.
  - On the new crate `msp430-hal`, 0.8.0 is a no-change release from 0.7.0, just to keep version numbers in line.

## [v0.7.0] - 2026-08-11
- Add support for the MSP430FR25x2 subfamily.
- Add support for the MSP430FR247x subfamily.
- Add support for pin remapping on certain peripherals on certain devices (like the FR247x subfamily). For devices that don't support pin remapping this should be invisible for the most part.
- Add defmt support through `defmt` feature flag.
- Optimised GPIO toggling implementation
- Fixed a bug around enabling/disabling the PWM on the FR2433 Timer1 peripheral.

## [v0.6.1] - 2026-03-11
- Fix missing readme on crates.io

## [v0.6.0] - 2026-03-11
- The library now requires specifying a device feature (e.g. `msp430fr2355`) as part of the process to support more devices. Users must enable exactly one device feature that matches the device being targetted.
- The underlying MSP430FR2355 PAC version has been bumped to v0.6, which now uses svd2rust 0.37.1. This has changed the capitalisation of peripheral instances from `SCREAMING_CASE` to `snake_case`, e.g.`periph.WDT_A` is now `periph.wdt_a`. Peripheral type names have also changed case, and underscores have been omitted, e.g. `E_USCI_A0` is now `EUsciA0`. Both the HAL and PAC should be updated together to v0.6.
- Add initial support for the MSP430FR2433
- The `REFOCLK` and `VLOCLK` constants have been renamed to the more descriptive `REFOCLK_FREQ_HZ` and `VLOCLK_FREQ_HZ`.
- The frequency of MODCLK is now exported through the `MODCLK_FREQ_HZ` constant.
- Batch GPIO configuration now supports configuring pins to alternate modes
- Added Batch GPIO methods to set all pins in a port as inputs with either pullups or pulldowns. Useful for minimising power usage on unused pins, as leaving them in the default floating state can waste a lot of power through noise-induced schmitt trigger activations.
- The `pac::Sys` register block is now consumed by `Pmm::new()` and additionally returns an instance of `InfoMemory`. The `SYSCFGx` registers are touched internally by several peripherals in the HAL, but there was previously no mechanism to prevent users from accidentally modifying these registers afterwards. An instance of `pac::Sys` can be constructed unsafely using `pac::Sys::steal()` if it is required, but it is up to the user to ensure that control bits used by the HAL are not modified.
- Refactors to the Information Memory interface:
  - As above, rather than `InfoMemory::as_x()` methods consuming the `pac::Sys` register block, the `Pmm::new()` method returns an instance of `InfoMemory`.
  - `InfoMemory` now offers two modes of operation: The information memory is normally write protected, but `.write()` takes a closure where the info memory's write protection is temporarily disabled. Alternatively, `.into_unprotected()` disables the write protection entirely and returns a mutable reference to the underlying array.
- Fixed linker error in release mode in some examples: "undefined reference to `__mspabi_func_epilog`" by linking lgcc last in `.cargo/config.toml`. If you encounter this error in your project you can make the same fix.


## [v0.5.0] - 2025-10-30

### Additions
- Add support for low power modes
- Add SPI slave support. This includes modifying SPI configuration flow and `SpiErr`.
- Add support for I2C multi-master, slave, and master-slave roles.
- Add support for Smart Analog Combo and Enhanced Comparator modules (MSP430FR23xx only)
- Add support for reading from / writing to backup memory and information memory
- Add support for hardware CRC module
- Add implementations for embedded-hal 1.0 traits (including embedded-io and embedded-hal-nb).
- Support for delays when using sub-1MHz clock sources (e.g. ACLK, VLOCLK).
- Add methods to enable the internal voltage reference and temperature sensor.
- Add additional generic delay implementations for eh-0.2.7 DelayMs trait (u8, u32, i32).
- Derive `Debug` and `Copy` for `RecvError`.

### Changes
- MSRV updated to nightly-2023-09-01 (1.82) due to `msp430-rt` 0.4.1.
- Gate embedded-hal 0.2.7 implementations behind `embedded-hal-02` feature.
- Expose functionality from the traits dropped between eh-0.2.7 and eh-1.0 (ADC, timers, RTC, watchdog, etc.) as methods on structs instead.
- The SPI struct has been renamed from `SpiBus` to `Spi` to avoid naming conflicts with the new embedded-hal 1.0 trait `SpiBus`.
- Ensure crate builds successfully back to `nightly-2023-09-01`.
- Replace public references to `void::Void` with `core::convert::Infallible`.

### Bugfixes
- Fix SPI flushing bug.
- Fix GPIO pins labelled with incorrect SPI functionality for eUSCI A0 and A1.
- Fixed a bug that would sometimes cause an infinite loop during clock configuration (same bug mentioned in v0.4.0).
- Bring I2C implementation inline with the contracts mentioned in embedded-hal.

## [v0.4.1] - 2025-01-25

- Fix doc.rs build issue

## [v0.4.0] - 2025-01-22

- Add support for ADC interface
- Add support for SPI interface
- Add support for I2C interface
- Add `Delay` object, allowing for millisecond delays
- Change `ClockConfig::freeze` to return `Delay` in addition to its other return values
- Mitigate issue of device hanging after clock configuration by adding NOPs

## [v0.3.3] - 2022-12-24

- Bump `msp430fr2355` to v0.5.2 to ensure atomic PAC operations are single-instruction

## [v0.3.2] - 2022-10-26

- Bump `msp430fr2355` to v0.5.1
- Fix documentation generation on doc.rs

## [v0.3.1] - 2022-10-26

- Remove erroneous line in docs

## [v0.3.0] - 2022-10-26

- Bump `msp430fr2355` to v0.5
- Add CI pipeline
- Update dependencies to latest ecosystem versions
