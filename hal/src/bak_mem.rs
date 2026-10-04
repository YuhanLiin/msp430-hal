//! Backup Memory.
//! [BAK_MEM_SIZE] bytes of volatile memory that survives system resets. It keeps its content through
//! LPM3.5 (SLAU445I chapter 7, p. 309; SLASEC4D 6.10.10, p. 76; SLASE59F 6.10.10, p. 52; SLASEO7C
//! 9.10.10, p. 61; SLASEE4C 6.10.10, p. 55), whose wake-up is a BOR (SLAU445I 1.4.3.2, p. 42), and a
//! reset doesn't load a value into it (reset value "Undefined", SLAU445I Table 7-1, p. 310).
//!
//! This memory is still volatile however, so it won't survive power loss. The backup memory is powered in
//! all modes except LPM4.5 (SLASEC4D Table 6-1, p. 62; SLASE59F Table 6-1, p. 41; SLASEO7C Table 9-1,
//! p. 45; SLASEE4C Table 6-1, p. 45).
//!
//! The peripheral access crate exposes the backup memory as 16 individual 16-bit registers (BAKMEM0 to
//! BAKMEM15 from base address 0660h: SLASEC4D Table 6-54, p. 92; SLASE59F Table 6-43, p. 68; SLASEO7C
//! Table 9-54, p. 81; SLASEE4C Table 6-35, p. 69).
//! This module provides helper functions for reinterpreting the backup memory as various array types.
//!
//! After choosing the most convenient data type for your application call the relevant method,
//! such as [`BackupMemory::as_u8s()`], to recieve a mutable reference to the backup memory.

use crate::device_specific::_pac::Bkmem;
pub use crate::device_specific::BAK_MEM_SIZE;
use core::mem::size_of;

/// Helper struct with static methods for interpreting the backup memory into more usable forms (the
/// BAKMEM registers: SLAU445I Table 7-1, p. 310)
pub struct BackupMemory;

macro_rules! as_x {
    ($fn_name: ident, $arr: ty) => {
        #[doc = "Interpret the backup memory as a `&mut"]
        #[doc = stringify!($arr)]
        #[doc = "`. See also: [BAK_MEM_SIZE]"]
        #[inline(always)]
        pub fn $fn_name(_reg: Bkmem) -> &'static mut $arr {
            const { assert!(core::mem::size_of::<$arr>() == BAK_MEM_SIZE) }
            // BAKMEM0 to BAKMEM15 at the Backup Memory base address 0660h, word or byte accessible (SLAU445I
            // Table 7-1, p. 310; SLASEO7C Table 9-54, p. 81 and the matching tables of the other data sheets)
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
