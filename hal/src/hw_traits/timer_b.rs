#[allow(unused_imports)] // The device modules import the shared traits through this
pub use crate::hw_traits::timer_base::*;

// Timer_B adds the compare latches (CLLD) and the counter length (CNTL) to the shared features
// (SLAU445I 14.1.1, p. 391). Timer_B is on the MSP430FR2x5x (SLASEC4D 6.10.9, p. 73) and the MSP430FR247x
// (SLASEO7C Table 9-15, p. 59).
macro_rules! timer_b_impl {
    ($($args:tt)*) => { $crate::hw_traits::timer_base::timer_base_impl!(B, $($args)*); };
}
pub(crate) use timer_b_impl;
