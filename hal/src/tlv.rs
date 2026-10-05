//! The device descriptor table (TLV): which device this is, and calibration values measured during
//! production
//!
//! Each device describes itself in its device descriptors, from 1A00h on (data sheets: Device
//! Descriptors, SLASEC4D Table 6-70, p. 107 to p. 108; SLASEO7C Table 9-30, p. 71 to p. 72; SLASE59F
//! Table 6-22, p. 60 to p. 61; SLASEE4C Table 6-18, p. 61 to p. 62; the structure: SLAU445I 1.13, p. 57
//! to p. 58; how to use the calibration values: SLAU445I 1.13.3, p. 59 to p. 60). In table order:
//!
//! - The information block: [`device_id()`], `hardware_revision()` (not on the MSP430FR247x and
//!   MSP430FR25x2), [`firmware_revision()`], and the CRC that [`crc_matches()`] checks the table against.
//! - The die record: [`die_record()`].
//! - The ADC calibration: [`adc_gain_factor()`], [`adc_offset()`] and [`TempSensorCalibration`]. That
//!   converts temperature sensor readings much more accurately than the typical sensor voltage and slope
//!   from the data sheet (SLAU445I 21.2.7.8, p. 556: "The temperature sensor offset error can be large
//!   and must be calibrated for most applications"; SLASE59F Table 5-22 note 2, p. 36, and SLASEE4C
//!   Table 5-22 note 2, p. 39: "for higher accuracy").
//! - The reference and DCO calibration: [`reference_factor()`], [`dco_tap_16mhz()`] and, on the
//!   MSP430FR2x5x, `dco_tap_24mhz()`.

use crate::_pac;
use crate::crc::Crc;
use crate::pmm::ReferenceVoltage;

// The device descriptors, from 1A00h (SLASEC4D Table 6-70, p. 107; SLASEO7C Table 9-30, p. 71; SLASE59F
// Table 6-22, p. 60; SLASEE4C Table 6-18, p. 61). They're programmed in production and read-only, so
// reading them can't disturb anything else.
#[inline(always)]
fn tlv() -> &'static _pac::tlv::RegisterBlock { unsafe { &*_pac::Tlv::ptr() } }

/// The device ID, which tells the devices apart (SLASEC4D Table 6-69, p. 107; SLASEO7C Table 9-29, p. 71;
/// SLASE59F Table 6-21, p. 60; SLASEE4C Table 6-17, p. 61):
///
/// | Device       | ID    |
/// |--------------|-------|
/// | MSP430FR2355 | 830Ch |
/// | MSP430FR2353 | 830Dh |
/// | MSP430FR2155 | 831Eh |
/// | MSP430FR2153 | 831Dh |
/// | MSP430FR2476 | 832Ah |
/// | MSP430FR2475 | 832Bh |
/// | MSP430FR2433 | 8240h |
/// | MSP430FR2522 | 8310h |
/// | MSP430FR2512 | 831Ch |
#[inline]
pub fn device_id() -> u16 { tlv().device_id().read().bits() }

/// The hardware revision of the die (SLASEC4D 6.13.1, p. 109: "The hardware revision is also stored in
/// the Device Descriptor structure"): 20h for revision B of the MSP430FR2x5x (SLAZ695J 5.3, p. 5); 11h
/// for revisions B and C and 10h for revision A of the MSP430FR2433 (SLAZ664S 5.3, p. 4 to p. 5). The
/// errata sheets also describe the revision marking on the package.
///
/// Not on the MSP430FR247x and MSP430FR25x2: their data sheets list it at 1A06h, but their errata sheets
/// say "This device does not support reading the hardware revision from memory" (SLAZ726B 5.3, p. 4;
/// SLAZ705H 5.3, p. 4).
#[cfg(feature = "tlv_hw_revision")]
#[inline]
pub fn hardware_revision() -> u8 { tlv().hw_revision().read().bits() }

/// The firmware revision, set per unit
#[inline]
pub fn firmware_revision() -> u8 { tlv().fw_revision().read().bits() }

/// Whether the device descriptors match the CRC stored with them, so that the values in this module can
/// be trusted. This restarts `crc`, so any signature it was computing is lost.
///
/// The CRC is the CRC-CCITT of 1A04h to 1AF7h on the MSP430FR2x5x and MSP430FR247x, and of 1A04h to 1AF5h
/// on the MSP430FR2433 and MSP430FR25x2 (SLASEC4D Table 6-70 note 1, p. 107; SLASEO7C Table 9-30 note 1,
/// p. 72; SLASE59F Table 6-22 note 1, p. 60; SLASEE4C Table 6-18 note 1, p. 61).
pub fn crc_matches(crc: &mut Crc) -> bool {
    // The data sheets give the polynomial and the range, but not the seed or the bit order. Seed FFFFh,
    // with the bytes in address order through CRCDIRB, reproduced the stored CRC on an MSP430FR2476. That
    // is the sequence that gives 029B1h for "123456789" in SLAU445I Example 11-2, p. 356.
    crc.reset(0xFFFF);
    for byte in tlv().crc_data_iter() {
        crc.add_byte_lsb(byte.read().bits());
    }
    crc.result() == tlv().crc_value().read().bits()
}

/// The die record: the die's lot wafer ID, its X and Y position, and its test result, each set per unit
/// (SLASEC4D Table 6-70, p. 107; SLASEO7C Table 9-30, p. 71; SLASE59F Table 6-22, p. 60; SLASEE4C
/// Table 6-18, p. 61)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DieRecord {
    /// The lot wafer ID, from the four bytes at 1A0Ah to 1A0Dh
    pub lot_wafer_id: u32,
    /// The die X position
    pub x_position: u16,
    /// The die Y position
    pub y_position: u16,
    /// The test result
    pub test_result: u16,
}

/// Read the die record
#[inline]
pub fn die_record() -> DieRecord {
    let tlv = tlv();
    DieRecord {
        lot_wafer_id: tlv.lot_wafer_id().read().bits(),
        x_position: tlv.die_x_position().read().bits(),
        y_position: tlv.die_y_position().read().bits(),
        test_result: tlv.test_result().read().bits(),
    }
}

/// The factory calibration of the internal temperature sensor: the ADC counts it gave at 30 °C and at
/// a high temperature, 105 °C on the MSP430FR2x5x and MSP430FR247x and 85 °C on the MSP430FR2433 and
/// MSP430FR25x2, measured against the internal reference. (SLAU445I 1.13.3.3, p. 60: "The temperature
/// sensor is calibrated using the internal voltage references"; 105 °C: SLASEC4D Table 6-70 note 3, p. 108,
/// and SLASEO7C Table 9-30, p. 72; 85 °C: SLASE59F Table 6-22, p. 60, and SLASEE4C Table 6-18, p. 61)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TempSensorCalibration {
    count_30c: u16,
    count_high: u16,
    high_celsius: i16,
    /// Tenths of a degree per count, in 1/65536ths: worked out once, so that each conversion takes a
    /// multiplication instead of a division, which the MSP430 has no hardware for
    scale_q16: i32,
}

impl TempSensorCalibration {
    /// The calibration for readings against the internal reference at `vref`.
    #[inline]
    pub fn new(vref: ReferenceVoltage) -> Self {
        // Pairs of counts at 30 °C and at the high temperature, one pair per reference level (1.5 V at
        // 1A1Ah, 2.0 V at 1A1Eh and 2.5 V at 1A22h: SLASEC4D Table 6-70, p. 108; SLASEO7C Table 9-30, p. 72;
        // only 1.5 V on the MSP430FR2433 and MSP430FR25x2: SLASE59F Table 6-22, p. 60; SLASEE4C Table 6-18,
        // p. 61)
        let calibration = tlv().adc_temp_cal(vref as usize);
        // The high calibration temperature: 105 °C (SLASEC4D Table 6-70 note 3, p. 108; SLASEO7C Table 9-30,
        // p. 72) or 85 °C (SLASE59F Table 6-22, p. 60; SLASEE4C Table 6-18, p. 61)
        #[cfg(feature = "enhanced_ref")]
        let high_celsius = 105;
        #[cfg(not(feature = "enhanced_ref"))]
        let high_celsius = 85;
        let count_30c = calibration.temp_30c().read().bits();
        let count_high = calibration.temp_high().read().bits();
        let span = count_high as i32 - count_30c as i32;
        // 75 or 55 °C over the counts between the two calibration points, in tenths of a degree (SLAU445I
        // 1.13.3.3, p. 60: Equations 9 and 10)
        // A blank calibration (both counts equal) gives 0
        let scale_q16 = ((high_celsius as i32 - 30) * 10 * 65536).checked_div(span).unwrap_or(0);
        TempSensorCalibration { count_30c, count_high, high_celsius, scale_q16 }
    }

    /// The count at 30 °C
    #[inline]
    pub fn count_30c(&self) -> u16 { self.count_30c }

    /// The count at [`high_celsius`](TempSensorCalibration::high_celsius)
    #[inline]
    pub fn count_high(&self) -> u16 { self.count_high }

    /// The high calibration temperature in °C
    #[inline]
    pub fn high_celsius(&self) -> i16 { self.high_celsius }

    /// Convert a reading of the temperature sensor to tenths of a degree Celsius, rounded to the
    /// nearest tenth (SLAU445I 1.13.3.3, p. 60: Equations 9 and 10).
    ///
    /// The reading must be taken as the calibration was: against the same internal reference
    /// ([`PositiveReference::Internal`](crate::adc::PositiveReference::Internal)), at the ADC's full
    /// resolution (12-bit, or 10-bit on the MSP430FR2433 and MSP430FR25x2), in the unsigned format and
    /// with a sample time of at least 30 µs (SLAU445I 21.2.7.8, p. 556: "the sample period must be greater
    /// than 30 µs").
    #[inline]
    pub fn decicelsius(&self, count: u16) -> i16 {
        let delta = count as i32 - self.count_30c as i32;
        // Wrapping, so a garbled calibration gives a garbled value rather than a panic
        let tenths = delta.wrapping_mul(self.scale_q16).wrapping_add(1 << 15) >> 16;
        // Plus 30 °C, the lower calibration point (SLAU445I 1.13.3.3, p. 60: Equations 9 and 10)
        (tenths + 300) as i16
    }
}

/// The ADC gain correction factor, in 1/32768ths: multiply results by it and divide by 32768 (SLAU445I
/// 1.13.3.2, p. 60: Equation 6). The factory measured it at 2.4 V and room temperature, with external
/// references on VeREF+ and VeREF- (SLASEO7C Table 9-30 note 3, p. 72; the other data sheets don't give
/// these conditions), so other settings can need a different factor.
#[inline]
pub fn adc_gain_factor() -> u16 { tlv().adc_gain_factor().read().bits() }

/// The ADC offset correction, in counts: add it to results after the gain correction (SLAU445I 1.13.3.2,
/// p. 60: Equations 4 and 7; it is "stored as a twos-complement number"). Measured as
/// [`adc_gain_factor()`] was (SLASEO7C Table 9-30 note 4, p. 72).
#[inline]
pub fn adc_offset() -> i16 { tlv().adc_offset().read().bits() as i16 }

/// The correction factor of the internal reference at `vref`, in 1/32768ths: multiply results
/// measured against it by the factor and divide by 32768 (SLAU445I 1.13.3, p. 59, in "1.5-V Reference
/// Calibration": Equations 2 and 3).
#[inline]
pub fn reference_factor(vref: ReferenceVoltage) -> u16 {
    // The 1.5 V, 2.0 V and 2.5 V factors at 1A28h, 1A2Ah and 1A2Ch (SLASEC4D Table 6-70, p. 108; SLASEO7C
    // Table 9-30, p. 72), or the 1.5 V factor only, at 1A20h (SLASE59F Table 6-22, p. 61; SLASEE4C
    // Table 6-18, p. 62)
    tlv().ref_factor(vref as usize).read().bits()
}

/// The DCO tap setting for 16 MHz at 30 °C, a value for CSCTL0 (SLASEC4D Table 6-70, p. 108; SLASEO7C
/// Table 9-30, p. 72; SLASE59F Table 6-22, p. 61; SLASEE4C Table 6-18, p. 62). "Loading this value to the
/// CSCTL0 register significantly reduces the FLL lock time when the MCU reboot or exits from a low-power
/// mode", and "If a possible frequency overshoot caused by temperature drift is expected after exit from
/// an LPM, TI recommends dividing the DCO frequency before use" (SLAU445I 1.13.3.4, p. 60).
#[inline]
pub fn dco_tap_16mhz() -> u16 {
    // At 1A2Eh after the three reference factors (SLASEC4D Table 6-70, p. 108; SLASEO7C Table 9-30, p. 72),
    // at 1A22h after the one (SLASE59F Table 6-22, p. 61; SLASEE4C Table 6-18, p. 62)
    tlv().dco_tap_16mhz().read().bits()
}

// The MSP430FR247x table lists this entry too (SLASEO7C Table 9-30, p. 72), but those devices run at up to
// 16 MHz (SLASEO7C 8.3, p. 20), and on an MSP430FR2476 the calibration length at 1A27h read 08h instead of
// the table's 0Ah, which ends the calibration before 1A30h.
/// The DCO tap setting for 24 MHz at 30 °C, a value for CSCTL0 (SLASEC4D Table 6-70 note 4, p. 108: "This
/// value can be directly loaded into the DCO bits in the CSCTL0 register to get an accurate 24-MHz
/// frequency at room temperature, especially when MCU exits from LPM3 and below"). See
/// [`dco_tap_16mhz()`].
#[cfg(feature = "enhanced_cs")]
#[inline]
pub fn dco_tap_24mhz() -> u16 { tlv().dco_tap_24mhz().read().bits() }
