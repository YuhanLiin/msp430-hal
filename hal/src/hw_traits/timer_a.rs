#[allow(unused_imports)] // The device modules import the shared traits through this
pub use crate::hw_traits::timer_base::*;

// Timer_A has no features beyond the shared ones that the HAL uses (its SCCI bit, which Timer_B lacks,
// goes unused: SLAU445I 14.1.1, p. 391). The MSP430FR2x5x has no Timer_A, only Timer0_B3 to Timer3_B7
// (SLASEC4D 6.10.9, p. 73).
#[allow(unused_macros)] // Not every device has a Timer_A
macro_rules! timer_a_impl {
    ($($args:tt)*) => { $crate::hw_traits::timer_base::timer_base_impl!(A, $($args)*); };
}
#[allow(unused_imports)]
pub(crate) use timer_a_impl;
