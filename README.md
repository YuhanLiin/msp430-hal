# `msp430-hal`

---

*This project was previously called `msp430fr2x5x-hal`, recently renamed to better represent the scope of the project.*

---

> A high-level Hardware Abstraction Layer (HAL) for the MSP430 family of microcontrollers, principally targetting the FR2xxx / 4xxx family, but seeking to support the all MSP430 devices eventually.

[![Crates.io](https://img.shields.io/crates/v/msp430fr2x5x-hal.svg)](https://crates.io/crates/msp430fr2x5x-hal)
[![Docs.rs](https://docs.rs/msp430fr2x5x-hal/badge.svg)](https://docs.rs/msp430fr2x5x-hal)
[![CI](https://github.com/YuhanLiin/msp430fr2x5x-hal/actions/workflows/build.yml/badge.svg)](https://github.com/YuhanLiin/msp430fr2x5x-hal/actions)
[![License](https://img.shields.io/crates/l/msp430fr2x5x-hal.svg)](https://crates.io/crates/msp430fr2x5x-hal)
[![MSRV](https://img.shields.io/badge/rust-1.82%2B-blue.svg)](https://www.rust-lang.org)

This crate is primarily designed to be used as a dependency in another project, but 
the examples in the repo can be build and flashed directly to a device, provided the 
required dependencies are installed.

# Dependencies

To build the examples [`msp430-gcc`](https://www.ti.com/tool/MSP430-GCC-OPENSOURCE) 
should be available on your PATH.

To flash an example [`mspdebug`](https://dlbeer.co.nz/mspdebug/) should also be 
available on your PATH.

# Usage

The `device-examples/` folder contains typical projects for various supported devices, each of which containing a number of examples
They can be built by moving into the relevant project folder, then running `cargo build --example <example_name>`

An example can be flashed to a connected device with 
`cargo run --example <example_name>`

## Hardware multiplier

Every supported device has the 32-bit hardware multiplier (MPY32). For multiplications to use it, link TI's
`libmul_f5` before `libgcc`, as the example projects do in `.cargo/config.toml`:

```toml
rustflags = [
    # ...
    "-C", "link-arg=-lmul_f5",
    "-C", "link-arg=-lgcc",
]
```

With `libmul_none` multiplications run in software, about 5 to 10 times slower. Don't use `libmul_32`: it
expects the multiplier at another address, which is PM5CTL0 on these devices.

## Debugging

Erratum EEM23 affects debugging on every supported device: "When modules using wait states (USB, MPY,
CRC and FRAM controller in manual mode) are enabled, the EEM may trigger incorrectly. This can lead to
an incorrect profile counter value or cause issues with the EEMs data watch point, state storage, and
breakpoint functionality." There is no workaround, and only debugging is affected. Programs
built like the examples use the multiplier (MPY32, through `libmul_f5`), the `crc` module uses the CRC,
and the HAL sets FRAM wait states (NWAITS, SLAU445I 6.5, p. 302) for an MCLK above 8 MHz. On the
MSP430FR2x5x, also "When more than 2 software breakpoints are set in continuous instruction addresses ...
program counter (PC) will stuck at the first breakpoint and can't move on". (SLAZ695J EEM23, p. 9;
SLAZ664S EEM23, p. 10; SLAZ726B EEM23, p. 7 to p. 8; SLAZ705H EEM23, p. 8)

# Supported Devices
The library currently supports a subset of the MSP430FR2xxx / 4xxx family: the MSP430FR2x5x and MSP430FR247x and MSP430FR25x2 subfamilies, and the MSP430FR2433.
Adding support for a device in the MSP430FR2xxx/4xxx family is easy, see [Supporting additional devices](#Supporting-additional-devices).

The device being targetted is must be specified by enabling exactly one device feature, such as 
`msp430fr2355`. This is required to build any code from this library.

### Currently Supported Devices
| Device       | Feature name   |
| ------------ | -------------- |
| MSP430FR2476 | `msp430fr2476` |
| MSP430FR2475 | `msp430fr2475` |
| MSP430FR2433 | `msp430fr2433` |
| MSP430FR2355 | `msp430fr2355` |
| MSP430FR2353 | `msp430fr2353` |
| MSP430FR2155 | `msp430fr2155` |
| MSP430FR2153 | `msp430fr2153` |
| MSP430FR2512 | `msp430fr2512` |
| MSP430FR2522 | `msp430fr2522` |

The documentation on crates.rs (and example programs) target the MSP430FR2355. Documentation for a particular device can be 
built by running `cargo doc --open --features <device>` from within the `hal/` folder, or `cargo doc --open --package msp430fr2x5x-hal` in a 
cargo project with `msp430fr2x5x-hal` correctly configured as a dependency, such as the projects in the `device-examples/` folder.

# Documents

The code follows TI's family user's guide, the device data sheets and errata sheets, and the
LaunchPad user's guides. Comments cite them by section, table and page, so each register setting
and pin function can be checked against its source. [REFERENCES.md](REFERENCES.md) lists the
documents, their revisions and the reference format.

# Functionality
The library is mostly feature complete for the FR2xxx/4xxx family. The following FR2xxx/4xxx peripherals do not yet have drivers:
- LCD driver
- CapTIvate
- TIA
- SAC-L1

PRs with implementations for these peripherals are welcome.

If you encounter any use cases not supported please open an issue (or submit a pull request).

## Supporting additional devices

The maintainers don't have access to every device in the MSP430FR2xxx / 4xxx family, so if you want to add support for a particular device (or subfamily) we are happy to accept pull requests.

This repo contains two main parts - drivers for peripherals (shared across many devices), and pin mappings for a specific device.
Many peripheral drivers have already been written, so adding support for a new device is usually just a matter of defining what peripherals a device has, and how they're connected to it's GPIO pins.
The following section describes how to do this:

To add support for a device (or subfamily) you should fork this repo and:
1. Add a new device feature in `hal/cargo.toml` and determine which features should be derived for your device.
2. Create a new file in `hal/src/device_specific/`, import a Peripheral Access Crate (PAC) for your device, and follow `hal/src/device_specific/msp430fr2x5x.rs` as an example to see how to e.g. mark which GPIO pins are capable of what functionality.
    * (Note in some cases the PAC may be missing some register fields, as TI's .svd files aren't perfect. In this case the PAC should be modified if these missing register fields are used by the HAL).
3. Append an entry to `hal/src/device_specific.rs` to re-export your PAC and any device-specific constants to the rest of the library.
4. Add a project crate to `device_examples/` and add some examples to test everything works (again refer to the msp430fr2355 as an example/template). 
    * Ensure the `memory.x` file is correct, as this is usually unique to each device.
    * Check that the device's hardware multiplier suits `libmul_f5` in `.cargo/config.toml`: an MPY32 at 04C0h, `MPY_TYPE` 8 in msp430-gcc's `include/devices.csv`.
5. Add the device name to the CI in `.github/workflows/build.yml` to make it automatically build all your device examples. Check that the CI passes.

For issues or concerns, feel free to open an issue.

### Devices outside the FR2xxx/4xxx family

If your device exists outside the FR2xxx/4xxx family then the above still holds, but additionally it is up to you to ensure the drivers written here are compatible with your device's peripherals 
(e.g. by comparing the FR2xxx/4xxx user guide against your user guide).
The drivers provided here have been written by referencing the 2xxx / 4xxx user guide, so if they support for other devices out-of-the-box this is incidental,
however they may work regardless, require only minor modifications, or may at least serve as a reasonable starting point.

If you do find that a peripheral driver requires changes, we are happy to accept new drivers for devices outside the 2xxx / 4xxx family.

# Feature Flags

In addition to the device feature flags mentioned above, this crate provides an implementation of the legacy 0.2.7 version of embedded-hal behind the `embedded-hal-02` feature. Support for embedded-hal 1.0 is available by default.

Support for `defmt` is available through the `defmt` feature. See `device-examples/msp430fr2355/examples/defmt.rs` for a defmt implementation on the MSP430.

# Minimum Supported Rust Version (MSRV)

This crate requires the `nightly` toolchain to compile, currently targetting `nightly-2024-09-01` 
(Rust 1.82) or later. It might compile with older versions but that may change in any new patch release.

# Assumptions

The HAL provides a maximal set of GPIO pins (targetting the package with the most pins). If you 
are using one of the variants with fewer pins then it is up to you to ensure that the GPIO pins 
you use are in fact available.
For example, on the 28-pin variant of the MSP430FR2355 eUSCI_B1 only supports I2C due to missing pins. 
This is not checked in the HAL. 

# Panics

Some configuration functions panic on arguments the hardware doesn't support, as their documentation describes
under "Panics": XT1 and DCO frequencies outside the supported ranges, a UART baud rate above a third of its clock,
I2C clock divisors below the user's guide minimum, and more writable program FRAM than the device has. With
constant, valid arguments these checks are usually optimised away.

Apart from these, the library is intended to be panic-free, though this hasn't been verified. If you encounter panics
while using the library (or `panic-never` points to the existence of possible panics) please open an issue.

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
