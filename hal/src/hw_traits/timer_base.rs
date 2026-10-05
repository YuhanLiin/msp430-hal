// Functionality common to both TimerA and TimerB (Timer_B is identical to Timer_A apart from the
// differences in SLAU445I 14.1.1, p. 391)

use super::Steal;

// TASSEL/TBSSEL values (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
pub enum Tbssel {
    Tbxclk,
    Aclk,
    Smclk,
    Inclk,
}

/// Timer clock divider (ID: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
pub enum TimerDiv {
    /// No division
    _1,
    /// Divide by 2
    _2,
    /// Divide by 4
    _4,
    /// Divide by 8
    _8,
}

/// Timer expansion clock divider, applied on top of the normal clock divider (TAIDEX/TBIDEX: SLAU445I
/// 13.2.1.1, p. 370; SLAU445I Table 13-9, p. 389; SLAU445I Table 14-11, p. 414)
pub enum TimerExDiv {
    /// No division
    _1,
    /// Divide by 2
    _2,
    /// Divide by 3
    _3,
    /// Divide by 4
    _4,
    /// Divide by 5
    _5,
    /// Divide by 6
    _6,
    /// Divide by 7
    _7,
    /// Divide by 8
    _8,
}

// OUTMOD values (SLAU445I Table 13-2, p. 376; SLAU445I Table 14-4, p. 401)
pub enum Outmod {
    Out,
    Set,
    ToggleReset,
    SetReset,
    Toggle,
    Reset,
    ToggleSet,
    ResetSet,
}

// CM values (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
pub enum Cm {
    NoCap,
    RisingEdge,
    FallingEdge,
    BothEdges,
}

// CCIS values (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
pub enum Ccis {
    InputA,
    InputB,
    Gnd,
    Vcc,
}

// CLLD values: when a Timer_B compare latch TBxCLn loads the new TBxCCRn value (SLAU445I Table 14-2, p. 400;
// SLAU445I Table 14-8, p. 411)
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Clld {
    // "immediately when TBxCCRn is written"
    Immediately = 0b00,
    // "when TBxR counts to 0"
    AtZero = 0b01,
    // "when TBxR counts to 0 for up and continuous modes", and "to the old TBxCL0 value or to 0 for up/down
    // mode"
    AtZeroOrTop = 0b10,
    // "when TBxR counts to the old TBxCLn value"
    AtOldValue = 0b11,
}

// Accessors of a timer's TAxCTL/TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409 to
// p. 410), TAxR/TBxR (SLAU445I Table 13-5, p. 385; SLAU445I Table 14-7, p. 410), TAxIV/TBxIV (SLAU445I
// Table 13-8, p. 388; SLAU445I Table 14-10, p. 414) and TAxEX0/TBxEX0 (SLAU445I Table 13-9, p. 389;
// SLAU445I Table 14-11, p. 414)
pub trait TimerBase: Steal {
    /// Reset timer countdown (TBCLR: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
    fn reset(&self);

    // The three mode functions write MC, set TBCLR and clear TBIFG (SLAU445I Table 13-4, p. 384; SLAU445I
    // Table 14-6, p. 409 to p. 410)
    /// Set to upmode, reset timer, and clear interrupts
    fn upmode(&self);
    /// Set to continuous mode, reset timer, and clear interrupts
    fn continuous(&self);
    /// Set to up/down mode, reset timer, and clear interrupts
    fn updown_mode(&self);
    /// The counting mode (MC: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
    fn mode_rd(&self) -> u8;
    /// Set the counter length (Timer_B CNTL: 0 = 16-bit, 1 = 12-bit, 2 = 10-bit, 3 = 8-bit). Timer_A has none
    /// (SLAU445I Table 14-6, p. 409; SLAU445I 14.1.1, p. 391).
    fn set_cntl(&self, cntl: u8);
    /// Set the compare latch grouping (Timer_B TBCLGRP: 0 = none, 1 = pairs, 2 = triples, 3 = all). Timer_A has
    /// none (SLAU445I Table 14-6, p. 409; SLAU445I Table 14-3, p. 400).
    fn set_tbclgrp(&self, tbclgrp: u8);

    /// Configure the timer in one write of TAxCTL/TBxCTL, with MC = 0: the clock source and divider (TBSSEL
    /// and ID: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409), and on a Timer_B the counter
    /// length and the compare latch grouping (CNTL and TBCLGRP, as for `set_cntl` and `set_tbclgrp`)
    fn config_ctl(&self, tbssel: Tbssel, div: TimerDiv, cntl: u8, tbclgrp: u8);

    /// Check if timer is stopped (MC = 0 in TBxCTL: SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
    fn is_stopped(&self) -> bool;

    /// Stop timer (MC = 0: SLAU445I Table 13-1, p. 371; SLAU445I Table 14-1, p. 394)
    fn stop(&self);

    /// Resume a *stopped* timer. Assumes the previous mode was 'stop'.
    /// Atomic, fast. It sets MC in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409).
    fn resume(&self, mode: RunningMode);

    /// Change a timer's mode. Non-atomic, slower. To go from one mode to another the user's guide stops the
    /// timer first (MC = 0) (SLAU445I 13.2.3, p. 371; 14.2.3, p. 394). It writes MC in TBxCTL (SLAU445I
    /// Table 13-4, p. 384; SLAU445I Table 14-6, p. 409).
    fn change_mode(&self, mode: Mode);

    /// Set expansion register clock divider settings (TBIDEX: SLAU445I Table 13-9, p. 389; SLAU445I
    /// Table 14-11, p. 414)
    fn set_tbidex(&self, tbidex: TimerExDiv);

    // TAIFG/TBIFG, the timer overflow flag in TAxCTL/TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I
    // Table 14-6, p. 410)
    fn tbifg_rd(&self) -> bool;
    fn tbifg_clr(&self);

    // TAIE/TBIE, its interrupt enable in TAxCTL/TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6,
    // p. 410)
    fn tbie_set(&self);
    fn tbie_clr(&self);

    // TAxIV/TBxIV; a read clears the highest pending flag (SLAU445I Table 13-8, p. 388; SLAU445I
    // Table 14-10, p. 414; SLAU445I 13.2.6.2, p. 380)
    fn tbxiv_rd(&self) -> crate::timer::TimerVector;

    /// Get the current timer value (TBxR: SLAU445I Table 13-5, p. 385; SLAU445I Table 14-7, p. 410).
    fn get_tbxr(&self) -> u16;
}

// MC values (SLAU445I Table 13-1, p. 371; SLAU445I Table 14-1, p. 394)
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RunningMode {
    Up = 0b01,
    Continuous = 0b10,
    UpDown = 0b11,
}
#[repr(u8)]
pub enum Mode {
    Stop = 0b00,
    Up = 0b01,
    Continuous = 0b10,
    UpDown = 0b11,
}

// Accessors of one capture/compare block: its TAxCCRn/TBxCCRn (SLAU445I Table 13-7, p. 388; SLAU445I
// Table 14-9, p. 413) and TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 386 to p. 387; SLAU445I Table 14-8,
// p. 411 to p. 412)
pub trait CCRn<C>: Steal {
    // TAxCCRn/TBxCCRn (SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413). `set_ccrn` stops a running
    // Timer_A for the write (SLAU445I 13.2.4.2, p. 376); `set_ccrn_stopped` is for a timer the caller has
    // stopped (MC = 0), and leaves out the check.
    fn set_ccrn(&self, count: u16);
    fn set_ccrn_stopped(&self, count: u16);
    fn get_ccrn(&self) -> u16;

    // TAxCCTLn/TBxCCTLn, for compare mode with an output mode or for capture mode (SLAU445I Table 13-6,
    // p. 386; SLAU445I Table 14-8, p. 411). `config_outmod` stops a running timer for the write (SLAU445I
    // 13.2.7, p. 382; SLAU445I 14.2.7, p. 407); `config_outmod_stopped` is for a timer the caller has
    // stopped (MC = 0), and leaves out the check. On a Timer_B, `config_outmod_stopped` sets when the compare
    // latch loads (CLLD, see `set_clld`) in the same write.
    fn config_outmod(&self, outmod: Outmod);
    fn config_outmod_stopped(&self, outmod: Outmod, clld: Clld);
    fn config_cap_mode(&self, cm: Cm, ccis: Ccis);

    // CCIFG in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
    fn ccifg_rd(&self) -> bool;
    fn ccifg_clr(&self);

    // CCIE in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
    fn ccie_set(&self);
    fn ccie_clr(&self);

    // COV and CCIFG in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
    fn cov_ccifg_rd(&self) -> (bool, bool);
    fn cov_ccifg_clr(&self);
    fn cov_clr(&self);

    /// The output mode (OUTMOD: SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
    fn outmod_rd(&self) -> u8;
    /// Switch between an output mode and its inverse (set ↔ reset, toggle/reset ↔ toggle/set,
    /// set/reset ↔ reset/set: SLAU445I Table 13-2, p. 376) by setting or clearing the top OUTMOD bit. The
    /// other two bits stay, so this never passes through mode 0 (SLAU445I 13.2.5.1.3, p. 379; 14.2.5.1.3,
    /// p. 404, note "Switching between output modes").
    fn set_outmod_high_bit(&self, set: bool);
    /// Switch the capture input between GND and VCC (CCIS bit 0), for a software capture (SLAU445I
    /// 13.2.4.1.1, p. 376; 14.2.4.1.1, p. 399)
    fn toggle_ccis_low_bit(&self);
    /// Set when the compare latch loads (Timer_B CLLD, see [`Clld`]). Timer_A has no compare latch
    /// (SLAU445I 14.1.1, p. 391). In up mode `AtZero` and `AtZeroOrTop` load at once on the MSP430FR2x5x and
    /// MSP430FR247x (SLAZ695J TB25, p. 11; SLAZ726B TB25, p. 8).
    fn set_clld(&self, clld: Clld);
    /// When the compare latch loads, as a [`Clld`] value. 0 on a Timer_A, which has no compare latch
    /// (SLAU445I 14.1.1, p. 391).
    fn clld_rd(&self) -> u8;
}

// The capture/compare registers TAxCCR0 to TAxCCR6 and TBxCCR0 to TBxCCR6 (SLAU445I Table 13-3, p. 383;
// SLAU445I Table 14-5, p. 408). How many a timer has depends on the device, see the CapCmpTimer traits.
/// Label for capture-compare register 0
pub struct CCR0;
/// Label for capture-compare register 1
pub struct CCR1;
/// Label for capture-compare register 2
pub struct CCR2;
/// Label for capture-compare register 3
pub struct CCR3;
/// Label for capture-compare register 4
pub struct CCR4;
/// Label for capture-compare register 5
pub struct CCR5;
/// Label for capture-compare register 6
pub struct CCR6;

// Write a Timer_B-only field, or nothing for Timer_A: CLLD in TBxCCTLn (SLAU445I Table 14-8, p. 411) or
// CNTL in TBxCTL (SLAU445I Table 14-6, p. 409), which Timer_A lacks (SLAU445I 14.1.1, p. 391)
macro_rules! timer_b_field {
    (A, $reg:expr, $field:ident, $value:expr) => { let _ = $value; };
    (B, $reg:expr, $field:ident, $value:expr) => {
        $reg.modify(|_, w| unsafe { w.$field().bits($value) });
    };
}
pub(crate) use timer_b_field;

// The write of TAxCTL/TBxCTL that configures a timer: TASSEL/TBSSEL and ID, and on a Timer_B CNTL and
// TBCLGRP, which Timer_A lacks (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409; SLAU445I 14.1.1,
// p. 391). The other fields are 0, so MC = 0: the timer stops.
macro_rules! ctl_write {
    (A, $ctl:expr, $txssel:ident, $tbssel:expr, $div:expr, $cntl:expr, $tbclgrp:expr) => {{
        let _ = ($cntl, $tbclgrp);
        $ctl.write(|w| unsafe { w.$txssel().bits($tbssel as u8).id().bits($div as u8) });
    }};
    (B, $ctl:expr, $txssel:ident, $tbssel:expr, $div:expr, $cntl:expr, $tbclgrp:expr) => {
        $ctl.write(|w| unsafe { w
            .$txssel().bits($tbssel as u8)
            .id().bits($div as u8)
            .cntl().bits($cntl)
            .tbclgrp().bits($tbclgrp)
        });
    };
}
pub(crate) use ctl_write;

// A write of TAxCCTLn/TBxCCTLn: OUTMOD, and on a Timer_B CLLD, which Timer_A lacks, with CAP = 0 (compare
// mode) and every other field cleared (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411; SLAU445I
// 14.1.1, p. 391)
macro_rules! cctl_write {
    (A, $cctl:expr, $outmod:expr, $clld:expr) => {{
        let _ = $clld;
        $cctl.write(|w| unsafe { w.outmod().bits($outmod as u8) });
    }};
    (B, $cctl:expr, $outmod:expr, $clld:expr) => {
        $cctl.write(|w| unsafe { w.outmod().bits($outmod as u8).clld().bits($clld as u8) });
    };
}
pub(crate) use cctl_write;

// Read a Timer_B-only field, or 0 for Timer_A, which lacks it
macro_rules! timer_b_read {
    (A, $value:expr) => { 0 };
    (B, $value:expr) => { $value };
}
pub(crate) use timer_b_read;

// Run `$body` with the timer stopped (MC = 0) and its mode set back afterwards. The user's guide lists
// OUTMOD among the controls "designed not to be dynamically updated while the timer is running", to be
// changed after "Write zero to the mode control bits (MC = 0)" (SLAU445I 13.2.7, p. 382; SLAU445I 14.2.7,
// p. 407), and "TI recommends stopping the timer (MC = 0) before changing the OUTMOD bits" (SLAU445I
// 13.2.5.1.3, p. 379). "Switching to Stop mode can be performed at any time" (SLAU445I 13.2.7, p. 382). In
// stop mode "The timer is halted" (SLAU445I Table 13-1, p. 371; SLAU445I Table 14-1, p. 394), and set back
// to up mode "the timer starts counting up from the value in TAxR" (SLAU445I 13.2.3.1.1, p. 371), so it
// only loses the counts of the few cycles it was stopped. MC is bits 5-4 of TAxCTL/TBxCTL (SLAU445I
// Table 13-4, p. 384; SLAU445I Table 14-6, p. 409). The restore takes MC from the register value read at
// the start, so the compiler can keep the bits in place instead of shifting them down and back up.
macro_rules! timer_stopped {
    ($ctl:expr, $body:expr) => {{
        let ctl = $ctl;
        let r = ctl.read();
        let running = !r.mc().is_stop();
        if running {
            unsafe { ctl.clear_bits(|w| w.mc().stop()) };
        }
        let ret = $body;
        if running {
            unsafe { ctl.set_bits(|w| w.mc().bits(r.mc().bits())) };
        }
        ret
    }};
}
pub(crate) use timer_stopped;

// Run `$body` with a Timer_A stopped, as `timer_stopped`; a Timer_B runs it as it is. For Timer_A "In
// Compare mode, the timer should be stopped by writing the MC bits to zero (MC = 0) before writing new data
// to TAxCCRn" (SLAU445I 13.2.4.2, p. 376). A Timer_B buffers new TBxCCRn values in its compare latches
// instead (SLAU445I 14.2.4.2.1, p. 400), and its chapter has no such note.
macro_rules! timer_a_stopped {
    (B, $ctl:expr, $body:expr) => { $body };
    (A, $ctl:expr, $body:expr) => { $crate::hw_traits::timer_base::timer_stopped!($ctl, $body) };
}
pub(crate) use timer_a_stopped;

// Mark Timer_B peripherals
macro_rules! timer_b_marker {
    (A, $TBx:ident) => {};
    (B, $TBx:ident) => { impl crate::timer::TimerB for $TBx {} };
}
pub(crate) use timer_b_marker;

// TAxIV/TBxIV as a `TimerVector`. The field and its values are TAIV, TACCRn and TAIFG on a Timer_A, and TBIV,
// TBCCRn and TBIFG on a Timer_B (SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414).
macro_rules! timer_vector {
    (A, $iv:expr) => {{
        use crate::timer::TimerVector;
        let r = $iv.read();
        let iv = r.taiv();
        if iv.is_taccr1() { TimerVector::SubTimer1 }
        else if iv.is_taccr2() { TimerVector::SubTimer2 }
        else if iv.is_taccr3() { TimerVector::SubTimer3 }
        else if iv.is_taccr4() { TimerVector::SubTimer4 }
        else if iv.is_taccr5() { TimerVector::SubTimer5 }
        else if iv.is_taccr6() { TimerVector::SubTimer6 }
        else if iv.is_taifg() { TimerVector::MainTimer }
        else { TimerVector::NoInterrupt }
    }};
    (B, $iv:expr) => {{
        use crate::timer::TimerVector;
        let r = $iv.read();
        let iv = r.tbiv();
        if iv.is_tbccr1() { TimerVector::SubTimer1 }
        else if iv.is_tbccr2() { TimerVector::SubTimer2 }
        else if iv.is_tbccr3() { TimerVector::SubTimer3 }
        else if iv.is_tbccr4() { TimerVector::SubTimer4 }
        else if iv.is_tbccr5() { TimerVector::SubTimer5 }
        else if iv.is_tbccr6() { TimerVector::SubTimer6 }
        else if iv.is_tbifg() { TimerVector::MainTimer }
        else { TimerVector::NoInterrupt }
    }};
}
pub(crate) use timer_vector;

macro_rules! ccrn_impl {
    ($kind:ident, $TBx:ident, $tbxctl:ident, $CCRn:ident, $tbxcctln:ident, $tbxccrn:ident) => {
        impl CCRn<$CCRn> for $TBx {
            // TAxCCRn/TBxCCRn (SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413), written with a
            // Timer_A stopped (SLAU445I 13.2.4.2, p. 376)
            #[inline(always)]
            fn set_ccrn(&self, count: u16) {
                $crate::hw_traits::timer_base::timer_a_stopped!($kind, self.$tbxctl(), {
                    self.$tbxccrn().write(|w| unsafe { w.bits(count) })
                });
            }

            // TAxCCRn/TBxCCRn of a stopped timer (SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413)
            #[inline(always)]
            fn set_ccrn_stopped(&self, count: u16) {
                self.$tbxccrn().write(|w| unsafe { w.bits(count) });
            }

            // TAxCCRn/TBxCCRn (SLAU445I Table 13-7, p. 388; SLAU445I Table 14-9, p. 413)
            #[inline(always)]
            fn get_ccrn(&self) -> u16 { self.$tbxccrn().read().bits() }

            // A write: OUTMOD, with CAP = 0 (compare mode) and every other field cleared (SLAU445I
            // Table 13-6, p. 386; SLAU445I Table 14-8, p. 411), with the timer stopped (SLAU445I 13.2.7,
            // p. 382; SLAU445I 14.2.7, p. 407)
            #[inline(always)]
            fn config_outmod(&self, outmod: Outmod) {
                $crate::hw_traits::timer_base::timer_stopped!(self.$tbxctl(), {
                    self.$tbxcctln().write(|w| unsafe { w.outmod().bits(outmod as u8) })
                });
            }

            // The same write, of a stopped timer, with CLLD on a Timer_B (SLAU445I Table 14-8, p. 411): the
            // user's guide configures TBxCCTLn in one step ("Apply desired configuration to TBxCCRn, TBIDEX,
            // and TBxCCTLn", SLAU445I 14.2.7, p. 407)
            #[inline(always)]
            fn config_outmod_stopped(&self, outmod: Outmod, clld: Clld) {
                $crate::hw_traits::timer_base::cctl_write!($kind, self.$tbxcctln(), outmod, clld);
            }

            #[inline(always)]
            fn config_cap_mode(&self, cm: Cm, ccis: Ccis) {
                // A write of TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411).
                // CAP = 1 selects capture mode (SLAU445I 13.2.4.1, p. 374). SCS synchronizes the capture with
                // the timer clock, which the user's guide recommends (SLAU445I 13.2.4.1, p. 375; SLAU445I
                // 14.2.4.1, p. 398).
                self.$tbxcctln().write(|w| unsafe { w
                    .cap().set_bit()
                    .scs().set_bit()
                    .cm().bits(cm as u8)
                    .ccis().bits(ccis as u8)
                });
            }

            // CCIFG in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
            #[inline(always)]
            fn ccifg_rd(&self) -> bool { self.$tbxcctln().read().ccifg().bit() }

            // CCIFG in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
            #[inline(always)]
            fn ccifg_clr(&self) {
                unsafe { self.$tbxcctln().clear_bits(|w| w.ccifg().clear_bit()) };
            }

            // CCIE in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
            #[inline(always)]
            fn ccie_set(&self) { unsafe { self.$tbxcctln().set_bits(|w| w.ccie().set_bit()) }; }

            // CCIE in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
            #[inline(always)]
            fn ccie_clr(&self) { unsafe { self.$tbxcctln().clear_bits(|w| w.ccie().clear_bit()) }; }

            // COV and CCIFG in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
            #[inline(always)]
            fn cov_ccifg_rd(&self) -> (bool, bool) {
                let cctl = self.$tbxcctln().read();
                (cctl.cov().bit(), cctl.ccifg().bit())
            }

            // COV in TAxCCTLn/TBxCCTLn, which "must be reset with software" (SLAU445I Table 13-6, p. 387;
            // SLAU445I Table 14-8, p. 412)
            #[inline(always)]
            fn cov_clr(&self) {
                unsafe { self.$tbxcctln().clear_bits(|w| w.cov().clear_bit()) };
            }

            // COV and CCIFG in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 387; SLAU445I Table 14-8, p. 412)
            #[inline(always)]
            fn cov_ccifg_clr(&self) {
                unsafe {
                    self.$tbxcctln().clear_bits(|w| w
                        .ccifg().clear_bit()
                        .cov().clear_bit())
                };
            }

            // OUTMOD in TAxCCTLn/TBxCCTLn (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
            #[inline(always)]
            fn outmod_rd(&self) -> u8 { self.$tbxcctln().read().outmod().bits() }

            #[inline(always)]
            fn set_outmod_high_bit(&self, set: bool) {
                // The top bit of OUTMOD (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411), set or
                // cleared in one instruction, with the timer stopped (SLAU445I 13.2.7, p. 382; SLAU445I
                // 14.2.7, p. 407)
                $crate::hw_traits::timer_base::timer_stopped!(self.$tbxctl(), {
                    if set {
                        unsafe { self.$tbxcctln().set_bits(|w| w.outmod().bits(0b100)) };
                    } else {
                        unsafe { self.$tbxcctln().clear_bits(|w| w.outmod().bits(0b011)) };
                    }
                });
            }

            #[inline(always)]
            fn toggle_ccis_low_bit(&self) {
                // The low bit of CCIS, toggled in one instruction: GND (10b) to VCC (11b) and back
                // (SLAU445I Table 13-6, p. 386; SLAU445I Table 14-8, p. 411)
                unsafe { self.$tbxcctln().toggle_bits(|w| w.ccis().bits(0b01)) };
            }

            // CLLD is a Timer_B field (SLAU445I Table 14-8, p. 411)
            #[inline(always)]
            fn set_clld(&self, clld: Clld) {
                $crate::hw_traits::timer_base::timer_b_field!($kind, self.$tbxcctln(), clld, clld as u8);
            }

            // CLLD in TBxCCTLn (SLAU445I Table 14-8, p. 411)
            #[inline(always)]
            fn clld_rd(&self) -> u8 {
                $crate::hw_traits::timer_base::timer_b_read!($kind, self.$tbxcctln().read().clld().bits())
            }
        }
    };
}
pub(crate) use ccrn_impl;

// Implements the traits on a Timer_A or Timer_B. The registers are TAxCTL/TBxCTL, TAxEX0/TBxEX0, TAxIV/TBxIV,
// TAxR/TBxR and the TAxCCTLn/TBxCCTLn and TAxCCRn/TBxCCRn pairs (SLAU445I Table 13-3, p. 383; SLAU445I
// Table 14-5, p. 408). The field names differ: TACLR/TBCLR, TAIFG/TBIFG, TAIDEX/TBIDEX, TAIE/TBIE and
// TASSEL/TBSSEL (SLAU445I Table 13-4, p. 384; SLAU445I Table 13-9, p. 389; SLAU445I Table 14-6, p. 409 to
// p. 410; SLAU445I Table 14-11, p. 414).
macro_rules! timer_base_impl {
    (
        $kind:ident, // A for Timer_A, B for Timer_B
        $TBx:ident, $tbx:ident, $tbxctl:ident, $tbxex:ident, $tbxiv:ident, $tbxr:ident, // Timer registers
        $txclr:ident, $txifg:ident, $txidex:ident, $txie:ident, $txssel:ident, // Register field names (differ between TimerA and TimerB)
        $([$CCRn:ident, $tbxcctln:ident, $tbxccrn:ident]),* // CCR registers
    ) => {
        impl Steal for $TBx {
            #[inline(always)]
            unsafe fn steal() -> Self {
                $TBx::steal()
            }
        }

        impl TimerBase for $TBx {
            // TBCLR clears the count, the clock divider logic and the count direction (SLAU445I Table 13-4,
            // p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn reset(&self) {
                unsafe { self.$tbxctl().set_bits(|w| w.$txclr().set_bit()) };
            }

            // The three mode functions set TBCLR along with MC, which also restarts the divider logic as a
            // TBIDEX change needs (SLAU445I 13.3.6, p. 389; SLAU445I 14.3.6, p. 414). MC values: SLAU445I
            // Table 13-1, p. 371; SLAU445I Table 14-1, p. 394.
            #[inline(always)]
            fn upmode(&self) {
                self.$tbxctl().modify(|r, w| {
                    unsafe { w.bits(r.bits())
                        .$txclr().set_bit()
                        .$txifg().clear_bit()
                        .mc().up()
                    }
                });
            }

            // TBxCTL: TBCLR, TBIFG and MC = 10b (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn continuous(&self) {
                self.$tbxctl().modify(|r, w| {
                    unsafe { w.bits(r.bits())
                        .$txclr().set_bit()
                        .$txifg().clear_bit()
                        .mc().continuous()
                    }
                });
            }

            // TBxCTL: TBCLR, TBIFG and MC = 11b (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn updown_mode(&self) {
                self.$tbxctl().modify(|r, w| {
                    unsafe { w.bits(r.bits())
                        .$txclr().set_bit()
                        .$txifg().clear_bit()
                        .mc().updown()
                    }
                });
            }

            // MC in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn mode_rd(&self) -> u8 {
                self.$tbxctl().read().mc().bits()
            }

            // CNTL in TBxCTL, Timer_B only (SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn set_cntl(&self, cntl: u8) {
                $crate::hw_traits::timer_base::timer_b_field!($kind, self.$tbxctl(), cntl, cntl);
            }

            // TBCLGRP in TBxCTL, Timer_B only (SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn set_tbclgrp(&self, tbclgrp: u8) {
                $crate::hw_traits::timer_base::timer_b_field!($kind, self.$tbxctl(), tbclgrp, tbclgrp);
            }

            // A write of TBSSEL and ID, and on a Timer_B CNTL and TBCLGRP, in TBxCTL (SLAU445I Table 13-4,
            // p. 384; SLAU445I Table 14-6, p. 409), so MC = 0 and the timer stops: the clock source, the
            // dividers, the counter length and the grouping are only to be changed while it's stopped
            // (SLAU445I 13.2.1.1, p. 370, note "Timer_A dividers"; 13.2.7, p. 382; 14.2.1.2, p. 393; 14.2.7,
            // p. 407), where the user's guide writes TBxCTL in one step ("Apply desired configuration to
            // TBxCTL including the MC bits", SLAU445I 14.2.7, p. 407)
            #[inline(always)]
            fn config_ctl(&self, tbssel: Tbssel, div: TimerDiv, cntl: u8, tbclgrp: u8) {
                $crate::hw_traits::timer_base::ctl_write!($kind, self.$tbxctl(), $txssel, tbssel, div, cntl, tbclgrp);
            }

            // MC = 0 is stop mode (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn is_stopped(&self) -> bool {
                self.$tbxctl().read().mc().is_stop()
            }

            // Clears MC in TBxCTL: stop mode (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn stop(&self) {
                unsafe { self.$tbxctl().clear_bits(|w| w.mc().stop()) };
            }

            // Writes MC in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn change_mode(&self, mode: Mode) {
                self.$tbxctl().modify(|_,w| unsafe{ w.mc().bits(mode as u8) });
            }

            // Sets MC in TBxCTL from 00b (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 409)
            #[inline(always)]
            fn resume(&self, mode: RunningMode) {
                unsafe { self.$tbxctl().set_bits(|w| w.mc().bits(mode as u8)) };
            }

            // TBIDEX in TBxEX0 (SLAU445I Table 13-9, p. 389; SLAU445I Table 14-11, p. 414)
            #[inline(always)]
            fn set_tbidex(&self, tbidex: TimerExDiv) {
                self.$tbxex().write(|w| unsafe { w.$txidex().bits(tbidex as u8) });
            }

            // TBIFG in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 410)
            #[inline(always)]
            fn tbifg_rd(&self) -> bool {
                self.$tbxctl().read().$txifg().bit()
            }

            // TBIFG in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 410)
            #[inline(always)]
            fn tbifg_clr(&self) {
                unsafe { self.$tbxctl().clear_bits(|w| w.$txifg().clear_bit()) };
            }

            // TBIE in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 410)
            #[inline(always)]
            fn tbie_set(&self) {
                unsafe { self.$tbxctl().set_bits(|w| w.$txie().set_bit()) };
            }

            // TBIE in TBxCTL (SLAU445I Table 13-4, p. 384; SLAU445I Table 14-6, p. 410)
            #[inline(always)]
            fn tbie_clr(&self) {
                unsafe { self.$tbxctl().clear_bits(|w| w.$txie().clear_bit()) };
            }

            // TBxIV (SLAU445I Table 13-8, p. 388; SLAU445I Table 14-10, p. 414). Reading it clears the
            // highest pending flag (SLAU445I 13.2.6.2, p. 380; 14.2.6.2, p. 405)
            #[inline(always)]
            fn tbxiv_rd(&self) -> crate::timer::TimerVector {
                $crate::hw_traits::timer_base::timer_vector!($kind, self.$tbxiv())
            }

            // TBxR (SLAU445I Table 13-5, p. 385; SLAU445I Table 14-7, p. 410)
            #[inline(always)]
            fn get_tbxr(&self) -> u16 {
                self.$tbxr().read().bits()
            }
        }

        // Timer_B only: CLLD and CNTL (SLAU445I 14.1.1, p. 391)
        $crate::hw_traits::timer_base::timer_b_marker!($kind, $TBx);

        $($crate::hw_traits::timer_base::ccrn_impl!($kind, $TBx, $tbxctl, $CCRn, $tbxcctln, $tbxccrn);)*
    };
}
pub(crate) use timer_base_impl;
