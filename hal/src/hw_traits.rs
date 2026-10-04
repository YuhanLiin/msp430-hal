pub trait Steal {
    unsafe fn steal() -> Self;
}

#[cfg(feature = "ecomp")]
pub mod ecomp;
pub mod eusci;
pub mod gpio;
#[cfg(feature = "sac")]
pub mod sac;
pub mod timer_a;
// Timer_B, on the devices that have one (the `timer_b` feature in Cargo.toml lists them)
#[cfg(feature = "timer_b")]
pub mod timer_b;
pub mod timer_base;
