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
| SLAZ695J | [MSP430FR2355 Device Erratasheet](https://www.ti.com/lit/er/slaz695j/slaz695j.pdf) | J, September 2021 | Silicon errata of the MSP430FR2x5x |
| SLAZ664S | [MSP430FR2433 Device Erratasheet](https://www.ti.com/lit/er/slaz664s/slaz664s.pdf) | S, May 2021 | Silicon errata of the MSP430FR2433 |
| SLAZ726B | [MSP430FR2476 Device Erratasheet](https://www.ti.com/lit/er/slaz726b/slaz726b.pdf) | B, August 2021 | Silicon errata of the MSP430FR247x |
| SLAZ705H | [MSP430FR2522 Device Erratasheet](https://www.ti.com/lit/er/slaz705h/slaz705h.pdf) | H, May 2021 | Silicon errata of the MSP430FR25x2 |
| SLAU680 | [MSP430FR2355 LaunchPad Development Kit (MSP-EXP430FR2355) User's Guide](https://www.ti.com/lit/ug/slau680/slau680.pdf) | May 2018 | Board wiring used by the MSP430FR2355 examples |
| SLAU739 | [MSP430FR2433 LaunchPad Development Kit (MSP-EXP430FR2433) User's Guide](https://www.ti.com/lit/ug/slau739/slau739.pdf) | October 2017 | Board wiring used by the MSP430FR2433 examples |
| SLAU802 | [MSP430FR2476 LaunchPad Development Kit (LP-MSP430FR2476) User's Guide](https://www.ti.com/lit/ug/slau802/slau802.pdf) | March 2019 | Board wiring used by the MSP430FR2476 examples |

There is no LaunchPad user's guide for the MSP430FR25x2, so its examples only cite the data sheet.

## How references are written

A reference names the document by its ID, then a section number, a table or a figure, then the page:

```rust
// SLAU445I 13.2.1, p. 371
// SLASEO7C Table 9-12, p. 55
// SLAZ726B USCI42
```

- Page numbers are PDF page numbers, which in these documents are also the numbers printed on the
  pages.
- A short quote or title is added where it helps to find the exact statement.
- Section, table and page numbers refer to the revisions listed above. Later revisions of a
  document can renumber them, so check the revision before following a reference.
- Errata are cited by the errata sheet and the erratum name.
- Facts that come from measurements on hardware rather than from a document say so, for example
  "measured on an MSP430FR2476".
