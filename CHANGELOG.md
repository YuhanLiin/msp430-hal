# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
- Add XT1 support, in crystal and bypass mode, through `Xt1Config` and `ClockConfig::xt1clk_on()`. XT1 can source ACLK, MCLK and SMCLK, and serve as the FLL reference. `Xt1Config::crystal()` and `bypass()` set up a 32 kHz XT1; on the MSP430FR2x5x, `crystal_hf()` and `bypass_hf()` set up a 1 MHz to 24 MHz XT1, typed as `HighFrequency`. The XT1 pins of each device are available as `clock::Xt1Xin` and `clock::Xt1Xout`.
- Add `ClockConfig::try_freeze()`, which gives up on XT1 after a timeout instead of blocking, and `ClockConfig::xt1clk_off()` to fall back to the internal oscillators.
- Add `Xt1clk::is_faulted()` and `Xt1clk::clear_fault()`. XT1 faults switch the clocks it sources to a fallback oscillator until the fault flags are cleared.
- Add `Xt1clk::enable_fault_interrupt()` and `Xt1clk::disable_fault_interrupt()`, which report oscillator faults through the user NMI, and `clock::take_fault_interrupt()` to handle it in the `UNMI` interrupt handler.
- Add `ClockConfig::refo_low_power()` on the MSP430FR2x5x (enhanced clock system).
- Add `TimerConfig::vloclk()`, for the timers that can be clocked from the VLO: TA0 and TA2 on the MSP430FR247x, and TA0 on the MSP430FR25x2.
- Add timer cascading: `TimerConfig::cascade()` clocks a timer from the CCR2 output of another timer, set up with `SubTimer::into_cascade_output()` or `PwmUninit::into_cascade_output()`, so that it counts the periods of that timer. Available for TA1 (from TA0) and TA3 (from TA2) on the MSP430FR247x, TA1 (from TA0) on the MSP430FR25x2, and TB1 (from TB0) on the MSP430FR2x5x.
- Add `TimerParts2` for timers with two capture/compare registers: TA2 and TA3 on the MSP430FR2433, which couldn't be used before. They aren't connected to any pins, so they work as timers only.
- The DCO is now trimmed in software for every frequency except the device's highest, as the user's guide recommends, so the FLL locks reliably.
- The 8 MHz and 16 MHz DCO settings now run at 7.995 MHz and 15.991 MHz. They previously ran slightly above 8 MHz and 16 MHz, which needed an extra FRAM wait state and, at 16 MHz, exceeded the maximum frequency of most devices.
- FRAM wait states now also cover MCLK while the DCO is being configured, which runs undivided before the MCLK divider is applied.
- The RTC can now be clocked from ACLK and XT1CLK. `Rtc::start()` now resets the counter after selecting the clock, as the user's guide recommends.
- `enter_lpm3_5()` now accepts an RTC clocked from XT1CLK. After a wake-up from LPM3.5, `Pmm::new_locked()` and `Pmm::unlock_lpm5()` allow XT1 to be reconfigured before the pins are unlocked, so it keeps clocking the RTC.
- PWM pins no longer specify their alternate function separately: it is derived from the pin type.
- Breaking: `ClockConfig::aclk_vloclk()` is no longer available on the MSP430FR25x2, which can't source ACLK from VLO.
- Fixed XT1 pins on the MSP430FR2x5x, which need alternate function 2.
- Fixed LPM3.5 entry stopping XT1 on devices other than the MSP430FR2x5x, and not resetting `P2SEL1`. LPMx.5 entry now also clears ACLKREQEN, as the user's guide requires.
- Fixed `delay_ns()` and `delay_us()`, which waited whole milliseconds, at most 1 ms. They now wait the requested time, counted in MCLK cycles, plus about 60 MCLK cycles for the call. `SysDelay` also implements the embedded-hal 0.2 `DelayUs` now.
- Fixed the TB0 clock input pin (TB0CLK) on the MSP430FR247x, which is P6.1, not P2.7.
- Fixed `InfoMemory::write()` and `into_unprotected()` switching off the program FRAM write protection (PFWP) until the next reset. `write()` now runs with interrupts disabled, as the user's guide recommends.
- Fixed `Spi::change_mode()` disabling the SPI interrupts.
- Fixed pin remapping on the MSP430FR247x and MSP430FR25x2 resetting the rest of SYSCFG2/SYSCFG3: setting up eUSCI_B0 switched an RTC on ACLK to SMCLK (and cleared the ADC input enables on the MSP430FR25x2), and the eUSCI_A0, eUSCI_B1, TA2 and TA3 remaps undid each other.
- Breaking: the MSP430FR25x2 ADC is 10-bit with a 1.5 V reference only, so `Resolution::Bits12` and `ReferenceVoltage::V2_0`/`V2_5` are not available there. Its analog inputs are now enabled with `to_adc_mode()` (SYSCFG2.ADCPCTLx), as on the MSP430FR2433; `Alternate3` selected CapTIvate.
- Fixed on the MSP430FR25x2: 256 bytes of information memory, remapped eUSCI_B0 pins on alternate function 2, and pin functions as in the data sheet (CAP1.x only on the MSP430FR2522).
- Fixed on the MSP430FR2x5x: eCOMP input pins on alternate function 3, pin directions as in the data sheet, and the SAC op-amp pins on P3 only on the MSP430FR235x.
- Fixed on the MSP430FR247x: PWM on TA2/TA3 with `RemappedMapping` uses the remapped pins (`PwmPeriph`, `PwmUninit` and `Pwm` take the pin mapping), the TB0 CCR6 capture input is P4.4, and the TA3 CCR2 PWM pin P3.7 uses alternate function 1.
- Fixed on the MSP430FR2433: TA1 capture inputs on P1.5/P1.4, ACLK output on P2.2 on alternate function 2, and the eUSCI_A1 SPI STE pin (P3.1) added.
- Fixed the MODCLK frequency constants, now the data sheets' typical values: 4.8 MHz on the MSP430FR2433 and MSP430FR25x2, 3.8 MHz on the MSP430FR247x.
- Add `Capture::interrupt_capture()` for CCR0, whose capture flag is cleared when its own interrupt is serviced. Captures arriving while a capture is read are now reported as overcaptures instead of being lost.
- GPIO batches now turn pin interrupts off while reconfiguring a port, and switch pins whose two function select bits both change through PxSELC. Pins a device doesn't have start out `Unavailable` and can't be used. On the MSP430FR2433 and MSP430FR25x2, a batch's `to_adc_mode()` and `from_adc_mode()` now set and clear the pin's ADCPCTL bit; before, they only changed its type.
- Add `to_output_low()` and `to_output_high()` for GPIO pins, since PxOUT is undefined after a reset.
- Add `Batch::pulldown_unused()`, which gives every pin that is still a floating input its pulldown and keeps the configuration of the others, to terminate unused pins as the user's guide recommends. `Batch::pulldown_all()` and `pullup_all()` no longer turn the slots of pins a device doesn't have into usable pins.
- Add `clock::fll_status()`, which reports whether the FLL is locked or the DCO is too slow, too fast or out of range.
- PWM `max_duty_cycle()` and `get_max_duty()` now return the period (CCR0 + 1), so the maximum duty cycle is 100 %.
- `Timer::count()` takes the median of three reads, for timers clocked asynchronously to MCLK.
- The PMM is unlocked and locked again around each register write, and `enable_internal_reference()` waits until the reference has settled.
- ADC: `read_count()` no longer returns the result of a pending conversion of another channel, and `count_to_mv()` divides by 2^n with a shift instead of a division. Its documentation gives the conversion formula it inverts (SLAU445I 21.2.1) and how far the data sheets' 2^n - 1 is from it.
- UART: the eUSCI is configured while held in reset, and a baud rate above a third of the clock panics instead of being clamped.
- I2C: clock divisors below the user's guide minimum (4, or 8 with several masters) panic, and `zero_byte_write()` sets START and STOP together. Breaking: `send_nack()` is only available in slave roles.
- Breaking: `Wdt::wait()` is only available in interval mode.
- Breaking: the HAL uses the MSP430FR2355 PAC v0.7, the MSP430FR2433 PAC v0.3, the MSP430FR247x PAC v0.4 and the MSP430FR25x2 PAC v0.4, which name peripherals, registers, fields and enum variants the same on every device, as the family user's guide (SLAU445I) names them. On the MSP430FR2433 the peripherals are named as on the other devices: `periph.wdt_a`, `periph.frctl`, `periph.e_usci_a0`, `periph.ta0` and so on, instead of `periph.watchdog_timer`, `periph.fram`, `periph.usci_a0_uart_mode` and `periph.timer_0_a3`. On every device the backup memory is `periph.bakmem` instead of `periph.bkmem`.
- `WdtClkPeriods`, `SvsState`, `fram::WaitStates`, `sys::NmiEdge`, `icc::Priority`, `adc::DataFormat`, `adc::ConversionMode` and `ir::IrMode` are the PAC's enums, with the same variants on every device. Breaking: `SvsState`'s variants are `Disabled` and `Enabled` instead of `Svshe0` and `Svshe1`, and on the MSP430FR2433 `WdtClkPeriods` is named as on the other devices (`_2g` instead of `_2048m`, for example).
- `pmm::ResetCause`, `pmm::ReferenceVoltage`, `sys::SystemNmi` and the ADC settings `SampleTime`, `ClockDivider`, `Predivider`, `Resolution`, `SamplingRate`, `TriggerSource` and `AdcVector` are the PAC's enums too, so the HAL no longer decodes register values itself, and a device has only the variants its data sheet lists (`Resolution::Bits12` and `TriggerSource::Comparator`, for example). Breaking: `SampleTime::_4` is `SampleTime::Cycles4` and so on, `Resolution::_10BIT` is `Resolution::Bits10`, `SamplingRate::_200KSPS` is `SamplingRate::Max200ksps`, and `ReferenceVoltage::_1V5` is `ReferenceVoltage::V1_5`; these enums no longer implement `Default`, and `ReferenceVoltage` no longer implements `Ord`. `ClockDivider` and `Predivider` keep their variant names.
- With the `defmt` feature, the values the HAL returns implement `defmt::Format`: the interrupt vectors `GpioVector`, `TimerVector`, `SpiVector` and `I2cVector`, `I2cEvent`, `i2c::TransmissionMode`, `AdcInterruptFlags` and `tlv::TempSensorCalibration`, and the PAC enums the HAL re-exports, such as `pmm::ResetCause`, `sys::SystemNmi`, `adc::AdcVector` and `clock::FllStatus`, as the feature enables the PACs' `defmt` feature. `GpioVector` and `TimerVector` also derive `Debug`, `Copy`, `Clone`, `PartialEq` and `Eq`.
- Add `enter_lpm0_with_interrupts()`, `request_lpm3_with_interrupts()` and `request_lpm4_with_interrupts()`, which enable interrupts in the same instruction that starts the sleep. Entering a low-power mode is now a compiler barrier.
- Add `enter_lpm0_fram_off()` and `enter_lpm0_fram_off_with_interrupts()`, which power the FRAM down while the CPU sleeps in LPM0 (FRPWR). The first access to the FRAM powers it up again.
- Add `Pmm::set_lpm3_5_switch()` and `Pmm::lpm3_5_switch_connected()` on the MSP430FR2433 and MSP430FR25x2, the devices that have it, for manual control of the switch that connects the LPM3.5 domain to the core supply (LPM5SM, LPM5SW). In manual mode `enter_lpm3_5()` and `enter_lpm4_5()` disconnect it first, as the user's guide recommends. Not tested on hardware yet.
- Breaking: the SAC amplifier modes only accept the inputs the user's guide supports (SLAU445I Table 20-1). `PositiveInput`, for the open-loop and non-inverting modes, no longer has a `Dac` variant; the inverting amplifier takes a `BiasInput` (OA+ or the DAC) and the buffer a `BufferInput` (OA+, the DAC or the paired amplifier).
- Add `DacConfig::configure_with_interrupts()` and `Dac::data_loaded()`, for the SAC DAC interrupt that requests new data after a timer-triggered load.
- Add `Pwm::duty()`, `Pwm::enable()` and `Pwm::disable()`, which were only available through embedded-hal 0.2.
- The example projects now link `libmul_f5`, so multiplication uses the hardware multiplier (MPY32): 16-bit products are about 5 times and 32/64-bit products about 10 times faster than with `libmul_none`. The commented-out `libmul_32` used the register addresses of other devices, which are PM5CTL0 here.
- Add the `sys` module. `SysParts` gives the RST/NMI pin (reset or NMI mode, pull resistor, reset filter, NMI edge and interrupt), the vacant memory access interrupt and the JTAG mailbox (16 and 32-bit transfers). On the MSP430FR2433 the pin always keeps a pull resistor: clearing SYSRSTRE would also disable the TEST/SBWTCK pin's pull-down there (erratum PORT28), so `RstPull` has no `None`. `sys::take_nmi_pin_interrupt()` and `sys::take_system_nmi()` serve the `UNMI` and `SYSNMI` interrupt handlers.
- Add interrupt vectors in RAM (SYSRIVECT): `SysParts::interrupt_vectors` copies the vector table to the top of RAM with `use_ram()`, changes handlers while the program runs with `set_handler()`, and goes back to the FRAM table with `use_fram()`. Only a BOR switches back by itself, so the RAM table stays in use through other resets and through flashing a new program. `memory.x` must leave the top 128 bytes of RAM out for the table: the MSP430FR2476 example `ram_vectors` and its `memory.x` say how.
- Add the rest of the SYS settings: `SysParts::bsl` protects the bootloader (BSL) memory, switches it off or assigns RAM to it, and reports a BSL entry; `SysParts::jtag_pins` dedicates the JTAG pins to JTAG and `SysParts::pmm_protection` protects the PMM registers, each until the next BOR.
- Add `Pmm::take_reset_cause()`, which reports why the device reset, `Pmm::software_bor()` and `Pmm::software_por()`, and `Pmm::set_svsh()` to turn the high-side supply voltage supervisor off in LPM2 to LPM4.
- Add FRAM bit error handling: `Fram::set_uncorrectable_bit_error_action()` resets the device or requests the system NMI for errors the FRAM can't correct, and `Fram::enable_correctable_bit_error_interrupts()` reports corrected ones.
- Add `Fram::set_writable_program_fram()` on the MSP430FR2x5x and MSP430FR2522, which leaves the start of program FRAM writable (FRWPOA).
- Add `ClockConfig::reset_on_fll_unlock()`, which resets the device if the DCO runs too fast for the FLL.
- Add the FLL unlock interrupt (FLLWARNEN): `clock::enable_fll_unlock_interrupt()` requests the user NMI when the FLL loses its lock, and `clock::fll_unlock_history()` and `clock::clear_fll_unlock_history()` read and clear what the DCO has been since: too slow, too fast or both. `clock::take_fault_interrupt()` serves it in the `UNMI` handler, as it does oscillator faults.
- Fixed `Fram::set_wait_states()` leaving the FRAM controller registers unlocked.
- Add ADC reference selection: `Adc::with_reference()` measures against the internal reference or external references on the VeREF+ and VeREF- pins instead of AVCC and AVSS.
- Add ADC sequences, repeated conversions and hardware triggers: `Adc::start()` takes a `ConversionConfig` (conversion mode, trigger source among software, the RTC, a timer and eCOMP0, pulse or extended sampling, back-to-back conversions), and `Adc::result()` and `Adc::stop()` go with it. `PwmUninit::into_adc_trigger()` sets up the timer trigger without a pin.
- Add the ADC window comparator, `Adc::set_window()`, and ADC interrupts: `Adc::enable_interrupts()`, `Adc::interrupt_flags()` and `Adc::interrupt_source()`.
- Add signed ADC results, through `AdcConfig::data_format`.
- Add `Pmm::enable_vref_output()`, which outputs the 1.2 V reference on the VREF+ pin. It starts the buffered bandgap (REFBGEN) and waits until it is ready (REFBGRDY).
- Add the `tlv` module with the device descriptors: the device ID, hardware revision (on the MSP430FR2x5x and MSP430FR2433 only, as the errata sheets of the MSP430FR247x and MSP430FR25x2 say it can't be read there) and firmware revision, die record, ADC and reference calibration, and the DCO tap settings for 16 MHz (and 24 MHz on the MSP430FR2x5x). `tlv::crc_matches()` checks the descriptors against their CRC. `TempSensorCalibration` converts temperature sensor readings with the calibration, and the temperature sensor examples now use it instead of typical values, some of which were wrong.
- Add eCOMP edge flags, `Comparator::rising_edge_flag()`, `falling_edge_flag()` and `clear_edge_flags()`, and `Comparator::interrupt_source()` for the interrupt handler.
- Add center-aligned PWM, `PwmParts3::new_center_aligned()` and `PwmParts7::new_center_aligned()`, and `Pwm::set_polarity()` for active-low outputs.
- Add `period_output` to `PwmParts3` and `PwmParts7`: the CCR0 output, a square wave at half the PWM frequency, on the timers that have a pin for it (TA2, TA3 and TB0 on the MSP430FR247x).
- PWM duty cycle changes on a Timer_B go through its compare latches, so a change never cuts a period short or leaves it high to its end: with center-aligned PWM the new duty cycle loads when the timer next counts to 0 or to the top, with edge-aligned PWM when it reaches the old duty cycle (CLLD = 11b). Erratum TB25 makes the modes that load at the start of a period load at once on the MSP430FR2x5x and the MSP430FR247x, the supported devices that have a Timer_B; it doesn't list CLLD = 11b, which measured clean on an MSP430FR2476. Add `Pwm::enable_period_interrupt()`, `Pwm::disable_period_interrupt()` and `Pwm::take_period_flag()`, to change a duty cycle exactly once per period.
- Add `TimerConfig::high_impedance_trigger()`. After reset, a high eCOMP output switches the outputs of some Timer_B peripherals to high impedance (eCOMP0 those of TB0 and TB1, eCOMP1 those of TB2 and TB3), which stops their PWM; this selects the TBxTRG pin instead, or nothing.
- Add `TimerConfig::counter_length()` for Timer_B, and `Timer::start_up_down()` for up/down counting. `Timer::resume()` now keeps the counting mode instead of switching to up mode.
- Add `TimerConfig::compare_latch_groups()` for Timer_B (TBCLGRP), which loads the compare latches of a group of capture/compare registers together, once all of them are written, for example PWM duty cycles that must change in the same period. Measured on an MSP430FR2476, the grouping works with center-aligned PWM, and not with edge-aligned PWM, whose compare latches load when the timer reaches the old duty cycle. The MSP430FR2476 example `pwm_center_aligned` groups the two outputs of its half bridge.
- Add captures started from software: the `config_capN_software()` methods and `Capture::trigger_capture()`.
- Add `CaptureParts2`, for the timers with two capture/compare registers: TA2 and TA3 on the MSP430FR2433, where TA3 can capture the outputs of TA2.
- Add the UART multiprocessor formats and automatic baud-rate detection, through `SerialConfig::mode()`, with `Tx::send_address()`, `Tx::send_break()`, `Rx::set_dormant()`, `Rx::read_with_address_flag()` and `Rx::auto_baud_errors()`.
- Add `SerialConfig::irda()` for IrDA, `SerialConfig::deglitch()`, and `SerialConfig::break_interrupts()`, which reports received breaks as the new `RecvError::Break` (breaking for exhaustive matches).
- Add UART start-bit and transmit-complete interrupts, and `interrupt_source()` on `Tx` and `Rx`. It leaves the receive and transmit flags set, so `read()` and `write()` work after it.
- Fixed UART `flush()` returning while the last character was still being sent. It now waits until the eUSCI is idle.
- Add the inverted UART of eUSCI_A1 on the MSP430FR2x5x: its pins P4.2 and P4.3 in alternate function 2.
- Add `SpiConfig::seven_bit_characters()`, and `SpiConfig::single_slave_bus()`, where the eUSCI drives the slave's enable signal on STE.
- Add `SpiConfig::multi_master_bus()`, for an SPI bus shared with other masters, which take it through the STE pin. Writes wait while another master has the bus, as erratum USCI50 requires, see `Spi::bus_available()`, and a transfer that another master interrupts returns the new `SpiErr::BusConflict` (breaking for exhaustive matches), after which the eUSCI is ready for the next one.
- Add I2C settings: `clock_low_timeout()`, `byte_counter()` (with an automatic STOP for single masters) and `ack_last_byte()`, and for slaves `general_call()`, three more own addresses (`own_address()`), `address_mask()`, `software_address_ack()` with `I2cRoleSlave::acknowledge_address()`, `early_tx_interrupt()` and `I2cRoleSlave::received_address()`.
- Add the `ir` module for the infrared modulator: `IrModulator` combines the CCR2 outputs of two timers (from `PwmUninit::into_ir_input()`) and data from software or eUSCI_A0's UART into an ASK or FSK signal on eUSCI_A0's TXD pin. Only one TXD pin carries it, in the pin mapping `ir::IrMapping`: on the MSP430FR247x it's P1.4, as eUSCI_A0's remapped pin P5.2 doesn't. The MSP430FR2476 example `ir_remote` sends NEC remote control frames.
- Add the `icc` module on the MSP430FR2x5x: the Interrupt Compare Controller, which gives each interrupt source one of four priorities and lets higher priorities interrupt lower ones.
- Add the `mfm` module on the MSP430FR2x5x: the Manchester Function Module on P5.0 and P5.1, with eUSCI_B1 as its SPI slave through `SpiConfig::mfm_slave()`. Not tested on hardware yet.
- Add `ClockConfig::mclk_dcoclk_hz()`, which runs the DCO at any frequency from 1 MHz up to the device maximum. The FLL locks to the largest multiple of its reference that doesn't exceed the target, with the DCO in the range whose nominal frequency is closest to it. The MSP430FR2476 example `dco_delay_test` checks it and the delays on the board, with an oscilloscope, and optionally with a function generator as the FLL reference.
- Fixed `I2cRoleSlave::poll()` panicking when the start flag had been cleared, by reading the interrupt vector say, while a received byte was still unread and the master had started reading. It now reports `OverrunWrite`.
- Fixed UART receive missing an overrun between reading UCAxSTATW and UCAxRXBUF: UCOE is now checked again after the read, as the user's guide recommends.
- Fixed SPI `interrupt_source()` clearing the flag it reported, after which the checked `read()` and `write()` returned `WouldBlock`.
- Fixed I2C `schedule_stop()` clearing a received byte's flag, after which `read_rx_buf()` blocked. It still clears the other flags.
- I2C `byte_counter()` takes no `auto_stop` for slaves, multi-masters and master-slaves, which can be in slave mode, where the automatic STOP isn't available (breaking).
- `SerialConfig::irda()` with `IrdaClock::Brclk` asserts the prescaler UCBRx is at least 5, as the user's guide requires.
- The UART, SPI and I2C set-up now follows the user's guide order: the pin remapping bits are set while the eUSCI is held in reset.
- `Rtc::enable_interrupts()` clears an old overflow first, so it doesn't fire at once.
- `Icc::enable()` and `Icc::disable()` change ICCEN with interrupts disabled, as the user's guide recommends.
- Timer_A capture/compare registers are written with the timer stopped, and output modes are changed with the timer stopped, as the user's guide recommends. The timer pauses for a few cycles each time.
- Capture input A of capture/compare register 0 is `NoCapturePin` where nothing is connected to it: TA0 and TA1 on the MSP430FR2433, TA1 on the MSP430FR247x and the MSP430FR25x2, TB2 and TB3 on the MSP430FR2x5x (breaking for code selecting it).
- On the MSP430FR2x5x, `request_lpm3()` and `request_lpm4()` enter LPM0 while XT1 runs in high-frequency mode, and LPMx.5 switches a high-frequency XT1 off, as the data sheet requires that before LPM3, LPM4 and LPMx.5.
- Work around errata CS13 and PMM32 on the MSP430FR2355, MSP430FR2433 and MSP430FR25x2 in `request_lpm3()` and `request_lpm4()`: the FRAM is switched to inactive from RAM before the sleep, and a DCO above 2 MHz is slowed to below 1 MHz until the CPU returns, so interrupt handlers run slower meanwhile. Called with interrupts enabled, they disable them between the FRAM switch-off and the sleep instruction, which enables them again. On the MSP430FR2433 the FRAM bit error handling is paused around the sleep for erratum GC5, and turned on again after a read of five FRAM cache lines, so that at least one read reaches the FRAM. The `lpm` module documents what interrupt handlers need on these devices (`#[interrupt(wake_cpu)]`, as the errata also name returns from a handler into the sleep).
- Work around erratum RTC15 on the MSP430FR2355, MSP430FR2433 and MSP430FR25x2: `Rtc::start()` toggles XIN after moving the RTC off a stopped XT1CLK. It clears XT1OFFG first and toggles XIN only if the flag comes back, so a stale flag doesn't make it drive XIN against a running XT1.
- The MSP430FR2476 examples' `memory.x` no longer lets ROM overlap the interrupt vectors.
- New MSP430FR2476 examples for the new features, each with test steps for an oscilloscope, function generator, multimeter or serial terminal: `tlv`, `vacant_memory`, `vref_output`, `svsh_current`, `fll_unlock_reset`, `pwm_center_aligned`, `pwm_pin_mappings`, `pwm_period_interrupt`, `timer_b_high_impedance`, `timer_external_clock`, `timer_b_modes`, `capture_software`, `capture_frequency`, `adc_reference`, `adc_timer_trigger`, `adc_window`, `adc_sequence`, `adc_comparator_trigger`, `comparator_edges`, `uart_multiprocessor`, `uart_irda`, `uart_break_interrupts`, `uart_auto_baud`, `spi_single_slave`, `spi_multi_master`, `fll_unlock_interrupt`, `ram_vectors` and `lpm0_fram_off`.
- `Pmm::new()` and `Pmm::new_locked()` document the data sheets' order: configure the ports, then clear LOCKLPM5.
- Examples: the watchdog examples no longer rely on P1OUT keeping its value through a reset; the MSP430FR2433 and MSP430FR25x2 ADC examples sample long enough for their data sheets; the MSP430FR2476 ADC and capture examples moved off P1.1, which the LaunchPad's temperature sensor drives, and the capture example prints on the backchannel UART; the MSP430FR2355 SPI slave example configures the master first (erratum USCI47); the MSP430FR25x2 capture and alternate examples capture on CCR2 and output SMCLK on P1.2; LED variable names on the MSP430FR2476 match the board.
- The comments in the HAL and the examples now cite the TI documents each register setting, procedure, pin function and limit comes from: document, section, table or figure, and page. `REFERENCES.md` lists the documents, their revisions and how references are written.
- New examples, so that every feature can be tried on every device that has it: ports of the MSP430FR2476 examples to the MSP430FR2355, MSP430FR2433 and MSP430FR2522, examples of the features that had none, and tests of the errata CS13, PMM32, GC4, GC5, RTC15, USCI42, USCI47 and TB25. Nobody has run them on hardware yet, and each starts with a note saying so.
- REFERENCES.md lists the errata sheets of all nine supported devices, with a table of every erratum and how the HAL handles it. The code rustc generates was checked for the CPU errata that TI's compilers work around (CPU21, CPU22, CPU40) and for CPU46: it has no POPM, no indirect `@PC` source and no jump followed by 0X40h/0X50h. The README's new Debugging section describes erratum EEM23.
- Fixed the MSP430FR2433 eUSCI on MODCLK possibly getting no clock: `use_modclk()` and `to_master_using_modclk()` set MODOSCREQEN, as the user's guide doesn't say which kind of MODOSC request the eUSCI makes.
- `Fram::new()` switches the FRAM bit error handling off (UBDRSTEN, UBDIE, CBDIE), which otherwise only a BOR does, so a program no longer inherits it from the program flashed before. `Fram::new_raw()` leaves it as it is.
- The SAC DAC's timer load triggers `LoadTrigger::TB2_1` and `TB2_2` take a `sac::DacTrigger`, made from TB2's `pwm1` or `pwm2` with the new `PwmUninit::into_dac_trigger()`, instead of a `SubTimer`, whose output never rose, so the DAC never loaded (breaking).
- Fixed the blocking I2C transfers clearing every interrupt flag, which lost a START that addressed an `I2cMasterSlave` during its own transfer: they now clear only their own transaction's flags, and wait for their STOP. `return_to_master()` clears the arbitration lost flag, `schedule_stop()` no longer clears the slave's flags, and `transaction()` with adjacent writes no longer hangs. Add `tx_buf_empty()` and `stop_sent()` to the non-blocking master interface. `poll()` of an `I2cMasterSlave` returns `WouldBlock` in master mode (breaking).
- Add `SpiSlave::reset()`, the reset workaround of erratum USCI47. The UART, SPI, ADC, PWM, capture, eCOMP, `sys` and BSL docs cite the errata that apply to them (USCI42, USCI45, USCI47, USCI50, ADC50, ADC63, TB25, COMP12, CPU46, BSL18) with every affected family's errata sheet.
- Add `serial::BaudConfig`, which `SerialConfig::new()` takes in place of the baud rate: the baud-rate settings, worked out by the compiler in a `const` with `BaudConfig::new()`, or taken from the user's guide's table of recommended settings or its detailed error calculation with `BaudConfig::from_fields()`, so that the program doesn't calculate them. `DcoclkFreqSel::freq()` is a `const fn`, so such a constant can use the DCO frequency. New example `uart_baud_config`.
- UART: the baud-rate calculation needs fewer 32-bit divisions and looks UCBRSx up in a table, and the compiler works it out wherever the clock frequency and the baud rate are constants, also with several UARTs.
- `ClockConfig::freeze()` and `try_freeze()` work out the clock frequencies, the FLL settings and the FRAM wait states before they write a register, so that a constant clock configuration compiles to its register writes and the DCO trim, without the 32-bit division routine.
- Fixed the embedded-hal I2C `transaction()` with operations without bytes: an empty read at the end left the bus held without a STOP, and a write after an empty write waited forever. A read without bytes puts nothing on the bus, so the STOP now comes after the last operation that isn't one, and an operation after a write without bytes, which sends the address and a STOP, now starts with a START. embedded-hal 0.2's `write_read()` with an empty buffer now sends the STOP with the write.
- GPIO batches no longer read and rewrite PxOUT unchanged when no pin of the port has a pull resistor, which makes `split()` up to 16 bytes smaller per port.
- `InfoMemory` no longer stores a pointer, and indexing it no longer links the panic and formatting code in debug builds.
- ADC: reading several channels with `read_count()` shares one copy of its code, and a constant `AdcConfig` compiles to its register writes in debug builds too.
- Batches from `Batch::new` write only the register bits that differ from their reset values, so a port whose pins all stay inputs costs no code. `Parts::batch()` still writes every register: configure a port whose registers the program wrote through the PAC before with `Batch::new(port).split(&pmm).batch()`. `Batch` has a new type parameter for this, `FromReset` (the default) or `FromParts` (breaking for code that names the type of a `Parts::batch()` result).
- Add `free()` to the UART (`Tx`, `Rx`), SPI (`Spi`, `SpiSlave`) and I2C drivers: it gives back the eUSCI, held in reset, and the pins, as floating GPIO inputs, so that the eUSCI can be set up again, with other pins for example. The two halves of `SerialConfig::split()` only go back together, with `Tx::free_with()`, so one half can't be given back while the other still uses the eUSCI. `tx_only()` and `rx_only()` return a `TxOnly` and an `RxOnly`, which have `free()` and deref to the `Tx` and `Rx` in them, so a function that takes `&mut Tx<USCI>` takes them too (breaking for code that stores a `tx_only()` or `rx_only()` result in a variable of type `Tx` or `Rx`). The IR modulator holds the transmitter it sends from and stands in for it, so the program sends through the modulator as before, and `disable()` gives the transmitter back. `Mfm::free()` gives back eUSCI_B1 and the MFM's pins; `SpiConfig::mfm_slave()` returns an `MfmSlave` for `Mfm::new()`, which has no pins to give back.

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
