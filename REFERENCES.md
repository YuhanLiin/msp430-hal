# Document references

The HAL configures the hardware as the TI documents below describe it. Comments in the code cite
these documents, so that each register setting, procedure, pin function and limit can be checked
against its source.

## Documents

| ID | Document | Revision | Used for |
|----|----------|----------|----------|
| SLAU445I | [MSP430FR4xx and MSP430FR2xx Family User's Guide](https://www.ti.com/lit/ug/slau445i/slau445i.pdf) | I, March 2019 | Peripherals, registers and procedures of every supported device |
| SLASEC4D | [MSP430FR235x, MSP430FR215x Mixed-Signal Microcontrollers](https://www.ti.com/lit/ds/slasec4d/slasec4d.pdf) (data sheet) | D, December 2019 | MSP430FR2x5x (`device_specific/fr2x5x.rs`): pin functions, signal connections, memory map, electrical limits |
| SLASE59F | [MSP430FR2433 Mixed-Signal Microcontroller](https://www.ti.com/lit/ds/slase59f/slase59f.pdf) (data sheet) | F, December 2019 | MSP430FR2433 (`device_specific/fr2433.rs`) |
| SLASEO7C | [MSP430FR247x Mixed-Signal Microcontrollers](https://www.ti.com/lit/ds/slaseo7c/slaseo7c.pdf) (data sheet) | C, September 2021 | MSP430FR247x (`device_specific/fr247x.rs`) |
| SLASEE4C | [MSP430FR25x2 Capacitive Touch Sensing Mixed-Signal Microcontrollers](https://www.ti.com/lit/ds/slasee4c/slasee4c.pdf) (data sheet) | C, December 2019 | MSP430FR25x2 (`device_specific/fr25x2.rs`) |
| SLAZ695J | [MSP430FR2355 Microcontroller](https://www.ti.com/lit/er/slaz695j/slaz695j.pdf) (errata) | J, September 2021 | Silicon errata of the MSP430FR2355, cited for the MSP430FR2x5x |
| SLAZ696H | [MSP430FR2353 Microcontroller](https://www.ti.com/lit/er/slaz696h/slaz696h.pdf) (errata) | H, August 2021 | Silicon errata of the MSP430FR2353 |
| SLAZ722H | [MSP430FR2155 Microcontroller](https://www.ti.com/lit/er/slaz722h/slaz722h.pdf) (errata) | H, August 2021 | Silicon errata of the MSP430FR2155 |
| SLAZ723H | [MSP430FR2153 Microcontroller](https://www.ti.com/lit/er/slaz723h/slaz723h.pdf) (errata) | H, August 2021 | Silicon errata of the MSP430FR2153 |
| SLAZ664S | [MSP430FR2433 Microcontroller](https://www.ti.com/lit/er/slaz664s/slaz664s.pdf) (errata) | S, May 2021 | Silicon errata of the MSP430FR2433 |
| SLAZ726B | [MSP430FR2476 Microcontroller](https://www.ti.com/lit/er/slaz726b/slaz726b.pdf) (errata) | B, August 2021 | Silicon errata of the MSP430FR2476, cited for the MSP430FR247x |
| SLAZ727B | [MSP430FR2475 Microcontroller](https://www.ti.com/lit/er/slaz727b/slaz727b.pdf) (errata) | B, August 2021 | Silicon errata of the MSP430FR2475 |
| SLAZ705H | [MSP430FR2522 Microcontroller](https://www.ti.com/lit/er/slaz705h/slaz705h.pdf) (errata) | H, May 2021 | Silicon errata of the MSP430FR2522, cited for the MSP430FR25x2 |
| SLAZ708H | [MSP430FR2512 Microcontroller](https://www.ti.com/lit/er/slaz708h/slaz708h.pdf) (errata) | H, May 2021 | Silicon errata of the MSP430FR2512 |
| SLAU680 | [MSP430FR2355 LaunchPad Development Kit (MSP-EXP430FR2355) User's Guide](https://www.ti.com/lit/ug/slau680/slau680.pdf) | May 2018 | Board wiring used by the MSP430FR2355 examples |
| SLAU739 | [MSP430FR2433 LaunchPad Development Kit (MSP-EXP430FR2433) User's Guide](https://www.ti.com/lit/ug/slau739/slau739.pdf) | October 2017 | Board wiring used by the MSP430FR2433 examples |
| SLAU802 | [MSP430FR2476 LaunchPad Development Kit (LP-MSP430FR2476) User's Guide](https://www.ti.com/lit/ug/slau802/slau802.pdf) | March 2019 | Board wiring used by the MSP430FR2476 examples |

There is no LaunchPad user's guide for the MSP430FR25x2, so its examples only cite the data sheet.

## How references are written

A reference names the document by its ID, then a section number, a table or a figure, then the page:

```rust
// SLAU445I 13.2.1, p. 371
// SLASEO7C Table 9-12, p. 55
// SLAZ726B USCI42, p. 8
```

- Page numbers are PDF page numbers, which in these documents are also the numbers printed on the
  pages.
- A short quote or title is added where it helps to find the exact statement.
- Section, table and page numbers refer to the revisions listed above. Later revisions of a
  document can renumber them, so check the revision before following a reference.
- Errata are cited by the errata sheet, the erratum name and the page, citing the first sheet of each
  device family, see [Errata](#errata).
- Facts that come from measurements on hardware rather than from a document say so, for example
  "measured on an MSP430FR2476".

## Errata

Every supported device has its own errata sheet. The sheets of one family list the same errata with the
same text on the same pages; only the device name and the revision history differ. So the code cites the
first sheet of each family:

| Family | Errata sheets | Cited |
|--------|---------------|-------|
| MSP430FR2x5x | SLAZ695J (MSP430FR2355), SLAZ696H (MSP430FR2353), SLAZ722H (MSP430FR2155), SLAZ723H (MSP430FR2153) | SLAZ695J |
| MSP430FR2433 | SLAZ664S | SLAZ664S |
| MSP430FR247x | SLAZ726B (MSP430FR2476), SLAZ727B (MSP430FR2475) | SLAZ726B |
| MSP430FR25x2 | SLAZ705H (MSP430FR2522), SLAZ708H (MSP430FR2512) | SLAZ705H |

The errata, with their pages in the cited sheet of each family (blank: the family's sheets don't list it),
and how the HAL deals with them. A feature `erratum_*` is enabled by the device features of the devices
that have the erratum, see `hal/Cargo.toml`.

| Erratum | MSP430FR2x5x | MSP430FR2433 | MSP430FR247x | MSP430FR25x2 | In the HAL |
|---------|--------------|--------------|--------------|--------------|------------|
| ADC50: temperature sensor results wrong with ACLK as the ADC clock in LPM3 | | p. 6 | | p. 5 | Documented at the `adc` module and `AdcConfig::use_aclk` |
| ADC63: ADCHI and ADCLO reset when ADCCTL2's high byte is written byte-wise | | p. 6 | | | Worked around: `AdcConfig::configure` writes ADCCTL2 as a word |
| BSL18: an empty reset vector doesn't invoke the BSL; revision A only (p. 2) | | p. 6 | | | Documented at `sys::Bsl` |
| COMP12: eCOMP0's output doesn't reach the TB0 CCI1B input | | | p. 5 | | Documented at the `ecomp` module and capture input B |
| CPU21, CPU22, CPU40: fixed by TI's compilers | p. 6 to p. 7 | p. 6 to p. 8 | p. 5 to p. 6 | p. 5 to p. 6 | Not in the code rustc generates, see below |
| CPU46: POPM up to the top of the stack can set VMAIFG | p. 7 to p. 8 | p. 8 to p. 9 | p. 6 to p. 7 | p. 6 to p. 7 | Documented at `sys::VacantMemory`; rustc doesn't use POPM, see below |
| CS13: LPM3/LPM4 entry can lock up with the DCO above 2 MHz | p. 8 to p. 9 | p. 9 to p. 10 | | p. 7 to p. 8 | `erratum_cs13`: the `lpm` functions bring the DCO to 2 MHz or lower for LPM3 and LPM4 |
| EEM23: debugger triggers with MPY, CRC and FRAM wait states; on the MSP430FR2x5x also consecutive software breakpoints | p. 9 | p. 10 | p. 7 to p. 8 | p. 8 | Debug only, see Debugging in README.md |
| GC4: PUC for bit errors that don't exist at 16 MHz with UBDRSTEN | | p. 10 | | | Documented at the `fram` module; `Fram::new` leaves UBDRSTEN at 0, the erratum's second workaround |
| GC5: FRAM bit errors reported after LPM1 to LPM4 that don't exist | | p. 10 to p. 11 | | | `erratum_gc5`: the `lpm` functions pause the bit error handling around LPM3 and LPM4, and turn it on again after reading five FRAM cache lines |
| PMM32: LPM3/LPM4 entry can lock up or run unintended code | p. 9 to p. 11 | p. 11 to p. 12 | | p. 8 to p. 10 | `erratum_pmm32`: the `lpm` functions switch the FRAM off from RAM before LPM3 and LPM4, with interrupts disabled until the sleep starts. A handler that returns to the sleep isn't covered: the `lpm` docs say to use `wake_cpu` |
| PORT28: clearing SYSRSTRE disables the pull-down of the TEST pin | | p. 12 to p. 13 | | | `erratum_port28`: `sys::RstPull` has no `None` |
| RTC15: the RTC hangs when moved off a stopped XT1CLK | p. 11 | p. 13 | | p. 10 | `erratum_rtc15`: the `rtc` module toggles XIN after moving off a stopped XT1CLK, if XT1OFFG comes back after it is cleared |
| TB25: in up mode, CLLD = 01b and 10b load at once | p. 11 | | p. 8 | | Worked around: Timer_B PWM uses CLLD = 11b; documented at `timer` and `pwm` |
| USCI42: UART UCTXCPTIFG after every byte | p. 12 | p. 13 | p. 8 | p. 10 | Worked around: `serial` waits for UCTXIFG and UCBUSY instead |
| USCI45: SPI clock stretching with an SCLK asynchronous to MCLK; on the MSP430FR2x5x with ACLK only | p. 12 | p. 13 to p. 14 | | | Documented at `SpiConfig::to_master_using_aclk` and `to_master_using_modclk` |
| USCI47: SPI slave with UCCKPH = 1 and SCLK not idle when it leaves reset | p. 12 to p. 13 | p. 14 | | p. 10 to p. 11 | Documented at `SpiConfig::to_slave`; `SpiSlave::reset` does the last workaround |
| USCI50: SPI 4-pin master with UCSTEM = 0, TXBUF written while STE is inactive | p. 13 | p. 14 to p. 15 | p. 8 to p. 9 | p. 11 | Worked around: the multi-master `spi` bus writes TXBUF only while STE is active |

No sheet lists preprogrammed software advisories (section 2, p. 2 of each sheet).

### Hardware revision

Section 5.3 of each sheet gives the hardware revision in the device descriptors (TLV) of each die
revision: 20h for revision B of the MSP430FR2x5x (SLAZ695J 5.3, p. 5), the only one its sheet lists; 11h
for revisions C and B and 10h for revision A of the MSP430FR2433 (SLAZ664S 5.3, p. 4 to p. 5). The sheets
of the MSP430FR247x and MSP430FR25x2 say "This device does not support reading the hardware revision from
memory" (SLAZ726B 5.3, p. 4; SLAZ705H 5.3, p. 4), so `tlv::hardware_revision()` exists only on the
MSP430FR2x5x and MSP430FR2433 (feature `tlv_hw_revision`). Each sheet's errata apply to every revision it
lists, except BSL18, which only revision A of the MSP430FR2433 has (SLAZ664S 1, p. 2).

### Fixed by compiler: CPU21, CPU22, CPU40

The sheets list these errata as fixed by the compilers (section 4, p. 2 of each sheet; the pages of each
erratum are in the table above): TI's MSP430 compiler with `--silicon_errata=CPU21`, `CPU22` and `CPU40`,
MSP430-GCC "4.9 build 167 or later" for CPU21 and CPU22, and IAR with `--hw_workaround=CPU40`; MSP430-GCC
is "Not affected" by CPU40, and IAR by CPU21 and CPU22. rustc and LLVM don't know these errata, so the
code they generate for the examples was checked instead, on 2026-10-05:

- Builds: the examples of the four example crates (MSP430FR2355, MSP430FR2433, MSP430FR2476 and
  MSP430FR2522), in the dev profile, as the CI builds them, with rustc 1.96.0-nightly (3b1b0ef4d
  2026-03-11, LLVM 22.1.0), 396 programs; in the release profile with the same compiler, 395 programs;
  and in the dev profile with the MSRV compiler, rustc 1.82.0-nightly (a7399ba69 2024-08-31, LLVM 19.1.0),
  the 368 that build with it. They were linked by msp430-elf-gcc 9.3.1.11 (Mitto Systems) with
  `-mcpu=msp430`, which links libgcc and libmul_f5 from its `430` multilib.
- Disassembly: msp430-elf-objdump 2.34, of every executable section of the linked programs, with the
  libgcc and libmul_f5 functions in them, and of `.data` where it holds the HAL's routine that runs from
  RAM for erratum PMM32.
- Checks: POPM (CPU21 if it restores SR, CPU46 for any POPM); the indirect mode with the PC as the source,
  `@PC` (CPU22; the immediate mode `@PC+` is fine); and the word after every JMP and conditional jump, which
  must not be "0X40h or 0X50h (where X = don't care)" (CPU40).
- Result: none of them in 1,562,212 instructions with 156,613 jumps. The programs contain no MSP430X
  instruction at all: rustc's code, the `asm!` blocks of the HAL and the examples, and libgcc's `430`
  multilib use the MSP430 instruction set only. POPM is one of the MSP430X extended instructions
  (SLAU445I 4.6.3.18, p. 241).

Code that is assembled by hand, or linked from libraries built for the MSP430X, needs the errata's own
workarounds.

### Debug only: EEM23

See Debugging in [README.md](README.md).
