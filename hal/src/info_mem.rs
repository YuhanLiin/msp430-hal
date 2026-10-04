//! Information Memory.
//! [INFO_MEM_SIZE] bytes of non-volatile memory.
//!
//! A single instance of [InfoMemory] is returned by [`Pmm::new()`](crate::pmm::Pmm::new()).
//!
//! Because the information memory has write protection, access is managed via a write method.
//!
//! For convenience there is also a method [`InfoMemory::into_unprotected()`] that disables write protection
//! and returns the infomation memory directly as an array instead.
//!

use core::ops::Index;

use crate::_pac;
pub use crate::device_specific::INFO_MEM_SIZE;

/// Start address of the information memory
const INFO_MEM_START_ADDR: *mut u8 = 0x1800 as *mut u8;
const SYSCFG0_PASSWORD: u8 = 0xA5;

/// A struct that manages writing and reading from information memory.
pub struct InfoMemory {
    info_mem: &'static mut [u8; INFO_MEM_SIZE],
}
impl InfoMemory {
    /// Creates a mutable reference to the information memory segment. Don't call this method more than once.
    #[inline(always)]
    pub(crate) fn new(_sys: _pac::Sys) -> Self {
        Self { info_mem: unsafe { &mut *(INFO_MEM_START_ADDR as *mut [u8; INFO_MEM_SIZE]) } }
    }

    /// Temporarily grants mutable access to the information memory as an array.
    ///
    /// Write protection is automatically disabled before calling the closure and restored immediately after it returns.
    /// The closure runs with interrupts disabled, as the user's guide recommends for FRAM writes, so keep it
    /// short (SLAU445I 1.9.3, p. 45: "completed within as short a time as possible with interrupts disabled").
    #[inline]
    pub fn write<T>(&mut self, f: impl FnOnce(&mut [u8; INFO_MEM_SIZE]) -> T) -> T {
        critical_section::with(|_| {
            Self::disable_write_protect();
            let ret = f(self.info_mem);
            Self::enable_write_protect();
            ret
        })
    }

    /// Disable write protection and directly return the info memory as an array.
    #[inline]
    pub fn into_unprotected(self) -> &'static mut [u8; INFO_MEM_SIZE] {
        Self::disable_write_protect();
        self.info_mem
    }

    #[inline(always)]
    fn disable_write_protect() {
        let sys = unsafe { _pac::Sys::steal() };
        sys.syscfg0().modify(|_, w| unsafe { w
            .frwppw().bits(SYSCFG0_PASSWORD)
            .dfwp().clear_bit()
        });
    }

    #[inline(always)]
    fn enable_write_protect() {
        let sys = unsafe { _pac::Sys::steal() };
        sys.syscfg0().modify(|_, w| unsafe { w
            .frwppw().bits(SYSCFG0_PASSWORD)
            .dfwp().set_bit()
        });
    }
}

impl Index<usize> for InfoMemory {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output { &self.info_mem[index] }
}
