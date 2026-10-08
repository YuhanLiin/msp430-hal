//! Backup Memory.
//! [BAK_MEM_SIZE] bytes of volatile memory that survives system resets. It keeps its content through
//! LPM3.5 (SLAU445I chapter 7, p. 309), whose wake-up is a BOR (SLAU445I 1.4.3.2, p. 42), and a reset
//! doesn't load a value into it (reset value "Undefined", SLAU445I Table 7-1, p. 310).
//!
//! This memory is still volatile however, so it won't survive power loss. The backup memory is powered in all modes except LPM4.5.
//!
//! The peripheral access crate exposes the backup memory as individual 16-bit registers (SLAU445I
//! Table 7-1, p. 310).
//! This module provides helper functions for reinterpreting the backup memory as various array types.
//!
//! After choosing the most convenient data type for your application call the relevant method,
//! such as [`BackupMemory::as_u8s()`], to recieve a mutable reference to the backup memory.

use crate::device_specific::_pac::Bkmem;
pub use crate::device_specific::BAK_MEM_SIZE;
use core::mem::size_of;

/// Helper struct with static methods for interpreting the backup memory into more usable forms
pub struct BackupMemory;

macro_rules! as_x {
    ($fn_name: ident, $arr: ty) => {
        #[doc = "Interpret the backup memory as a `&mut"]
        #[doc = stringify!($arr)]
        #[doc = "`. See also: [BAK_MEM_SIZE]"]
        #[inline(always)]
        pub fn $fn_name(_reg: Bkmem) -> &'static mut $arr {
            const { assert!(core::mem::size_of::<$arr>() == BAK_MEM_SIZE) }
            // The BAKMEM registers are consecutive, and word or byte accessible (SLAU445I Table 7-1, p. 310)
            unsafe { &mut *(Bkmem::PTR as *mut $arr) }
        }
    };
}

impl BackupMemory {
    as_x!(as_u8s,   [u8;  BAK_MEM_SIZE/size_of::<u8>()  ]);
    as_x!(as_u16s,  [u16; BAK_MEM_SIZE/size_of::<u16>() ]);
    as_x!(as_u32s,  [u32; BAK_MEM_SIZE/size_of::<u32>() ]);
    as_x!(as_u64s,  [u64; BAK_MEM_SIZE/size_of::<u64>() ]);
    as_x!(as_u128s, [u128;BAK_MEM_SIZE/size_of::<u128>()]);

    as_x!(as_i8s,   [i8;  BAK_MEM_SIZE/size_of::<i8>()  ]);
    as_x!(as_i16s,  [i16; BAK_MEM_SIZE/size_of::<i16>() ]);
    as_x!(as_i32s,  [i32; BAK_MEM_SIZE/size_of::<i32>() ]);
    as_x!(as_i64s,  [i64; BAK_MEM_SIZE/size_of::<i64>() ]);
    as_x!(as_i128s, [i128;BAK_MEM_SIZE/size_of::<i128>()]);
}
